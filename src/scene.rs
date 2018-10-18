use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use cgmath::Point3;

use glium::backend::Facade;
use glium::VertexBuffer;

use crate::aabb::AABB;
use crate::bvh::{SplitMode, BVH};
use crate::index_ptr::IndexPtr;
use crate::material::{GPUMaterial, Material};
use crate::mesh::{GPUMesh, Mesh};
use crate::obj_load;
use crate::stats;
use crate::triangle::{RTTriangle, RTTriangleBuilder};
use crate::vertex::{RawVertex, Vertex};
use crate::Float;

pub struct SceneBuilder {
    split_mode: SplitMode,
}

impl SceneBuilder {
    pub fn new() -> Self {
        Self {
            split_mode: SplitMode::SAH,
        }
    }

    #[allow(dead_code)]
    pub fn with_bvh(&mut self, split_mode: SplitMode) -> &mut Self {
        self.split_mode = split_mode;
        self
    }

    pub fn build(&self, scene_file: &Path) -> Arc<Scene> {
        let obj = obj_load::load_obj(scene_file)
            .unwrap_or_else(|err| panic!("Failed to load scene {:?}: {}", scene_file, err));
        let mut arc_scene = Scene::from_obj(&obj);
        let scene = Arc::get_mut(&mut arc_scene).unwrap();
        scene.build_bvh(self.split_mode);
        // Lights need to be constructed after bvh build
        scene.construct_lights();
        arc_scene
    }
}

/// Scene containing all the CPU resources
pub struct Scene {
    pub vertices: Vec<Vertex>,
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub triangles: Vec<RTTriangle>,
    pub lights: Vec<RTTriangle>,
    pub aabb: AABB,
    pub bvh: Option<BVH>,
}

/// Scene containing resources for GPU rendering
// Separate from Scene because GPU resources are not thread safe
pub struct GPUScene {
    pub meshes: Vec<GPUMesh>,
    pub materials: Vec<GPUMaterial>,
    pub vertex_buffer: VertexBuffer<RawVertex>,
}

impl Scene {
    fn empty() -> Arc<Self> {
        Arc::new(Self {
            vertices: Vec::new(),
            meshes: Vec::new(),
            materials: Vec::new(),
            triangles: Vec::new(),
            lights: Vec::new(),
            aabb: AABB::empty(),
            bvh: None,
        })
    }

    pub fn from_obj(obj: &obj_load::Object) -> Arc<Self> {
        let _t = stats::time("Convert");

        let mut arc_scene = Self::empty();
        let scene = Arc::get_mut(&mut arc_scene).unwrap();
        let mut vertex_map = HashMap::new();
        let mut material_map = HashMap::new();
        for range in &obj.material_ranges {
            let material_i = match material_map.get(&range.name) {
                Some(&i) => i,
                None => {
                    let obj_mat = obj
                        .materials
                        .get(&range.name)
                        .unwrap_or_else(|| panic!("Couldn't find material {}!", range.name));
                    let material = Material::new(obj_mat);
                    let i = scene.materials.len();
                    scene.materials.push(material);
                    material_map.insert(&range.name, i);
                    i
                }
            };
            let mut mesh = Mesh::new(material_i);
            for tri in &obj.triangles[range.start_i..range.end_i] {
                let mut tri_builder = RTTriangleBuilder::new();
                let planar_normal = calculate_normal(tri, &obj);
                for index_vertex in &tri.index_vertices {
                    let vertex_i = match vertex_map.get(index_vertex) {
                        // Vertex has already been added
                        Some(&i) => {
                            mesh.indices.push(i as u32);
                            i
                        }
                        None => {
                            let mut save = true;
                            let pos = obj.positions[index_vertex.pos_i];

                            let tex_coords = match index_vertex.tex_i {
                                Some(tex_i) => obj.tex_coords[tex_i],
                                None => [0.0; 2],
                            };
                            let normal = match index_vertex.normal_i {
                                Some(normal_i) => obj.normals[normal_i],
                                None => {
                                    // Don't save vertices without normals.
                                    // Otherwise the first tri defines the normal
                                    // for all remaining uses of the vertex.
                                    save = false;
                                    planar_normal
                                }
                            };

                            mesh.indices.push(scene.vertices.len() as u32);
                            if save {
                                vertex_map.insert(index_vertex, scene.vertices.len());
                            }
                            scene.vertices.push(Vertex::new(pos, normal, tex_coords));
                            scene.vertices.len() - 1
                        }
                    };
                    tri_builder.add_vertex(scene.vertex_ptr(vertex_i));
                }
                let triangle = tri_builder
                    .build(scene.material_ptr(material_i))
                    .expect("Failed to build tri!");
                scene.aabb.add_aabb(&triangle.aabb());
                scene.triangles.push(triangle);
            }
            if !mesh.indices.is_empty() {
                scene.meshes.push(mesh);
            }
        }
        arc_scene
    }

    // Warning: this will reorder triangles!
    fn build_bvh(&mut self, split_mode: SplitMode) {
        let (bvh, permutation) = BVH::build(&self.triangles, split_mode);
        self.bvh = Some(bvh);
        self.triangles = permutation
            .iter()
            .map(|i| self.triangles[*i].clone())
            .collect();
    }

    // Should be called after BVH build
    fn construct_lights(&mut self) {
        let _t = stats::time("Lights");
        if self.bvh.is_none() {
            println!("Constructing lights when there is no bvh!");
        }
        for tri in &self.triangles {
            let material = &tri.material;
            if material.emissive.is_some() {
                self.lights.push(tri.clone());
            }
        }
    }

    /// Load the textures + vertex and index buffers to the GPU
    pub fn upload_data<F: Facade>(&self, facade: &F) -> GPUScene {
        let _t = stats::time("Upload data");
        let raw_vertices: Vec<RawVertex> = self.vertices.iter().map(|v| v.into()).collect();
        let vertex_buffer =
            VertexBuffer::new(facade, &raw_vertices).expect("Failed to create vertex buffer!");
        let mut meshes = Vec::new();
        let mut materials = Vec::new();
        for mesh in &self.meshes {
            meshes.push(mesh.upload_data(facade));
        }
        for material in &self.materials {
            materials.push(material.upload_textures(facade));
        }
        GPUScene {
            meshes,
            materials,
            vertex_buffer,
        }
    }

    fn material_ptr(&self, i: usize) -> IndexPtr<Material> {
        IndexPtr::new(&self.materials, i)
    }

    fn vertex_ptr(&self, i: usize) -> IndexPtr<Vertex> {
        IndexPtr::new(&self.vertices, i)
    }

    /// Get the center of the scene as defined by the bounding box
    pub fn center(&self) -> Point3<Float> {
        self.aabb.center()
    }

    /// Get the approximate size of the scene
    pub fn size(&self) -> Float {
        self.aabb.longest_edge()
    }
}

/// Calculate planar normal for a triangle
fn calculate_normal(triangle: &obj_load::Triangle, obj: &obj_load::Object) -> [f32; 3] {
    let pos_i1 = triangle.index_vertices[0].pos_i;
    let pos_i2 = triangle.index_vertices[1].pos_i;
    let pos_i3 = triangle.index_vertices[2].pos_i;
    let pos_1 = obj.positions[pos_i1];
    let pos_2 = obj.positions[pos_i2];
    let pos_3 = obj.positions[pos_i3];
    let u = [
        pos_2[0] - pos_1[0],
        pos_2[1] - pos_1[1],
        pos_2[2] - pos_1[2],
    ];
    let v = [
        pos_3[0] - pos_1[0],
        pos_3[1] - pos_1[1],
        pos_3[2] - pos_1[2],
    ];
    let normal = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let length = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
    [normal[0] / length, normal[1] / length, normal[2] / length]
}
