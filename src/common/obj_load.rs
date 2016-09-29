use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::str::{Split, SplitWhitespace};
use std::vec::Vec;

#[derive(Debug, Default)]
pub struct Polygon {
    pub positions_i: Vec<u32>,
    pub normals_i: Vec<u32>,
    pub tex_coords_i: Vec<u32>,
    pub group: Option<String>,
    pub smoothing_group: Option<u32>,
    pub material: Option<String>,
}

impl Polygon {
    fn new(state: &ParseState) -> Polygon {
        Polygon {
            group: state.current_group.clone(),
            smoothing_group: state.current_smoothing_group.clone(),
            material: state.current_material.clone(),
            ..Default::default()
        }
    }
}

pub struct Group {
    pub name: String,
}

#[derive(Debug, Default)]
#[allow(non_snake_case)]
pub struct Material {
    pub name: String,
    pub Ka: Option<[f32; 3]>,
    pub Kd: Option<[f32; 3]>,
    pub Ks: Option<[f32; 3]>,
    pub Tf: Option<[f32; 3]>,
    pub Ke: Option<[f32; 3]>,
    pub illum: Option<u32>,
    pub d: Option<f32>,
    pub Ns: Option<f32>,
    pub sharpness: Option<f32>,
    pub Ni: Option<f32>,
    pub map_Ka: Option<PathBuf>,
    pub map_Kd: Option<PathBuf>,
    pub map_Ks: Option<PathBuf>,
    pub map_Ns: Option<PathBuf>,
    pub map_d: Option<PathBuf>,
    pub disp: Option<PathBuf>,
    pub decal: Option<PathBuf>,
    pub bump: Option<PathBuf>,
}

impl Material {
    fn new(name: &str) -> Material {
        Material { name: name.to_string(),
                 ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct Object {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tex_coords: Vec<[f32; 2]>,
    pub polygons: Vec<Polygon>,
    pub groups: Vec<Group>,
    pub materials: Vec<Material>
}

impl Object {
    fn new() -> Object {
        Object { ..Default::default() }
    }
}

#[derive(Default)]
struct ParseState {
    mat_libs: Vec<PathBuf>,
    current_group: Option<String>,
    current_smoothing_group: Option<u32>,
    current_material: Option<String>,
}

impl ParseState {
    fn new() -> ParseState {
        ParseState { ..Default::default() }
    }
}

fn parse_int(split_line: &mut SplitWhitespace) -> Result<u32, Box<Error>> {
    let item = try!(split_line.next().ok_or("Expected value after key"));
    Ok(try!(item.parse()))
}

fn parse_float(split_line: &mut SplitWhitespace) -> Result<f32, Box<Error>> {
    let item = try!(split_line.next().ok_or("Expected value after key"));
    Ok(try!(item.parse()))
}

fn parse_float2(split_line: &mut SplitWhitespace) -> Result<[f32; 2], Box<Error>> {
    let mut float2 = [0.0f32; 2];
    for i in 0..2 {
        let item = try!(split_line.next().ok_or("Float 2 didn't have 2 floats"));
        float2[i] = try!(item.parse());
    }
    Ok(float2)
}

fn parse_float3(split_line: &mut SplitWhitespace) -> Result<[f32; 3], Box<Error>> {
    let mut float3 = [0.0f32; 3];
    for i in 0..3 {
        let item = try!(split_line.next().ok_or("Float 3 didn't have 3 floats"));
        float3[i] = try!(item.parse());
    }
    Ok(float3)
}

fn parse_string(split_line: &mut SplitWhitespace) -> Result<String, Box<Error>> {
    let string = try!(split_line.next().ok_or("Couldnt not find string."));
    Ok(string.to_string())
}

fn parse_face(split_line: &mut SplitWhitespace, obj: &Object, state: &ParseState)
              -> Result<Polygon, Box<Error>> {
    let mut polygon = Polygon::new(state);
    for item in split_line {
        let index_values = try!(parse_face_values(&mut item.split('/')));
        if let Some(pos_i) = index_values[0] {
            if pos_i < 0 {
                let pos_i = obj.positions.len() as i32 + pos_i;
                polygon.positions_i.push(pos_i as u32);
            } else {
                polygon.positions_i.push(pos_i as u32 - 1);
            }
        }
        if let Some(tex_i) = index_values[1] {
            if tex_i < 0 {
                let tex_i = obj.positions.len() as i32 + tex_i;
                polygon.tex_coords_i.push(tex_i as u32);
            } else {
                polygon.tex_coords_i.push(tex_i as u32 - 1);
            }
        }
        if let Some(norm_i) = index_values[2] {
            if norm_i < 0 {
                let norm_i = obj.positions.len() as i32 + norm_i;
                polygon.normals_i.push(norm_i as u32);
            } else {
                polygon.normals_i.push(norm_i as u32 - 1);
            }
        }
    }
    // TODO: Sanity check
    Ok(polygon)
}

fn parse_face_values(split_face: &mut Split<char>) -> Result<[Option<i32>; 3], Box<Error>> {
    let mut result = [None; 3];
    for (i, num) in split_face.enumerate() {
        if i >= 3 {
            break;
        }
        if num == "" {
            result[i] = None;
        } else {
            result[i] = Some(try!(num.parse()));
        }
    }
    Ok(result)
}

pub fn load_obj(obj_path: &Path) -> Result<Object, Box<Error>> {
    let mut state = ParseState::new();
    let mut obj = Object::new();
    let obj_dir = try!(obj_path.parent().ok_or("Couldn't get object directory"));
    let obj_file = try!(File::open(obj_path));
    let obj_reader = BufReader::new(obj_file);
    for line in obj_reader.lines() {
        let line = line.expect("Failed to unwrap line");
        let mut split_line = line.split_whitespace();
        match split_line.next() {
            Some(key) => match key {
                "f" => {
                    let polygon = try!(parse_face(&mut split_line, &obj, &state));
                    obj.polygons.push(polygon);
                },
                "g" | "o" => {
                    let group_name = try!(parse_string(&mut split_line));
                    obj.groups.push(Group { name: group_name.clone() });
                    state.current_group = Some(group_name);
                },
                "mtllib" => state.mat_libs.push(obj_dir.join(try!(parse_string(&mut split_line)))),
                "s" => {
                    let val = try!(parse_string(&mut split_line));
                    if val == "off" || val == "0" {
                        state.current_smoothing_group = None;
                    } else {
                        state.current_smoothing_group = Some(try!(val.parse()));
                    }
                }
                "usemtl" => state.current_material = Some(try!(parse_string(&mut split_line))),
                "v" => obj.positions.push(try!(parse_float3(&mut split_line))),
                "vn" => obj.normals.push(try!(parse_float3(&mut split_line))),
                "vt" => obj.tex_coords.push(try!(parse_float2(&mut split_line))),
                _ => {
                    if !key.starts_with("#") {
                        println!("Unrecognised key {}", key);
                    }
                }
            },
            None => {}
        }
    }
    for matlib in state.mat_libs {
        obj.materials = try!(load_matlib(&obj_dir.join(matlib)));
    }
    Ok(obj)
}

pub fn load_matlib(matlib_path: &Path) ->  Result<Vec<Material>, Box<Error>> {
    let mut materials = Vec::<Material>::new();
    let matlib_dir = try!(matlib_path.parent().ok_or("Couldn't get material directory"));
    let matlib_file = try!(File::open(matlib_path));
    let matlib_reader = BufReader::new(matlib_file);
    for line in matlib_reader.lines() {
        let line = line.expect("Failed to unwrap line");
        let mut split_line = line.split_whitespace();
        match split_line.next() {
            Some(key) => match key {
                "newmtl" => materials.push(Material::new(&try!(parse_string(&mut split_line)))),
                "Ka" => {
                    let material = try!(materials.last_mut().ok_or("Found material properties before newmtl!"));
                    material.Ka = Some(try!(parse_float3(&mut split_line)));
                },
                "Kd" => {
                    let material = try!(materials.last_mut().ok_or("Found material properties before newmtl!"));
                    material.Kd = Some(try!(parse_float3(&mut split_line)));
                },
                "Ks" => {
                    let material = try!(materials.last_mut().ok_or("Found material properties before newmtl!"));
                    material.Ks = Some(try!(parse_float3(&mut split_line)));
                },
                "Tf" => {
                    let material = try!(materials.last_mut().ok_or("Found material properties before newmtl!"));
                    material.Tf = Some(try!(parse_float3(&mut split_line)));
                },
                "Ke" => {
                    let material = try!(materials.last_mut().ok_or("Found material properties before newmtl!"));
                    material.Ke = Some(try!(parse_float3(&mut split_line)));
                },
                "illum" => {
                    let material = try!(materials.last_mut().ok_or("Found material properties before newmtl!"));
                    material.illum = Some(try!(parse_int(&mut split_line)));
                },
                "d" => {
                    let material = try!(materials.last_mut().ok_or("Found material properties before newmtl!"));
                    material.d = Some(try!(parse_float(&mut split_line)));
                },
                "Ns" => {
                    let material = try!(materials.last_mut().ok_or("Found material properties before newmtl!"));
                    material.Ns = Some(try!(parse_float(&mut split_line)));
                },
                "sharpness" => {
                    let material = try!(materials.last_mut().ok_or("Found material properties before newmtl!"));
                    material.sharpness = Some(try!(parse_float(&mut split_line)));
                },
                "Ni" => {
                    let material = try!(materials.last_mut().ok_or("Found material properties before newmtl!"));
                    material.Ni = Some(try!(parse_float(&mut split_line)));
                },
                "map_Ka" => {
                    let material = try!(materials.last_mut().ok_or("Found material properties before newmtl!"));
                    material.map_Ka = Some(matlib_dir.join(try!(parse_string(&mut split_line))));
                },
                "map_Kd" => {
                    let material = try!(materials.last_mut().ok_or("Found material properties before newmtl!"));
                    material.map_Kd = Some(matlib_dir.join(try!(parse_string(&mut split_line))));
                },
                "map_Ks" => {
                    let material = try!(materials.last_mut().ok_or("Found material properties before newmtl!"));
                    material.map_Ks = Some(matlib_dir.join(try!(parse_string(&mut split_line))));
                },
                "map_Ns" => {
                    let material = try!(materials.last_mut().ok_or("Found material properties before newmtl!"));
                    material.map_Ns = Some(matlib_dir.join(try!(parse_string(&mut split_line))));
                },
                "map_d" => {
                    let material = try!(materials.last_mut().ok_or("Found material properties before newmtl!"));
                    material.map_d = Some(matlib_dir.join(try!(parse_string(&mut split_line))));
                },
                "disp" => {
                    let material = try!(materials.last_mut().ok_or("Found material properties before newmtl!"));
                    material.disp = Some(matlib_dir.join(try!(parse_string(&mut split_line))));
                },
                "decal" => {
                    let material = try!(materials.last_mut().ok_or("Found material properties before newmtl!"));
                    material.decal = Some(matlib_dir.join(try!(parse_string(&mut split_line))));
                },
                "bump" | "map_Bump" | "map_bump" => {
                    let material = try!(materials.last_mut().ok_or("Found material properties before newmtl!"));
                    material.bump = Some(matlib_dir.join(try!(parse_string(&mut split_line))));
                },
                _ => {
                    if !key.starts_with("#") {
                        println!("Unrecognised material key {}", key);
                    }
                }
            },
            None => {}
        }
    }
    Ok(materials)
}

#[allow(dead_code)]
pub fn print_obj(object: &Object) {
    println!("Polygons");
    for p in &object.polygons {
        println!("{:?}", p);
    }
    println!("Materials");
    for m in &object.materials {
        println!("{:?}", m);
    }
    //println!("Positions:");
    //for v in &object.positions {
        //println!("{:?}", v);
    //}
    //println!("Normals:");
    //for vn in &object.normals {
        //println!("{:?}", vn);
    //}
    //println!("Texture coordinates:");
    //for vt in &object.tex_coords {
        //println!("{:?}", vt);
    //}
}
