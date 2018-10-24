use cgmath::prelude::*;
use cgmath::{Point2, Point3, Vector3};

use crate::color::Color;
use crate::consts;
use crate::material::Material;
use crate::triangle::Triangle;
use crate::Float;

pub trait Intersect<'a, H> {
    fn intersect(&'a self, ray: &Ray) -> Option<H>;
}

#[derive(Clone, Debug)]
pub struct Ray {
    pub orig: Point3<Float>,
    pub dir: Vector3<Float>,
    pub length: Float,
    // For more efficient ray box intersections
    pub reciprocal_dir: Vector3<Float>,
    pub neg_dir: [bool; 3],
}

impl Ray {
    fn new(orig: Point3<Float>, dir: Vector3<Float>, length: Float) -> Ray {
        let reciprocal_dir = 1.0 / dir;
        let neg_dir = [dir.x < 0.0, dir.y < 0.0, dir.z < 0.0];
        Ray {
            orig,
            dir,
            length,
            reciprocal_dir,
            neg_dir,
        }
    }

    /// Infinite ray with a given direction and origin
    pub fn from_dir(mut orig: Point3<Float>, dir: Vector3<Float>) -> Ray {
        orig += consts::EPSILON * dir;
        Ray::new(orig, dir, consts::INFINITY)
    }

    /// Infinite ray from origin towards another point
    pub fn from_point(mut orig: Point3<Float>, to: Point3<Float>) -> Ray {
        let dir = (to - orig).normalize();
        orig += consts::EPSILON * dir;
        Ray::new(orig, dir, consts::INFINITY)
    }

    /// Shadow ray between two points
    pub fn shadow(mut orig: Point3<Float>, to: Point3<Float>) -> Ray {
        let dp = to - orig;
        let length = dp.magnitude() - 2.0 * consts::EPSILON;
        let dir = dp.normalize();
        orig += consts::EPSILON * dir;
        Ray::new(orig, dir, length)
    }
}

pub struct Interaction<'a> {
    pub tri: &'a Triangle,
    pub p: Point3<Float>,
    pub n: Vector3<Float>,
    pub t: Point2<Float>,
    pub mat: &'a Material,
}

impl Interaction<'_> {
    pub fn le(&self, dir: Vector3<Float>) -> Color {
        self.tri.le(dir)
    }
}
