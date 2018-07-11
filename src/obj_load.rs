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
fn parse_int(split_line: &mut SplitWhitespace) -> Option<u32> {
    let item = split_line.next()?;
    item.parse().ok()
}

/// Parse a single float from the split input line
fn parse_float(split_line: &mut SplitWhitespace) -> Option<f32> {
    let item = split_line.next()?;
    item.parse().ok()
}

/// Parse two floats from the split input line
#[cfg_attr(feature="cargo-clippy", allow(needless_range_loop))]
fn parse_float2(split_line: &mut SplitWhitespace) -> Option<[f32; 2]> {
    let mut float2 = [0.0f32; 2];
    for i in 0..2 {
        let item = split_line.next()?;
        float2[i] = item.parse().ok()?;
    }
    Some(float2)
}

/// Parse three floats from the split input line
#[cfg_attr(feature="cargo-clippy", allow(needless_range_loop))]
fn parse_float3(split_line: &mut SplitWhitespace) -> Option<[f32; 3]> {
    let mut float3 = [0.0f32; 3];
    for i in 0..3 {
        let item = split_line.next()?;
        float3[i] = item.parse().ok()?;
    }
    Some(float3)
}

/// Parse a string from the split input line
fn parse_string(split_line: &mut SplitWhitespace) -> Option<String> {
    let string = split_line.next()?;
    Some(string.to_string())
}

/// Parse a path from the split input line
fn parse_path(split_line: &mut SplitWhitespace) -> Option<PathBuf> {
    let path_str = parse_string(split_line)?;
    let mut path = PathBuf::new();
    for part in path_str.split(|c| c == '/' || c == '\\') {
        path.push(part);
    }
    Some(path)
}

/// Parse a polygon from the split input line
fn parse_polygon(split_line: &mut SplitWhitespace, obj: &Object, state: &ParseState)
              -> Option<Polygon> {
    let mut polygon = Polygon::new(state);
    for item in split_line {
        let mut index_vertex = IndexVertex::new();
        for (i, num) in item.split('/').enumerate() {
            if i >= 3 {
                println!("Vertex with more than three properties");
                break;
            }
            if num != "" {
                let num: isize = num.parse().ok()?;
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
    if polygon.index_vertices.len() > 2 {
        Some(polygon)
    } else {
        println!("Polygon with less than three vertices");
        None
    }
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
                    if let Some(polygon) = parse_polygon(&mut split_line, &obj, &state) {
                        // Auto convert to triangles
                        // TODO: Make triangle conversion optional
                        obj.triangles.append(&mut polygon.to_triangles());
                    }
                },
                "g" | "o" => {
                    if let Some(mut range) = state.current_group {
                        range.end_i = obj.triangles.len();
                        obj.group_ranges.push(range);
                    };
                    let group_name = parse_string(&mut split_line)
                        .ok_or("Got a group without a name")?;
                    state.current_group = Some(Range::new(&group_name, obj.triangles.len()));
                },
                "mtllib" => {
                    if let Some(path) = parse_path(&mut split_line) {
                        state.mat_libs.push(obj_dir.join(path));
                    }
                }
                "s" => {
                    let val = parse_string(&mut split_line)
                        .ok_or("Empty smoothing group definition")?;
                    if val == "off" || val == "0" {
                        state.current_smoothing_group = None;
                    } else {
                        state.current_smoothing_group = Some(val.parse()?);
                    }
                }
                "usemtl" => {
                    if let Some(mut range) = state.current_material {
                        range.end_i = obj.triangles.len();
                        obj.material_ranges.push(range);
                    };
                    let material_name = parse_string(&mut split_line)
                        .ok_or("Tried to use material with no name")?;
                    state.current_material = Some(Range::new(&material_name, obj.triangles.len()));
                },
                "v" => {
                    if let Some(pos) = parse_float3(&mut split_line) {
                        obj.positions.push(pos);
                    }
                }
                "vn" => {
                    if let Some(normal) = parse_float3(&mut split_line) {
                        obj.normals.push(normal);
                    }
                }
                "vt" => {
                    if let Some(tex_coord) = parse_float2(&mut split_line) {
                        obj.tex_coords.push(tex_coord);
                    }
                }
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
        obj.materials = load_matlib(&obj_dir.join(matlib))?;
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
        let line = line.unwrap();
        let mut split_line = line.split_whitespace();
        // Find the keyword of the line
        if let Some(key) = split_line.next() {
            if key == "newmtl" {
                if let Some(material) = current_material {
                    materials.insert(material.name.clone(), material);
                }
                let material_name = parse_string(&mut split_line)
                    .ok_or("Tried to define a material with no name")?;
                current_material = Some(Material::new(&material_name));
            } else if !key.starts_with('#') {
                let material = current_material.as_mut()
                    .ok_or("Found material properties before newmtl!")?;
                match key {
                    "Ka" => {
                        material.c_ambient = parse_float3(&mut split_line);
                    },
                    "Kd" => {
                        material.c_diffuse = parse_float3(&mut split_line);
                    },
                    "Ks" => {
                        material.c_specular = parse_float3(&mut split_line);
                    },
                    "Tf" => {
                        material.c_translucency = parse_float3(&mut split_line);
                    },
                    "Ke" => {
                        material.c_emissive = parse_float3(&mut split_line);
                    },
                    "illum" => {
                        material.illumination_model = parse_int(&mut split_line);
                    },
                    "d" | "Tr" => {
                        material.opacity = parse_float(&mut split_line);
                    },
                    "Ns" => {
                        material.shininess = parse_float(&mut split_line);
                    },
                    "sharpness" => {
                        material.sharpness = parse_float(&mut split_line);
                    },
                    "Ni" => {
                        material.refraction_i = parse_float(&mut split_line);
                    },
                    "map_Ka" => {
                        material.tex_ambient = parse_path(&mut split_line)
                            .map(|path| matlib_dir.join(path));
                    },
                    "map_Kd" => {
                        material.tex_diffuse = parse_path(&mut split_line)
                            .map(|path| matlib_dir.join(path));
                    },
                    "map_Ks" => {
                        material.tex_specular = parse_path(&mut split_line)
                            .map(|path| matlib_dir.join(path));
                    },
                    "map_Ns" => {
                        material.tex_shininess = parse_path(&mut split_line)
                            .map(|path| matlib_dir.join(path));
                    },
                    "map_d" | "map_Tr" | "map_opacity" => {
                        material.tex_opacity = parse_path(&mut split_line)
                            .map(|path| matlib_dir.join(path));
                    },
                    "disp" => {
                        material.tex_disp = parse_path(&mut split_line)
                            .map(|path| matlib_dir.join(path));
                    },
                    "decal" => {
                        material.tex_decal = parse_path(&mut split_line)
                            .map(|path| matlib_dir.join(path));
                    },
                    "bump" | "map_Bump" | "map_bump" => {
                        material.tex_bump = parse_path(&mut split_line)
                            .map(|path| matlib_dir.join(path));
                    },
                    _ => {
                        println!("Unrecognised material key: {}", key);
                    }
                }
            }
        }
    }
    let material = try!(current_material.ok_or("Didn't find any material definitions!"));
    materials.insert(material.name.clone(), material);
    Ok(materials)
}
