use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use cgmath::prelude::*;
use cgmath::{Point3, Vector3};

use glium::backend::Facade;
use glium::VertexBuffer;

use rand;

use crate::aabb::AABB;
use crate::bvh::{BVHNode, SplitMode, BVH};
use crate::config::RenderConfig;
use crate::float::*;
use crate::index_ptr::IndexPtr;
use crate::intersect::{Hit, Intersect, Ray};
use crate::light::{AreaLight, Light};
use crate::material::{GPUMaterial, Material};
use crate::mesh::{GPUMesh, Mesh};
use crate::obj_load;
use crate::stats;
use crate::triangle::{Triangle, TriangleBuilder};
use crate::vertex::{RawVertex, Vertex};

pub struct SceneBuilder {
    split_mode: SplitMode,
}

impl SceneBuilder {
    pub fn new(config: &RenderConfig) -> Self {
        Self {
            split_mode: config.bvh_split,
        }
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
    vertices: Vec<Vertex>,
    meshes: Vec<Mesh>,
    materials: Vec<Material>,
    triangles: Vec<Triangle>,
    lights: Vec<AreaLight>,
    light_distribution: Vec<Float>,
    aabb: AABB,
    bvh: Option<BVH>,
}

/// Scene containing resources for GPU rendering
// Separate from Scene because GPU resources are not thread safe
pub struct GPUScene {
    pub meshes: Vec<GPUMesh>,
    pub materials: Vec<GPUMaterial>,
    pub vertex_buffer: VertexBuffer<RawVertex>,
}

/// Calculate planar normal for a triangle
fn calculate_normal(triangle: &obj_load::Triangle, obj: &obj_load::Object) -> [f32; 3] {
    let pos_i1 = triangle.index_vertices[0].pos_i;
    let pos_i2 = triangle.index_vertices[1].pos_i;
    let pos_i3 = triangle.index_vertices[2].pos_i;
    let pos_1 = Vector3::from_array(obj.positions[pos_i1]);
    let pos_2 = Vector3::from_array(obj.positions[pos_i2]);
    let pos_3 = Vector3::from_array(obj.positions[pos_i3]);
    let u = pos_2 - pos_1;
    let v = pos_3 - pos_1;
    let normal = u.cross(v).normalize();
    normal.into_array()
}

impl Scene {
    fn empty() -> Arc<Self> {
        Arc::new(Self {
            vertices: Vec::new(),
            meshes: Vec::new(),
            materials: Vec::new(),
            triangles: Vec::new(),
            lights: Vec::new(),
            light_distribution: Vec::new(),
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
                let mut tri_builder = TriangleBuilder::new();
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
                    .build(planar_normal, scene.material_ptr(material_i))
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
        // TODO: this could be done better
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
        for (i, tri) in self.triangles.iter().enumerate() {
            let material = &tri.material;
            if material.emissive.is_some() {
                self.lights.push(AreaLight::new(self.tri_ptr(i)));
            }
        }
        // Sort light by decreasing power
        self.lights.sort_unstable_by(|l1, l2| {
            let b1 = l1.power().luma();
            let b2 = l2.power().luma();
            b2.partial_cmp(&b1).unwrap()
        });
        let mut power_distr: Vec<Float> = self.lights.iter().map(|l| l.power().luma()).collect();
        let total_power: Float = power_distr.iter().sum();
        for power in &mut power_distr {
            *power /= total_power;
        }
        self.light_distribution = power_distr;
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
            materials.push(material.upload(facade));
        }
        GPUScene {
            meshes,
            materials,
            vertex_buffer,
        }
    }

    /// Get an IndexPtr to ith material
    fn material_ptr(&self, i: usize) -> IndexPtr<Material> {
        IndexPtr::new(&self.materials, i)
    }

    /// Get an IndexPtr to ith tri
    fn tri_ptr(&self, i: usize) -> IndexPtr<Triangle> {
        IndexPtr::new(&self.triangles, i)
    }

    /// Get an IndexPtr to ith vertex
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

    pub fn sample_light(&self) -> Option<(&dyn Light, Float)> {
        let r = rand::random::<Float>();
        let mut sum = 0.0;
        for (i, &val) in self.light_distribution.iter().enumerate() {
            sum += val;
            if r < sum {
                let light = &self.lights[i];
                return Some((light, val));
            }
        }
        None
    }

    pub fn intersect<'a>(
        &'a self,
        ray: &mut Ray,
        node_stack: &mut Vec<(&'a BVHNode, Float)>,
    ) -> Option<Hit> {
        let bvh = self.bvh.as_ref().unwrap();
        node_stack.push((bvh.root(), 0.0));
        let mut closest_hit = None;
        while let Some((node, t)) = node_stack.pop() {
            // We've already found a closer hit
            if ray.length <= t {
                continue;
            }
            if node.is_leaf() {
                for tri in &self.triangles[node.start_i..node.end_i] {
                    if let Some(hit) = tri.intersect(&ray) {
                        ray.length = hit.t;
                        closest_hit = Some(hit);
                    }
                }
            } else {
                let (left, right) = bvh.get_children(node).unwrap();
                // TODO: Could this work without pushing the next node to the stack
                let left_intersect = left.intersect(&ray);
                let right_intersect = right.intersect(&ray);
                if let Some(t_left) = left_intersect {
                    if let Some(t_right) = right_intersect {
                        // Put the closer hit on top
                        if t_left >= t_right {
                            node_stack.push((left, t_left));
                            node_stack.push((right, t_right));
                        } else {
                            node_stack.push((right, t_right));
                            node_stack.push((left, t_left));
                        }
                    } else {
                        node_stack.push((left, t_left));
                    }
                } else if let Some(t_right) = right_intersect {
                    node_stack.push((right, t_right));
                }
            }
        }
        closest_hit
    }
}
