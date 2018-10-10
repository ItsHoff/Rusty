use std::collections::HashMap;
use std::path::Path;
use std::vec::Vec;

use cgmath::Point3;

use glium::backend::Facade;
use glium::VertexBuffer;

use crate::aabb::AABB;
use crate::bvh::{SplitMode, BVH};
use crate::material::{GPUMaterial, Material};
use crate::mesh::{GPUMesh, Mesh};
use crate::obj_load;
use crate::stats;
use crate::triangle::{RTTriangle, RTTriangleBuilder};
use crate::vertex::Vertex;

/// Scene containing all the CPU resources
pub struct Scene {
    pub vertices: Vec<Vertex>,
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub triangles: Vec<RTTriangle>,
    pub lights: Vec<RTTriangle>,
    pub aabb: AABB,
    pub bvh: BVH,
}

/// Scene containing minimum resources for GPU rendering
pub struct GPUScene {
    pub meshes: Vec<GPUMesh>,
    pub materials: Vec<GPUMaterial>,
    pub vertex_buffer: VertexBuffer<Vertex>,
}

impl Scene {
    pub fn new(scene_path: &Path) -> Scene {
        let mut scene = Scene {
            vertices: Vec::new(),
            meshes: Vec::new(),
            materials: Vec::new(),
            triangles: Vec::new(),
            lights: Vec::new(),
            aabb: AABB::empty(),
            bvh: BVH::empty(),
        };
        scene.load_scene(scene_path);
        let (bvh, permutation) = BVH::build(&scene.triangles, SplitMode::SAH);
        scene.bvh = bvh;
        scene.triangles = permutation
            .iter()
            .map(|i| scene.triangles[*i].clone())
            .collect();
        scene
    }

    /// Load a scene from the given path
    fn load_scene(&mut self, scene_path: &Path) {
        let obj = obj_load::load_obj(scene_path)
            .unwrap_or_else(|err| panic!("Failed to load scene {:?}: {}", scene_path, err));

        let mut _t = stats::time("Convert scene");
        // Closure to calculate planar normal for a triangle
        let calculate_normal = |triangle: &obj_load::Triangle| -> [f32; 3] {
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
        };

        // Group the polygons by materials for easy rendering
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
                    let i = self.materials.len();
                    self.materials.push(material);
                    material_map.insert(&range.name, i);
                    i
                }
            };
            let material = &self.materials[material_i];
            let mut mesh = Mesh::new(material_i);
            for tri in &obj.triangles[range.start_i..range.end_i] {
                let mut tri_builder = RTTriangleBuilder::new();
                let default_tex_coords = [0.0; 2];
                for index_vertex in &tri.index_vertices {
                    match vertex_map.get(index_vertex) {
                        // Vertex has already been added
                        Some(&i) => {
                            mesh.indices.push(i as u32);
                            tri_builder.add_vertex(self.vertices[i]);
                        }
                        None => {
                            let pos = obj.positions[index_vertex.pos_i];
                            self.aabb.add_point(&Point3::from(pos));

                            let tex_coords = match index_vertex.tex_i {
                                Some(tex_i) => obj.tex_coords[tex_i],
                                None => default_tex_coords,
                            };
                            let normal = match index_vertex.normal_i {
                                Some(normal_i) => obj.normals[normal_i],
                                None => calculate_normal(tri),
                            };

                            mesh.indices.push(self.vertices.len() as u32);
                            vertex_map.insert(index_vertex, self.vertices.len());
                            self.vertices.push(Vertex {
                                pos,
                                normal,
                                tex_coords,
                            });
                            tri_builder.add_vertex(*self.vertices.last().unwrap());
                        }
                    }
                }
                let triangle = tri_builder.build(material_i).expect("Failed to build tri!");
                if material.emissive.is_some() {
                    self.lights.push(triangle.clone());
                }
                self.triangles.push(triangle);
            }
            if !mesh.indices.is_empty() {
                self.meshes.push(mesh);
            }
        }
    }

    /// Load the textures + vertex and index buffers to the GPU
    pub fn upload_data<F: Facade>(&self, facade: &F) -> GPUScene {
        let _t = stats::time("Upload data");
        let vertex_buffer =
            VertexBuffer::new(facade, &self.vertices).expect("Failed to create vertex buffer!");
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

    /// Get the center of the scene as defined by the bounding box
    pub fn center(&self) -> Point3<f32> {
        self.aabb.center()
    }

    /// Get the approximate size of the scene
    pub fn size(&self) -> f32 {
        self.aabb.longest_edge()
    }
}
