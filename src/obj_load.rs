/// Simple module for loading wavefront object files

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::str::SplitWhitespace;
use std::vec::Vec;

/// Indices of vertex attributes in attribute vectors
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub struct IndexVertex {
    pub pos_i: usize,
    pub tex_i: Option<usize>,
    pub normal_i: Option<usize>,
}

impl IndexVertex {
    fn new() -> IndexVertex {
        IndexVertex {
            ..Default::default()
        }
    }
}

/// Representation of loaded polygon
#[derive(Debug, Default, Clone)]
pub struct Polygon {
    pub index_vertices: Vec<IndexVertex>,
    /// Name of polygons group
    pub group: Option<String>,
    /// Number of polygons smoothing group
    pub smoothing_group: Option<u32>,
    /// Name of polygons material
    pub material: Option<String>
}

impl Polygon {
    fn new(state: &ParseState) -> Polygon {
        Polygon {
            group: {
                match state.current_group {
                    Some(ref range) => Some(range.name.clone()),
                    None => None
                }
            },
            smoothing_group: state.current_smoothing_group,
            material: {
                match state.current_material {
                    Some(ref range) => Some(range.name.clone()),
                    None => None
                }
            },
            ..Default::default()
        }
    }

    /// Convert polygon to triangles
    pub fn to_triangles(&self) -> Vec<Triangle> {
        if self.index_vertices.len() == 3 {
            vec!(Triangle {
                    index_vertices: [self.index_vertices[0], self.index_vertices[1], self.index_vertices[2]],
                    group: self.group.clone(),
                    smoothing_group: self.smoothing_group,
                    material: self.material.clone()
                })
        } else {
            let mut tris = Vec::new();
            let tip = self.index_vertices[0];
            let mut v1 = self.index_vertices[1];
            // Go round the polygon and attach current two vertices to the central vertex
            for vertex in &self.index_vertices[2..] {
                let tri = Triangle {
                    index_vertices: [tip, v1, *vertex],
                    group: self.group.clone(),
                    smoothing_group: self.smoothing_group,
                    material: self.material.clone()
                };
                tris.push(tri);
                v1 = *vertex;
            }
            tris
        }
    }
}

/// Representation of loaded polygon
#[derive(Debug, Default, Clone)]
pub struct Triangle {
    pub index_vertices: [IndexVertex; 3],
    /// Name of triangles group
    pub group: Option<String>,
    /// Number of triangles smoothing group
    pub smoothing_group: Option<u32>,
    /// Name of triangles material
    pub material: Option<String>
}

/// Named range that represents ranges of certain properties
#[derive(Clone, Debug)]
pub struct Range {
    pub name: String,
    /// Inclusive start [start_i, end_i)
    pub start_i: usize,
    /// Exclusive end [start_i, end_i)
    pub end_i: usize
}

impl Range {
    /// Create a new named range [start, start)
    /// End should be set when whole range has been processed
    fn new(name: &str, start: usize) -> Range {
        Range { name: name.to_string(),
                start_i: start,
                end_i: start
        }
    }
}

/// Representation of a loaded material
#[derive(Debug, Default, Clone)]
pub struct Material {
    /// Name of the material
    pub name: String,
    /// Ambient color
    pub c_ambient: Option<[f32; 3]>,
    /// Diffuse color
    pub c_diffuse: Option<[f32; 3]>,
    /// Specular color
    pub c_specular: Option<[f32; 3]>,
    /// Translucent color
    pub c_translucency: Option<[f32; 3]>,
    /// Emissive color
    pub c_emissive: Option<[f32; 3]>,
    /// Illumination model
    pub illumination_model: Option<u32>,
    /// Opacity
    pub opacity: Option<f32>,
    /// Specular shininess
    pub shininess: Option<f32>,
    /// Sharpness of reflections
    pub sharpness: Option<f32>,
    /// Index of refraction
    pub refraction_i: Option<f32>,
    /// Ambient color texture
    pub tex_ambient: Option<PathBuf>,
    /// Diffuse color texture
    pub tex_diffuse: Option<PathBuf>,
    /// Specular color texture
    pub tex_specular: Option<PathBuf>,
    /// Specular shininess texture
    pub tex_shininess: Option<PathBuf>,
    /// Opacity texture
    pub tex_opacity: Option<PathBuf>,
    /// Displacement texture
    pub tex_disp: Option<PathBuf>,
    /// Decal texture
    pub tex_decal: Option<PathBuf>,
    /// Bump texture
    pub tex_bump: Option<PathBuf>,
}

impl Material {
    fn new(name: &str) -> Material {
        Material { name: name.to_string(),
                 ..Default::default()
        }
    }
}

/// Struct containing the loaded object file properties
#[derive(Default)]
pub struct Object {
    /// List of loaded vertex positions
    /// Indexed by index_vertices in triangles
    pub positions: Vec<[f32; 3]>,
    /// List of loaded vertex normals
    /// Indexed by index_vertices in triangles
    pub normals: Vec<[f32; 3]>,
    /// List of loaded vertex texture coordinates
    /// Indexed by index_vertices in triangles
    pub tex_coords: Vec<[f32; 2]>,
    /// List of loaded triangles
    pub triangles: Vec<Triangle>,
    /// Ranges of loaded groups
    /// Ranges index the triangles list
    pub group_ranges: Vec<Range>,
    /// Ranges of loaded materials
    /// Ranges index the triangles list
    pub material_ranges: Vec<Range>,
    /// Map of loaded materials
    pub materials: HashMap<String, Material>
}

impl Object {
    fn new() -> Object {
        Object { ..Default::default() }
    }
}

/// Internal representation of the parse state
#[derive(Default)]
struct ParseState {
    /// Paths to the material libraries defined in the object file
    mat_libs: Vec<PathBuf>,
    /// Group that is currently active
    current_group: Option<Range>,
    /// Smoothing group that is currently active
    current_smoothing_group: Option<u32>,
    /// Material that is currently active
    current_material: Option<Range>,
}

impl ParseState {
    fn new() -> ParseState {
        ParseState { ..Default::default() }
    }
}

/// Parse a single integer from the split input line
fn parse_int(split_line: &mut SplitWhitespace) -> Result<u32, Box<Error>> {
    let item = try!(split_line.next().ok_or("Expected value after key"));
    Ok(try!(item.parse()))
}

/// Parse a single float from the split input line
fn parse_float(split_line: &mut SplitWhitespace) -> Result<f32, Box<Error>> {
    let item = try!(split_line.next().ok_or("Expected value after key"));
    Ok(try!(item.parse()))
}

/// Parse two floats from the split input line
#[cfg_attr(feature="cargo-clippy", allow(needless_range_loop))]
fn parse_float2(split_line: &mut SplitWhitespace) -> Result<[f32; 2], Box<Error>> {
    let mut float2 = [0.0f32; 2];
    for i in 0..2 {
        let item = try!(split_line.next().ok_or("Float 2 didn't have 2 floats"));
        float2[i] = try!(item.parse());
    }
    Ok(float2)
}

/// Parse three floats from the split input line
#[cfg_attr(feature="cargo-clippy", allow(needless_range_loop))]
fn parse_float3(split_line: &mut SplitWhitespace) -> Result<[f32; 3], Box<Error>> {
    let mut float3 = [0.0f32; 3];
    for i in 0..3 {
        let item = try!(split_line.next().ok_or("Float 3 didn't have 3 floats"));
        float3[i] = try!(item.parse());
    }
    Ok(float3)
}

/// Parse a string from the split input line
fn parse_string(split_line: &mut SplitWhitespace) -> Result<String, Box<Error>> {
    let string = try!(split_line.next().ok_or("Couldnt not find string."));
    Ok(string.to_string())
}

/// Parse a path from the split input line
fn parse_path(split_line: &mut SplitWhitespace) -> Result<PathBuf, Box<Error>> {
    let path_str = try!(parse_string(split_line));
    let mut path = PathBuf::new();
    for part in path_str.split(|c| c == '/' || c == '\\') {
        path.push(part);
    }
    Ok(path)
}

/// Parse a polygon from the split input line
fn parse_polygon(split_line: &mut SplitWhitespace, obj: &Object, state: &ParseState)
              -> Result<Polygon, Box<Error>> {
    let mut polygon = Polygon::new(state);
    for item in split_line {
        let mut index_vertex = IndexVertex::new();
        for (i, num) in item.split('/').enumerate() {
            if i >= 3 {
                break;
            }
            if num != "" {
                let num: isize = try!(num.parse());
                if num < 0 {
                    match i {
                        0 => index_vertex.pos_i = (obj.positions.len() as isize + num) as usize,
                        1 => index_vertex.tex_i = Some((obj.tex_coords.len() as isize + num) as usize),
                        2 => index_vertex.normal_i = Some((obj.normals.len() as isize + num) as usize),
                        _ => unreachable!()
                    }
                } else {
                    match i {
                        0 => index_vertex.pos_i = (num - 1) as usize,
                        1 => index_vertex.tex_i = Some((num - 1) as usize),
                        2 => index_vertex.normal_i = Some((num - 1) as usize),
                        _ => unreachable!()
                    }
                }
            }
        }
        polygon.index_vertices.push(index_vertex);
    }
    // TODO: Sanity check
    Ok(polygon)
}

/// Load an object found at the given path
pub fn load_obj(obj_path: &Path) -> Result<Object, Box<Error>> {
    let mut obj = Object::new();
    let mut state = ParseState::new();
    let obj_dir = try!(obj_path.parent().ok_or("Couldn't get object directory"));
    let obj_file = try!(File::open(obj_path));
    let obj_reader = BufReader::new(obj_file);
    for line in obj_reader.lines() {
        let line = line.expect("Failed to unwrap line");
        let mut split_line = line.split_whitespace();
        // Find the keyword of the line
        if let Some(key) = split_line.next() {
            match key {
                "f" => {
                    let polygon = try!(parse_polygon(&mut split_line, &obj, &state));
                    // Auto convert to triangles
                    // TODO: Make triangle conversion optional
                    obj.triangles.append(&mut polygon.to_triangles());
                },
                "g" | "o" => {
                    if let Some(mut range) = state.current_group {
                        range.end_i = obj.triangles.len();
                        obj.group_ranges.push(range);
                    };
                    let group_name = try!(parse_string(&mut split_line));
                    state.current_group = Some(Range::new(&group_name, obj.triangles.len()));
                },
                "mtllib" => state.mat_libs.push(obj_dir.join(try!(parse_path(&mut split_line)))),
                "s" => {
                    let val = try!(parse_string(&mut split_line));
                    if val == "off" || val == "0" {
                        state.current_smoothing_group = None;
                    } else {
                        state.current_smoothing_group = Some(try!(val.parse()));
                    }
                }
                "usemtl" => {
                    if let Some(mut range) = state.current_material {
                        range.end_i = obj.triangles.len();
                        obj.material_ranges.push(range);
                    };
                    let material_name = try!(parse_string(&mut split_line));
                    state.current_material = Some(Range::new(&material_name, obj.triangles.len()));
                },
                "v" => obj.positions.push(try!(parse_float3(&mut split_line))),
                "vn" => obj.normals.push(try!(parse_float3(&mut split_line))),
                "vt" => obj.tex_coords.push(try!(parse_float2(&mut split_line))),
                _ => {
                    if !key.starts_with('#') {
                        println!("Unrecognised key {}", key);
                    }
                }
            }
        }
    }
    // Close the open ranges
    if let Some(mut range) = state.current_group {
        range.end_i = obj.triangles.len();
        obj.group_ranges.push(range);
    };
    if let Some(mut range) = state.current_material {
        range.end_i = obj.triangles.len();
        obj.material_ranges.push(range);
    };
    // Load materials
    for matlib in state.mat_libs {
        obj.materials = try!(load_matlib(&obj_dir.join(matlib)));
    }
    Ok(obj)
}

/// Load materials from the material library to a map
pub fn load_matlib(matlib_path: &Path) -> Result<HashMap<String, Material>, Box<Error>> {
    let mut materials = HashMap::new();
    let mut current_material: Option<Material> = None;
    let matlib_dir = try!(matlib_path.parent().ok_or("Couldn't get material directory"));
    let matlib_file = try!(File::open(matlib_path));
    let matlib_reader = BufReader::new(matlib_file);
    for line in matlib_reader.lines() {
        let line = line.expect("Failed to unwrap line");
        let mut split_line = line.split_whitespace();
        // Find the keyword of the line
        if let Some(key) = split_line.next() {
            match key {
                "newmtl" => {
                    if let Some(material) = current_material {
                        materials.insert(material.name.clone(), material);
                    }
                    current_material = Some(Material::new(&try!(parse_string(&mut split_line))));
                }
                "Ka" => {
                    let material = try!(current_material.as_mut()
                                        .ok_or("Found material properties before newmtl!"));
                    material.c_ambient = Some(try!(parse_float3(&mut split_line)));
                },
                "Kd" => {
                    let material = try!(current_material.as_mut()
                                        .ok_or("Found material properties before newmtl!"));
                    material.c_diffuse = Some(try!(parse_float3(&mut split_line)));
                },
                "Ks" => {
                    let material = try!(current_material.as_mut()
                                        .ok_or("Found material properties before newmtl!"));
                    material.c_specular = Some(try!(parse_float3(&mut split_line)));
                },
                "Tf" => {
                    let material = try!(current_material.as_mut()
                                        .ok_or("Found material properties before newmtl!"));
                    material.c_translucency = Some(try!(parse_float3(&mut split_line)));
                },
                "Ke" => {
                    let material = try!(current_material.as_mut()
                                        .ok_or("Found material properties before newmtl!"));
                    material.c_emissive = Some(try!(parse_float3(&mut split_line)));
                },
                "illum" => {
                    let material = try!(current_material.as_mut()
                                        .ok_or("Found material properties before newmtl!"));
                    material.illumination_model = Some(try!(parse_int(&mut split_line)));
                },
                "d" | "Tr" => {
                    let material = try!(current_material.as_mut()
                                        .ok_or("Found material properties before newmtl!"));
                    material.opacity = Some(try!(parse_float(&mut split_line)));
                },
                "Ns" => {
                    let material = try!(current_material.as_mut()
                                        .ok_or("Found material properties before newmtl!"));
                    material.shininess = Some(try!(parse_float(&mut split_line)));
                },
                "sharpness" => {
                    let material = try!(current_material.as_mut()
                                        .ok_or("Found material properties before newmtl!"));
                    material.sharpness = Some(try!(parse_float(&mut split_line)));
                },
                "Ni" => {
                    let material = try!(current_material.as_mut()
                                        .ok_or("Found material properties before newmtl!"));
                    material.refraction_i = Some(try!(parse_float(&mut split_line)));
                },
                "map_Ka" => {
                    let material = try!(current_material.as_mut()
                                        .ok_or("Found material properties before newmtl!"));
                    material.tex_ambient = Some(matlib_dir.join(try!(parse_path(&mut split_line))));
                },
                "map_Kd" => {
                    let material = try!(current_material.as_mut()
                                        .ok_or("Found material properties before newmtl!"));
                    material.tex_diffuse = Some(matlib_dir.join(try!(parse_path(&mut split_line))));
                },
                "map_Ks" => {
                    let material = try!(current_material.as_mut()
                                        .ok_or("Found material properties before newmtl!"));
                    material.tex_specular = Some(matlib_dir.join(try!(parse_path(&mut split_line))));
                },
                "map_Ns" => {
                    let material = try!(current_material.as_mut()
                                        .ok_or("Found material properties before newmtl!"));
                    material.tex_shininess = Some(matlib_dir.join(try!(parse_path(&mut split_line))));
                },
                "map_d" | "map_Tr" => {
                    let material = try!(current_material.as_mut()
                                        .ok_or("Found material properties before newmtl!"));
                    material.tex_opacity = Some(matlib_dir.join(try!(parse_path(&mut split_line))));
                },
                "disp" => {
                    let material = try!(current_material.as_mut()
                                        .ok_or("Found material properties before newmtl!"));
                    material.tex_disp = Some(matlib_dir.join(try!(parse_path(&mut split_line))));
                },
                "decal" => {
                    let material = try!(current_material.as_mut()
                                        .ok_or("Found material properties before newmtl!"));
                    material.tex_decal = Some(matlib_dir.join(try!(parse_path(&mut split_line))));
                },
                "bump" | "map_Bump" | "map_bump" => {
                    let material = try!(current_material.as_mut()
                                        .ok_or("Found material properties before newmtl!"));
                    material.tex_bump = Some(matlib_dir.join(try!(parse_path(&mut split_line))));
                },
                _ => {
                    if !key.starts_with('#') {
                        println!("Unrecognised material key {}", key);
                    }
                }
            }
        }
    }
    let material = try!(current_material.ok_or("Didn't find any material definitions!"));
    materials.insert(material.name.clone(), material);
    Ok(materials)
}
