mod bvh;
mod material;
mod mesh;
mod obj_load;

use std::collections::HashMap;
use std::path::Path;
use std::vec::Vec;

use cgmath::Point3;

use glium::VertexBuffer;
use glium::backend::Facade;

use renderer::{Vertex, RTTriangle, RTTriangleBuilder};

use aabb::AABB;
use self::bvh::BVHNode;
use self::mesh::Mesh;
pub use self::material::Material;


/// Renderer representation of a scene
#[derive(Default)]
pub struct Scene {
    pub vertices: Vec<Vertex>,
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub vertex_buffer: Option<VertexBuffer<Vertex>>,
    pub triangles: Vec<RTTriangle>,
    pub aabb: AABB,
    pub bvh: Vec<BVHNode>
}

#[cfg_attr(feature="clippy", allow(needless_range_loop))]
impl Scene {
    pub fn new<F: Facade>(scene_path: &Path, facade: &F) -> Scene {
        let mut scene = Scene { ..Default::default() };
        scene.load_scene(scene_path);
        scene.upload_data(facade);
        scene.bvh = bvh::build_object_median(&mut scene.triangles);
        scene
    }

    /// Load a scene from the given path
    fn load_scene(&mut self, scene_path: &Path) {
        let obj = obj_load::load_obj(scene_path).expect("Failed to load.");

        // Closure to calculate planar normal for a triangle
        let calculate_normal = |triangle: &obj_load::Triangle| -> [f32; 3] {
            let pos_i1 = triangle.index_vertices[0].pos_i;
            let pos_i2 = triangle.index_vertices[1].pos_i;
            let pos_i3 = triangle.index_vertices[2].pos_i;
            let pos_1 = obj.positions[pos_i1];
            let pos_2 = obj.positions[pos_i2];
            let pos_3 = obj.positions[pos_i3];
            let u = [pos_2[0] - pos_1[0],
                    pos_2[1] - pos_1[1],
                    pos_2[2] - pos_1[2]];
            let v = [pos_3[0] - pos_1[0],
                    pos_3[1] - pos_1[1],
                    pos_3[2] - pos_1[2]];
            [u[1]*v[2] - u[2]*v[1],
            u[2]*v[0] - u[0]*v[2],
            u[0]*v[1] - u[1]*v[0]]
        };

        // Group the polygons by materials for easy rendering
        let mut vertex_map = HashMap::new();
        for range in &obj.material_ranges {
            let obj_mat = obj.materials.get(&range.name)
                .expect(&::std::fmt::format(format_args!("Couldn't find material {}!", range.name)));
            let mut mesh = Mesh::new(self.materials.len());
            self.materials.push(Material::new(obj_mat));
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
                                None => default_tex_coords
                            };
                            let normal = match index_vertex.normal_i {
                                Some(normal_i) => obj.normals[normal_i],
                                None => calculate_normal(tri)
                            };

                            mesh.indices.push(self.vertices.len() as u32);
                            vertex_map.insert(index_vertex, self.vertices.len());
                            self.vertices.push(Vertex { pos, normal, tex_coords });
                            tri_builder.add_vertex(*self.vertices.last().unwrap());
                        }
                    }
                }
                self.triangles.push(tri_builder.build(self.materials.len() - 1)
                                    .expect("Failed to build tri!"));
            }
            if !mesh.indices.is_empty() {
                self.meshes.push(mesh);
            }
        }
    }

    /// Load the textures + vertex and index buffers to the GPU
    fn upload_data<F: Facade>(&mut self, facade: &F) {
        self.vertex_buffer = Some(VertexBuffer::new(facade, &self.vertices)
                                  .expect("Failed to create vertex buffer!"));
        for mesh in &mut self.meshes {
            mesh.upload_data(facade);
        }
        for material in &mut self.materials {
            material.upload_textures(facade);
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
