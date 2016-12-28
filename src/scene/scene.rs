use std::collections::HashMap;
use std::path::Path;
use std::vec::Vec;

use glium::backend::Facade;

use scene::obj_load;
use scene::Vertex;
use scene::mesh::Mesh;

/// Renderer representation of a scene
#[derive(Default)]
pub struct Scene {
    pub meshes: Vec<Mesh>,
    /// Bounding box of the scene
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Scene {
    /// Get the center of the scene as defined by the bounding box
    pub fn get_center(&self) -> [f32; 3] {
        let mut res = [0.0f32; 3];
        for i in 0..2 {
            res[i] = (self.min[i] + self.max[i]) / 2.0;
        }
        res
    }

    /// Get the longest edge of the bounding box
    pub fn get_size(&self) -> f32 {
        let mut max = 0.0f32;
        for i in 0..2 {
            max = max.max(self.max[i] - self.min[i]);
        }
        max
    }

    /// Update the bounding box with new position
    fn update_ranges(&mut self, new_pos: [f32; 3]) {
        for i in 0..2 {
            self.min[i] = self.min[i].min(new_pos[i]);
        }
        for i in 0..2 {
            self.max[i] = self.max[i].max(new_pos[i]);
        }
    }
}

/// Load a scene from the given path bind resources to given facade
pub fn load_scene<F: Facade>(scene_path: &Path, facade: &F) -> Scene {
    let mut scene = Scene { .. Default::default() };
    let obj = obj_load::load_obj(scene_path).expect("Failed to load.");

    // Closure to calculate planar normal for a polygon
    let calculate_normal = |polygon: &obj_load::Polygon| -> [f32; 3] {
        let pos_i1 = polygon.index_vertices[0][0].expect("No vertex positions!");
        let pos_i2 = polygon.index_vertices[1][0].expect("No vertex positions!");
        let pos_i3 = polygon.index_vertices[2][0].expect("No vertex positions!");
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
    for range in &obj.material_ranges {
        let obj_mat = obj.materials.get(&range.name)
            .expect(&::std::fmt::format(format_args!("Couldn't find material {}!", range.name)));
        let mut mesh = Mesh::new(&obj_mat);
        let mut vertex_map = HashMap::new();
        for tri in &obj.polygons[range.start_i..range.end_i] {
            let default_tex_coords= [0.0; 2];
            for index_vertex in &tri.index_vertices {
                match vertex_map.get(index_vertex) {
                    // Vertex has already been added
                    Some(&i) => mesh.indices.push(i),
                    None => {
                        // Add vertex to map
                        vertex_map.insert(index_vertex, mesh.vertices.len() as u32);
                        // Panic if there is no positions
                        let pos_i = index_vertex[0].expect("No vertex positions!");
                        let pos = obj.positions[pos_i];
                        scene.update_ranges(pos);

                        let tex_coords = match index_vertex[1] {
                            Some(tex_coords_i) => obj.tex_coords[tex_coords_i],
                            None => default_tex_coords
                        };
                        let normal = match index_vertex[2] {
                            Some(normal_i) => obj.normals[normal_i],
                            None => calculate_normal(tri)
                        };

                        mesh.indices.push(mesh.vertices.len() as u32);
                        mesh.vertices.push(Vertex { position: pos, normal: normal, tex_coords: tex_coords });
                    }
                }
            }
        }
        if !mesh.vertices.is_empty() {
            mesh.upload_data(facade);
            scene.meshes.push(mesh);
        }
    }
    scene
}
