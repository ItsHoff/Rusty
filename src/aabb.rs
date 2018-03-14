use cgmath::prelude::*;
use cgmath::Point3;

pub struct AABB {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

impl Default for AABB {
    fn default() -> AABB {
        AABB {
            min: Point3::origin(),
            max: Point3::origin(),
        }
    }
}

pub fn min_point(p1: &Point3<f32>, p2: &Point3<f32>) -> Point3<f32> {
    let mut p_min = Point3::origin();
    for i in 0..2 {
        p_min[i] = p1[i].min(p2[i]);
    }
    p_min
}

pub fn max_point(p1: &Point3<f32>, p2: &Point3<f32>) -> Point3<f32> {
    let mut p_max = Point3::origin();
    for i in 0..2 {
        p_max[i] = p1[i].max(p2[i]);
    }
    p_max
}

impl AABB {
    /// Update the bounding box with new position
    pub fn add_point(&mut self, new_pos: &Point3<f32>) {
        self.min = min_point(&self.min, new_pos);
        self.max = max_point(&self.max, new_pos);
    }

    /// Update the bounding box to enclose other aswell
    pub fn add_aabb(&mut self, other: &AABB) {
        self.min = min_point(&self.min, &other.min);
        self.max = max_point(&self.max, &other.max);
    }

    /// Get the center of the scene as defined by the bounding box
    pub fn center(&self) -> Point3<f32> {
        Point3::midpoint(self.min, self.max)
    }

    pub fn longest_edge(&self) -> f32 {
        let mut longest = 0.0f32;
        for i in 0..2 {
            longest = longest.max(self.max[i] - self.min[i]);
        }
        longest
    }

    pub fn longest_edge_i(&self) -> usize {
        let mut longest = 0.0f32;
        let mut index = 0;
        for i in 0..2 {
            let length = self.max[i] - self.min[i];
            if length > longest {
                longest = length;
                index = i;
            }
        }
        index
    }
}
