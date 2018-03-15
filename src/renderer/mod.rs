#![cfg_attr(feature="clippy", allow(forget_copy))]

mod gl_renderer;
mod pt_renderer;
mod triangle;
mod vertex;

use cgmath::{Vector3, Point3};

pub use self::gl_renderer::GLRenderer;
pub use self::pt_renderer::PTRenderer;
pub use self::triangle::{RTTriangle, RTTriangleBuilder};
pub use self::vertex::{Vertex, CGVertex};

pub trait Intersectable<'a, H> {
    fn intersect(&'a self, ray: &Ray) -> Option<H>;
}

#[derive(Debug)]
pub struct Hit<'a> {
    tri: &'a RTTriangle,
    t: f32,
    u: f32,
    v: f32,
}

#[derive(Clone, Copy)]
pub struct Ray {
    orig: Point3<f32>,
    dir: Vector3<f32>,
    length: f32,
}
