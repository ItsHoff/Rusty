use cgmath::prelude::*;
use cgmath::{Point3, Vector3};

use crate::camera::PTCamera;
use crate::color::Color;
use crate::consts;
use crate::float::*;
use crate::intersect::{Interaction, Ray};
use crate::light::Light;
use crate::sample;
use crate::triangle::Triangle;

#[derive(Debug)]
pub enum Vertex<'a> {
    Camera(CameraVertex<'a>),
    Light(LightVertex<'a>),
    Surface(Box<SurfaceVertex<'a>>),
}

impl<'a> Vertex<'a> {
    pub fn camera(camera: &'a PTCamera, ray: Ray) -> Self {
        Vertex::Camera(CameraVertex::new(camera, ray))
    }

    pub fn light(light: &'a dyn Light, pos: Point3<Float>, pdf: Float) -> Self {
        Vertex::Light(LightVertex::new(light, pos, pdf))
    }

    pub fn surface(ray: Ray, beta: Color, isect: Interaction<'a>, pdf: Float) -> Self {
        Vertex::Surface(Box::new(SurfaceVertex::new(ray, beta, isect, pdf)))
    }

    pub fn get_surface(&self) -> Option<&SurfaceVertex> {
        match self {
            Vertex::Camera(_) => None,
            Vertex::Light(_) => None,
            Vertex::Surface(vertex) => Some(vertex),
        }
    }

    /// Convert pdf to area measure
    pub fn convert_pdf(&mut self, prev: &Self) {
        match self {
            Vertex::Camera(_) => (),
            Vertex::Light(_) => (),
            Vertex::Surface(vertex) => {
                let to_prev = prev.pos() - vertex.isect.p;
                let length = to_prev.magnitude();
                let dir = to_prev / length;
                let pdf_a =
                    sample::to_area_pdf(vertex.pdf, length.powi(2), vertex.isect.cos_t(dir).abs());
                vertex.pdf = pdf_a;
            }
        };
    }

    pub fn cos_t(&self, dir: Vector3<Float>) -> Float {
        match self {
            Vertex::Camera(vertex) => vertex.camera.cos_t(dir),
            Vertex::Light(vertex) => vertex.light.cos_t(dir),
            Vertex::Surface(vertex) => vertex.isect.cos_t(dir),
        }
    }

    /// Throughput of a path continuing in dir
    fn path_throughput(&self, dir: Vector3<Float>) -> Color {
        match self {
            Vertex::Camera(vertex) => vertex.camera.we(dir),
            Vertex::Light(vertex) => vertex.light.le(dir) / vertex.pdf,
            Vertex::Surface(vertex) => vertex.beta * vertex.isect.bsdf(-vertex.ray.dir, dir),
        }
    }

    fn pos(&self) -> Point3<Float> {
        match self {
            Vertex::Camera(vertex) => vertex.camera.pos,
            Vertex::Light(vertex) => vertex.pos,
            Vertex::Surface(vertex) => vertex.isect.p,
        }
    }

    pub fn is_delta(&self) -> bool {
        match self {
            Vertex::Camera(_) => true,
            Vertex::Light(vertex) => vertex.light.is_delta(),
            Vertex::Surface(vertex) => vertex.isect.is_specular(),
        }
    }

    /// Connect this vertex to surface vertex other.
    /// Return the shadow ray and total path throughput.
    /// Will panic if other is not a surface vertex.
    pub fn connect_to_surface(&self, other: &Self) -> (Ray, Color) {
        let surface = other.get_surface().unwrap();
        let pos = self.pos();
        let ray = surface.isect.shadow_ray(pos);
        let beta = self.path_throughput(-ray.dir) * other.path_throughput(ray.dir);
        let g = (self.cos_t(ray.dir) * other.cos_t(ray.dir) / ray.length.powi(2)).abs();
        (ray, g * beta)
    }

    /// Sample outgoing ray from the vertex
    pub fn sample_next(&self) -> Option<(Color, Ray, Float)> {
        match self {
            Vertex::Camera(vertex) => {
                let dir = vertex.ray.dir;
                let beta =
                    vertex.camera.we(dir) * vertex.camera.cos_t(dir).abs() / vertex.camera.pdf(dir);
                Some((beta, vertex.ray.clone(), vertex.camera.pdf(dir)))
            }
            Vertex::Light(vertex) => {
                let (le, dir, dir_pdf) = vertex.light.sample_dir();
                let ray = Ray::from_dir(vertex.pos + consts::EPSILON * dir, dir);
                let beta = le * vertex.light.cos_t(ray.dir).abs() / (vertex.pdf * dir_pdf);
                Some((beta, ray, dir_pdf))
            }
            Vertex::Surface(vertex) => vertex.isect.sample_bsdf(-vertex.ray.dir),
        }
    }

    pub fn pdf_fwd(&self) -> Float {
        match self {
            Vertex::Camera(_) => 1.0,
            Vertex::Light(vertex) => vertex.pdf,
            Vertex::Surface(vertex) => vertex.pdf,
        }
    }

    /// Get the area pdf of sampling v3 from v2 when v1 is the previous vertex.
    /// v2 must be surface vertex or otherwise the function will panic.
    pub fn pdf(v1: &Self, v2: &Self, v3: &Self) -> Float {
        let dir_prev = (v1.pos() - v2.pos()).normalize();
        let to_next = v3.pos() - v2.pos();
        let length = to_next.magnitude();
        let dir_next = to_next / length;
        let surface = v2.get_surface().unwrap();
        let pdf_dir = surface.isect.pdf(dir_prev, dir_next);
        sample::to_area_pdf(pdf_dir, length.powi(2), v3.cos_t(dir_next).abs())
    }
}

#[derive(Debug)]
pub struct CameraVertex<'a> {
    camera: &'a PTCamera,
    ray: Ray,
}

impl<'a> CameraVertex<'a> {
    fn new(camera: &'a PTCamera, ray: Ray) -> Self {
        Self { camera, ray }
    }
}

#[derive(Debug)]
pub struct LightVertex<'a> {
    light: &'a dyn Light,
    pos: Point3<Float>,
    pdf: Float,
}

impl<'a> LightVertex<'a> {
    fn new(light: &'a dyn Light, pos: Point3<Float>, pdf: Float) -> Self {
        Self { light, pos, pdf }
    }
}

#[derive(Debug)]
pub struct SurfaceVertex<'a> {
    /// Ray that generated this vertex
    pub ray: Ray,
    /// Attenuation for radiance scattered from this vertex
    beta: Color,
    isect: Interaction<'a>,
    pdf: Float,
}

impl<'a> SurfaceVertex<'a> {
    fn new(ray: Ray, beta: Color, isect: Interaction<'a>, pdf: Float) -> Self {
        Self {
            ray,
            beta,
            isect,
            pdf,
        }
    }

    /// Radiance along the path ending at this vertex
    pub fn path_radiance(&self) -> Color {
        self.beta * self.isect.le(-self.ray.dir)
    }

    pub fn get_light(&self) -> Option<&Triangle> {
        let tri = self.isect.tri;
        if tri.is_emissive() {
            Some(tri)
        } else {
            None
        }
    }
}
