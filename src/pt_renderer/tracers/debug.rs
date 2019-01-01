use cgmath::prelude::*;

use crate::bvh::BVHNode;
use crate::color::Color;
use crate::config::*;
use crate::float::*;
use crate::intersect::Ray;
use crate::scene::Scene;

pub fn debug_trace<'a>(
    ray: Ray,
    mode: &DebugMode,
    scene: &'a Scene,
    config: &RenderConfig,
    node_stack: &mut Vec<(&'a BVHNode, Float)>,
) -> Color {
    match mode {
        DebugMode::Normals => trace_normals(ray, scene, config, node_stack, false),
        DebugMode::ForwardNormals => trace_normals(ray, scene, config, node_stack, true),
    }
}

fn trace_normals<'a>(
    mut ray: Ray,
    scene: &'a Scene,
    config: &RenderConfig,
    node_stack: &mut Vec<(&'a BVHNode, Float)>,
    forward_only: bool,
) -> Color {
    let mut c = Color::black();
    if let Some(hit) = scene.intersect(&mut ray, node_stack) {
        let isect = hit.interaction(config);
        if !forward_only || isect.ns.dot(ray.dir) > 0.0 {
            c = Color::from_normal(isect.ns);
        }
    }
    c
}
