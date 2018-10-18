use cgmath::prelude::*;
use cgmath::Point3;

use crate::pt_renderer::{Intersect, Ray};
use crate::Float;

#[derive(Clone, Debug)]
pub struct AABB {
    pub min: Point3<Float>,
    pub max: Point3<Float>,
}

impl AABB {
    pub fn empty() -> AABB {
        AABB {
            min: Point3::max_value(),
            max: Point3::min_value(),
        }
    }

    /// Update the bounding box to enclose other aswell
    pub fn add_aabb(&mut self, other: &AABB) {
        self.min = min_point(&self.min, &other.min);
        self.max = max_point(&self.max, &other.max);
    }

    /// Get the center of the scene as defined by the bounding box
    pub fn center(&self) -> Point3<Float> {
        if self.max.x < self.min.x {
            panic!("Tried to get center of an empty AABB");
        }
        Point3::midpoint(self.min, self.max)
    }

    pub fn longest_edge(&self) -> Float {
        let mut longest = 0.0 as Float;
        for i in 0..3 {
            longest = longest.max(self.max[i] - self.min[i]);
        }
        longest
    }

    pub fn longest_edge_i(&self) -> usize {
        let mut longest = 0.0;
        let mut index = 0;
        for i in 0..3 {
            let length = self.max[i] - self.min[i];
            if length > longest {
                longest = length;
                index = i;
            }
        }
        index
    }

    pub fn area(&self) -> Float {
        let lengths = self.max - self.min;
        2.0 * (lengths.x * lengths.y + lengths.y * lengths.z + lengths.z * lengths.x).max(0.0)
    }
}

impl Intersect<'_, Float> for AABB {
    fn intersect(&self, ray: &Ray) -> Option<Float> {
        let t1 = (self.min - ray.orig).mul_element_wise(ray.reciprocal_dir);
        let t2 = (self.max - ray.orig).mul_element_wise(ray.reciprocal_dir);
        let mut start = std::f64::MIN as Float;
        let mut end = std::f64::MAX as Float;
        for i in 0..3 {
            if ray.dir[i] == 0.0 && (ray.orig[i] < self.min[i] || ray.orig[i] > self.max[i]) {
                // Can't hit
                return None;
            } else if ray.neg_dir[i] {
                start = start.max(t2[i]);
                end = end.min(t1[i]);
            } else {
                start = start.max(t1[i]);
                end = end.min(t2[i]);
            }
        }
        if start <= end && end > 0.0 && start < ray.length {
            Some(start)
        } else {
            None
        }
    }
}

pub fn min_point(p1: &Point3<Float>, p2: &Point3<Float>) -> Point3<Float> {
    let mut p_min = Point3::max_value();
    for i in 0..3 {
        p_min[i] = p1[i].min(p2[i]);
    }
    p_min
}

pub fn max_point(p1: &Point3<Float>, p2: &Point3<Float>) -> Point3<Float> {
    let mut p_max = Point3::min_value();
    for i in 0..3 {
        p_max[i] = p1[i].max(p2[i]);
    }
    p_max
}
