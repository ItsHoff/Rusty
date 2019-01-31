use cgmath::prelude::*;
use cgmath::{Point3, Vector3};

use crate::camera::PTCamera;
use crate::color::Color;
use crate::consts;
use crate::float::*;
use crate::intersect::{Interaction, Ray};
use crate::light::Light;
use crate::sample;
use crate::scene::Scene;

pub struct BDPath<'a> {
    camera_vertex: &'a CameraVertex<'a>,
    light_vertex: LightVertex<'a>,
    /// Intermediate path from the light to camera
    path: Vec<SurfaceVertex<'a>>,
}

impl<'a> BDPath<'a> {
    pub fn new(light_vertex: LightVertex<'a>, light_path: &[SurfaceVertex<'a>],
               camera_vertex: &'a CameraVertex, camera_path: &[SurfaceVertex<'a>],
    ) -> Self {
        let mut path = light_path.to_vec();
        for v in camera_path.iter().rev() {
            path.push(v.clone());
        }
        Self {
            camera_vertex,
            light_vertex,
            path,
        }
    }

    /// Get the s:th path vertex
    fn get_s(&self, s: usize) -> &dyn Vertex {
        assert_ne!(s, 0, "Can't get s == 0");
        if s == 1 {
            &self.light_vertex
        } else if s == self.path.len() + 2 {
            self.camera_vertex
        } else {
            self.get_surface_s(s)
        }
    }

    fn get_surface_s(&self, s: usize) -> &SurfaceVertex {
        &self.path[s - 2]
    }

    /// Get the s:th path vertex
    fn get_t(&self, t: usize) -> &dyn Vertex {
        assert_ne!(t, 0, "Can't get t == 0");
        if t == 1 {
            self.camera_vertex
        } else if t == self.path.len() + 2 {
            &self.light_vertex
        } else {
            self.get_surface_t(t)
        }
    }

    fn get_surface_t(&self, t: usize) -> &SurfaceVertex {
        &self.path[self.path.len() + 1 - t]
    }

    fn is_valid(&self, s: usize, t: usize) -> bool {
        assert_eq!(s + t - 2, self.path.len(), "Trying to validate path with wrong length!");
        // Is light side vertex valid connection
        (if s == 0 {
            !self.light_vertex.light.delta_pos()
        } else {
            !self.get_s(s).delta_dir()
        })
        &&
        // Is camera side vertex valid connection
        (if t == 0 {
            false
        } else {
            !self.get_t(t).delta_dir()
        })
    }

    /// Compute the pdf for sampling the path with s light vertices and t camera vertices.
    /// Return None if (s, t) is not valid sampling strategy.
    pub fn pdf(&self, s: usize, t: usize) -> Option<Float> {
        if !self.is_valid(s, t) {
            return None;
        }
        let mut pdf_light = 1.0;
        for i in 1..=s {
            if i == 1 {
                pdf_light *= self.light_vertex.pdf_pos;
            } else if i == 2 {
                pdf_light *= self.light_vertex.pdf_next(self.get_surface_s(i));
            } else {
                let sv = self.get_surface_s(i - 1);
                if !sv.delta_dir() {
                    pdf_light *= pdf_scatter(self.get_s(i - 2), sv, self.get_s(i));
                }
            }
        }
        let mut pdf_camera = 1.0;
        // Point light so no reason to evaluate t == 1
        for i in 2..=t {
            if i == 2 {
                pdf_camera *= self.camera_vertex.pdf_next(self.get_surface_t(i));
            } else {
                let sv = self.get_surface_t(i - 1);
                if !sv.delta_dir() {
                    pdf_camera *= pdf_scatter(self.get_t(i - 2), sv, self.get_t(i));
                }
            }
        }
        Some(pdf_light * pdf_camera)
    }
}

pub trait Vertex {
    fn pos(&self) -> Point3<Float>;

    fn cos_t(&self, dir: Vector3<Float>) -> Float;

    fn delta_dir(&self) -> bool;

    /// Evaluate the throughput for a path continuing in dir
    fn path_throughput(&self, dir: Vector3<Float>) -> Color;

    /// Connect vertex to a surface vertex.
    /// Return the shadow ray and total path throughput.
    /// Will panic if other is not a surface vertex.
    fn connect_to(&self, surface: &SurfaceVertex) -> (Ray, Color) {
        let ray = surface.isect.shadow_ray(self.pos());
        let beta = self.path_throughput(-ray.dir) * surface.path_throughput(ray.dir);
        let g = (self.cos_t(ray.dir) * surface.cos_t(ray.dir) / ray.length.powi(2)).abs();
        (ray, g * beta)
    }
}

fn dir_and_dist(from: &dyn Vertex, to: &dyn Vertex) -> (Vector3<Float>, Float) {
    let to_next = to.pos() - from.pos();
    let dist = to_next.magnitude();
    let dir = to_next / dist;
    (dir, dist)
}

/// Get the area pdf of scattering v1 -> v2 -> v3;
pub fn pdf_scatter(v1: &dyn Vertex, v2: &SurfaceVertex, v3: &dyn Vertex) -> Float {
    let (dir_prev, _) = dir_and_dist(v2, v1);
    let (dir_next, dist) = dir_and_dist(v2, v3);
    let pdf_dir = v2.isect.pdf(dir_prev, dir_next);
    sample::to_area_pdf(pdf_dir, dist.powi(2), v3.cos_t(dir_next).abs())
}

#[derive(Debug)]
pub struct CameraVertex<'a> {
    pub camera: &'a PTCamera,
    ray: Ray,
}

impl<'a> CameraVertex<'a> {
    pub fn new(camera: &'a PTCamera, ray: Ray) -> Self {
        Self { camera, ray }
    }

    pub fn sample_next(&self) -> (Color, Ray) {
        // This is the real value but it always equals to 1.0
        // let dir = self.ray.dir;
        // let beta = self.camera.we(dir) * self.camera.cos_t(dir).abs() / self.camera.pdf(dir);
        let beta = Color::white();
        (beta, self.ray.clone())
    }

    pub fn pdf_next(&self, next: &SurfaceVertex) -> Float {
        let (dir, dist) = dir_and_dist(self, next);
        let pdf_dir = self.camera.pdf_dir(dir);
        sample::to_area_pdf(pdf_dir, dist.powi(2), next.cos_t(dir).abs())
    }
}

impl Vertex for CameraVertex<'_> {
    fn pos(&self) -> Point3<Float> {
        self.camera.pos
    }

    fn cos_t(&self, dir: Vector3<Float>) -> Float {
        self.camera.cos_t(dir)
    }

    fn delta_dir(&self) -> bool {
        false
    }

    fn path_throughput(&self, dir: Vector3<Float>) -> Color {
        self.camera.we(dir)
    }
}

#[derive(Clone, Debug)]
pub struct LightVertex<'a> {
    light: &'a dyn Light,
    pos: Point3<Float>,
    pdf_pos: Float,
}

impl<'a> LightVertex<'a> {
    pub fn new(light: &'a dyn Light, pos: Point3<Float>, pdf_pos: Float) -> Self {
        Self { light, pos, pdf_pos }
    }

    pub fn sample_next(&self) -> (Color, Ray) {
        let (le, dir, dir_pdf) = self.light.sample_dir();
        let ray = Ray::from_dir(self.pos + consts::EPSILON * dir, dir);
        let beta = le * self.light.cos_t(ray.dir).abs() / (self.pdf_pos * dir_pdf);
        (beta, ray)
    }

    pub fn pdf_next(&self, next: &SurfaceVertex) -> Float {
        let (dir, dist) = dir_and_dist(self, next);
        let pdf_dir = self.light.pdf_dir(dir);
        sample::to_area_pdf(pdf_dir, dist.powi(2), next.cos_t(dir).abs())
    }
}

impl Vertex for LightVertex<'_> {
    fn pos(&self) -> Point3<Float> {
        self.pos
    }

    fn cos_t(&self, dir: Vector3<Float>) -> Float {
        self.light.cos_t(dir)
    }

    fn delta_dir(&self) -> bool {
        false
    }

    fn path_throughput(&self, dir: Vector3<Float>) -> Color {
        self.light.le(dir) / self.pdf_pos
    }
}

#[derive(Clone, Debug)]
pub struct SurfaceVertex<'a> {
    /// Ray that generated this vertex
    pub ray: Ray,
    /// Attenuation for radiance scattered from this vertex
    beta: Color,
    pub isect: Interaction<'a>,
}

impl<'a> SurfaceVertex<'a> {
    pub fn new(ray: Ray, beta: Color, isect: Interaction<'a>) -> Self {
        Self {
            ray,
            beta,
            isect,
        }
    }

    /// Radiance along the path ending at this vertex
    pub fn path_radiance(&self) -> Color {
        self.beta * self.isect.le(-self.ray.dir)
    }

    /// Attempt to convert the vertex to a light vertex
    pub fn to_light_vertex(&self, scene: &Scene) -> Option<LightVertex> {
        let tri = self.isect.tri;
        if tri.is_emissive() {
            let pdf_light = scene.pdf_light(tri);
            let pdf_pos = tri.pdf_pos();
            Some(LightVertex::new(tri, self.isect.p, pdf_light * pdf_pos))
        } else {
            None
        }
    }
}

impl Vertex for SurfaceVertex<'_> {
    fn pos(&self) -> Point3<Float> {
        self.isect.p
    }

    fn cos_t(&self, dir: Vector3<Float>) -> Float {
        self.isect.cos_t(dir)
    }

    fn delta_dir(&self) -> bool {
        self.isect.is_specular()
    }

    fn path_throughput(&self, dir: Vector3<Float>) -> Color {
        self.beta * self.isect.bsdf(-self.ray.dir, dir)
    }
}