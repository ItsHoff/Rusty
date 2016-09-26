use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::str::{Split, SplitWhitespace};
use std::vec::Vec;

#[derive(Debug)]
pub struct Polygon {
    pub indices: Vec<u32>,
    pub normal_i: Vec<u32>,
    pub tex_i: Vec<u32>
}

pub struct Group {
    pub name: String,
    pub poly_i: Vec<u32>
}

pub struct Object {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tex_coords: Vec<[f32; 2]>,
    pub polygons: Vec<Polygon>,
    pub groups: Vec<Group>
}

struct ParseState {
    mat_libs: Vec<String>,
    current_material: String,
    current_group: usize
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

fn parse_face(split_line: &mut SplitWhitespace, obj: &Object) -> Result<Polygon, Box<Error>> {
    let mut polygon = Polygon { indices: vec!(), tex_i: vec!(), normal_i: vec!() };
    for item in split_line {
        let indices = try!(parse_face_values(&mut item.split('/')));
        if let Some(index) = indices[0] {
            if index < 0 {
                let index = obj.positions.len() as i32 + index;
                polygon.indices.push(index as u32);
            } else {
                polygon.indices.push(index as u32 - 1);
            }
        }
        if let Some(index) = indices[1] {
            if index < 0 {
                let index = obj.positions.len() as i32 + index;
                polygon.tex_i.push(index as u32);
            } else {
                polygon.tex_i.push(index as u32 - 1);
            }
        }
        if let Some(index) = indices[2] {
            if index < 0 {
                let index = obj.positions.len() as i32 + index;
                polygon.normal_i.push(index as u32);
            } else {
                polygon.normal_i.push(index as u32 - 1);
            }
        }
    }
    // TODO: Make this an error instead of a panic
    assert!(polygon.tex_i.len() == polygon.indices.len() || polygon.tex_i.len() == 0);
    assert!(polygon.normal_i.len() == polygon.indices.len() || polygon.normal_i.len() == 0);
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
    let mut state = ParseState { mat_libs: vec!(),
                                 current_material: String::new(),
                                 current_group: 0
    };
    let mut obj = Object { positions: vec!(),
                           normals: vec!(),
                           tex_coords: vec!(),
                           polygons: vec!(),
                           groups: vec!()
    };
    let obj_file = try!(File::open(obj_path));
    let obj_reader = BufReader::new(obj_file);
    for line in obj_reader.lines() {
        let line = line.expect("Failed to unwrap line");
        let mut split_line = line.split_whitespace();
        match split_line.next() {
            Some(key) => match key {
                "f" => {
                    let polygon = try!(parse_face(&mut split_line, &obj));
                    obj.polygons.push(polygon);
                },
                "g" => state.current_group = obj.groups.len(),  // TODO: handle this properly
                "mtllib" => state.mat_libs.push(try!(parse_string(&mut split_line))),
                "usemtl" => state.current_material = try!(parse_string(&mut split_line)),
                "v" => obj.positions.push(try!(parse_float3(&mut split_line))),
                "vn" => obj.normals.push(try!(parse_float3(&mut split_line))),
                "vt" => obj.tex_coords.push(try!(parse_float2(&mut split_line))),
                _ => {}
            },
            None => {}
        }
    }
    for mat in state.mat_libs {
        println!("Material library: {}", mat);
    }
    Ok(obj)
}

pub fn print_obj(object: Object) {
    println!("Positions:");
    for v in object.positions {
        println!("{:?}", v);
    }
    println!("Normals:");
    for vn in object.normals {
        println!("{:?}", vn);
    }
    println!("Texture coordinates:");
    for vt in object.tex_coords {
        println!("{:?}", vt);
    }
    println!("Polygons");
    for p in object.polygons {
        println!("{:?}", p);
    }
}
