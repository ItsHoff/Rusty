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

pub trait Intersect<'a, H> {
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
    pub orig: Point3<f32>,
    pub dir: Vector3<f32>,
    // For more efficient ray plane intersections
    pub reciprocal_dir: Vector3<f32>,
    pub length: f32,
}

impl Ray {
    fn new(orig: Point3<f32>, dir: Vector3<f32>, length: f32) -> Ray {
        let reciprocal_dir = 1.0 / dir;
        Ray { orig, dir, reciprocal_dir, length }
    }
}
