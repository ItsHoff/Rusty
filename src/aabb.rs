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

fn min_point(p1: Point3<f32>, p2: Point3<f32>) -> Point3<f32> {
    let mut p_min = Point3::origin();
    for i in 0..2 {
        p_min[i] = p1[i].min(p2[i]);
    }
    p_min
}

fn max_point(p1: Point3<f32>, p2: Point3<f32>) -> Point3<f32> {
    let mut p_max = Point3::origin();
    for i in 0..2 {
        p_max[i] = p1[i].max(p2[i]);
    }
    p_max
}

impl AABB {
    pub fn new(min: Point3<f32>, max: Point3<f32>) -> AABB {
        AABB { min, max }
    }

    /// Update the bounding box with new position
    pub fn update(&mut self, new_pos: Point3<f32>) {
        self.min = min_point(self.min, new_pos);
        self.max = max_point(self.max, new_pos);
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
}
