use cgmath::prelude::*;
use cgmath::{Point3, Vector3};

use crate::camera::PTCamera;
use crate::color::Color;
use crate::config::*;
use crate::consts;
use crate::float::*;
use crate::intersect::{Interaction, Ray};
use crate::light::Light;
use crate::pt_renderer::PathType;
use crate::sample;
use crate::scene::Scene;

pub struct BDPath<'a> {
    light_vertex: &'a LightVertex<'a>,
    light_path: &'a [SurfaceVertex<'a>],
    light_pdf_fwd: Vec<Option<Float>>,
    light_pdf_rev: Vec<Option<Float>>,
    camera_vertex: &'a CameraVertex<'a>,
    camera_path: &'a [SurfaceVertex<'a>],
    camera_pdf_fwd: Vec<Option<Float>>,
    camera_pdf_rev: Vec<Option<Float>>,
    config: &'a RenderConfig,
}

impl<'a> BDPath<'a> {
    pub fn new(
        light_vertex: &'a LightVertex<'a>,
        light_path: &'a [SurfaceVertex<'a>],
        camera_vertex: &'a CameraVertex,
        camera_path: &'a [SurfaceVertex<'a>],
        config: &'a RenderConfig,
    ) -> Self {
        // Precompute fwd and rev pdfs
        // None pdf corresponds to a delta distribution
        // TODO: handle delta distributions already in primitives and not just here
        // TODO: handle zeros in precomputed pdfs
        let mut light_pdf_fwd = Vec::new();
        let mut light_pdf_rev = Vec::new();
        for i in 0..=light_path.len() {
            if i == 0 {
                if light_vertex.light.delta_pos() {
                    light_pdf_fwd.push(None);
                } else {
                    light_pdf_fwd.push(Some(light_vertex.pdf_pos));
                }
            } else if i == 1 {
                if light_vertex.delta_dir() {
                    light_pdf_fwd.push(None);
                } else {
                    light_pdf_fwd.push(Some(light_vertex.pdf_next(&light_path[0])));
                }
            } else {
                let v_prev: &dyn Vertex = if i == 2 {
                    light_vertex
                } else {
                    &light_path[i - 3]
                };
                let v_mid = &light_path[i - 2];
                let v_next = &light_path[i - 1];
                // TODO: fwd and rev could be computed simultaneously
                let pdf_fwd = pdf_scatter(v_prev, v_mid, v_next);
                let pdf_rev = pdf_scatter(v_next, v_mid, v_prev);
                light_pdf_fwd.push(pdf_fwd);
                light_pdf_rev.push(pdf_rev);
            }
        }

        let mut camera_pdf_fwd = Vec::new();
        let mut camera_pdf_rev = Vec::new();
        for i in 0..=camera_path.len() {
            if i == 0 {
                // Pinhole camera
                camera_pdf_fwd.push(None);
            } else if i == 1 {
                if camera_vertex.delta_dir() {
                    camera_pdf_fwd.push(None);
                } else {
                    camera_pdf_fwd.push(Some(camera_vertex.pdf_next(&camera_path[0])));
                }
            } else {
                let v_prev: &dyn Vertex = if i == 2 {
                    camera_vertex
                } else {
                    &camera_path[i - 3]
                };
                let v_mid = &camera_path[i - 2];
                let v_next = &camera_path[i - 1];
                let pdf_fwd = pdf_scatter(v_prev, v_mid, v_next);
                let pdf_rev = pdf_scatter(v_next, v_mid, v_prev);
                camera_pdf_fwd.push(pdf_fwd);
                camera_pdf_rev.push(pdf_rev);
            }
        }

        Self {
            light_vertex,
            light_path,
            light_pdf_fwd,
            light_pdf_rev,
            camera_vertex,
            camera_path,
            camera_pdf_fwd,
            camera_pdf_rev,
            config,
        }
    }

    /// Get a subpath with s light vertices and t camera vertices
    /// Will panic if (s, t) is not a valid subpath
    pub fn subpath(&self, s: usize, t: usize) -> SubPath {
        let bounces = s + t - 2;
        assert!(
            bounces <= self.config.max_bounces,
            "Path contains {} bounces but it can't contain more than {} bounces!",
            bounces,
            self.config.max_bounces,
        );
        assert!(
            s <= self.light_path.len() + 1,
            "Cannot construct sub path with {} light vertices when there are only {}!",
            s,
            self.light_path.len() + 1,
        );
        assert!(
            t <= self.camera_path.len() + 1,
            "Cannot construct sub path with {} camera vertices when there are only {}!",
            t,
            self.camera_path.len() + 1,
        );
        assert!(
            s != 0 || self.camera_path[t - 2].isect.tri.is_emissive(),
            "Sub path ({}, {}) does not end at a emissive vertex!",
            s,
            t,
        );
        SubPath {
            path: self,
            s,
            t,
            tmp_light_vertex: None,
        }
    }

    /// Get a sub path with only camera vertices which ends at light_vertex
    pub fn subpath_with_light(&self, light_vertex: LightVertex<'a>, t: usize) -> SubPath {
        let mut subpath = self.subpath(0, t);
        subpath.tmp_light_vertex = Some(light_vertex);
        subpath
    }
}

pub struct SubPath<'a> {
    path: &'a BDPath<'a>,
    s: usize,
    t: usize,
    tmp_light_vertex: Option<LightVertex<'a>>,
}

impl SubPath<'_> {
    /// Compute the weight for the radiance that is transported along this path
    pub fn weight(&self) -> Float {
        let bounces = self.s + self.t - 2;
        if bounces == 0 {
            1.0
        } else if !self.path.config.mis {
            1.0 / (bounces + 2).to_float()
        } else {
            let power = 2; // for power heuristic
            let mut sum = 1.0;
            let mut light_ratio = 1.0;
            for si in (0..self.s).rev() {
                light_ratio *=
                    (self.camera_pdf(si + 1).unwrap_or(1.0) /
                    self.light_pdf(si + 1).unwrap_or(1.0)).powi(power);
                let delta_light = if si == 0 {
                    // No need to care about the tmp_light_vertex, since if it exists
                    // then self.s is always 0, and this branch is not evaluated.
                    self.path.light_vertex.light.delta_pos()
                } else {
                    self.get_vertex(si).delta_dir()
                };
                if !delta_light && !self.get_vertex(si + 1).delta_dir() {
                    sum += light_ratio;
                }
            }
            let mut camera_ratio = 1.0;
            for ti in (2..=self.t).rev() {
                let si = self.t_to_s(ti);
                camera_ratio *=
                    (self.light_pdf(si).unwrap_or(1.0) / self.camera_pdf(si).unwrap_or(1.0)).powi(power);
                if !self.get_vertex(si).delta_dir() && !self.get_vertex(si + 1).delta_dir() {
                    sum += camera_ratio;
                }
            }
            1.0 / sum
        }
    }

    /// Map camera side index t to light side index s
    fn t_to_s(&self, t: usize) -> usize {
        self.s + self.t - t + 1
    }

    /// Map light side index s to camera side index t
    fn s_to_t(&self, s: usize) -> usize {
        self.s + self.t - s + 1
    }

    /// Get the vertex s of the path
    fn get_vertex(&self, s: usize) -> &dyn Vertex {
        if s == 1 {
            if let Some(light) = &self.tmp_light_vertex {
                light
            } else {
                self.path.light_vertex
            }
        } else if self.s_to_t(s) == 1 {
            self.path.camera_vertex
        } else {
            self.get_surface(s)
        }
    }

    /// Get the s:th surface vertex on the path
    /// Will panic if the vertex does not exist
    fn get_surface(&self, s: usize) -> &SurfaceVertex {
        if s <= self.s {
            &self.path.light_path[s - 2]
        } else {
            &self.path.camera_path[self.s_to_t(s) - 2]
        }
    }

    /// Get the pdf of sampling vertex s from direction of the light
    fn light_pdf(&self, s: usize) -> Option<Float> {
        let mut pdf = if s <= self.s {
            self.path.light_pdf_fwd[s - 1]?
        } else {
            let t = self.s_to_t(s);
            // Connection vertex interpreted as light
            if self.s == 0 && t == self.t {
                let light = self.tmp_light_vertex.as_ref().unwrap();
                light.pdf_pos
            // Sampling emitted light from connection vertex
            } else if self.s == 0 && t == self.t - 1 {
                let light = self.tmp_light_vertex.as_ref().unwrap();
                light.pdf_next(self.get_surface(s))
            // Connection vertex sampled from the light
            } else if self.s == 1 && t == self.t {
                self.path.light_vertex.pdf_next(self.get_surface(s))
            // Scattering from the light direction for the connection vertices.
            } else if t >= self.t - 1 {
                let v1 = self.get_vertex(s - 2);
                let v2 = self.get_surface(s - 1);
                let v3 = self.get_vertex(s);
                if v2.delta_dir() {
                    return None;
                } else {
                    pdf_scatter(v1, v2, v3)?
                }
            // Backwards scattering along the light path
            } else {
                self.path.camera_pdf_rev[t - 1]?
            }
        };
        // Check if russian roulette bounce was needed to sample the vertex
        if let RussianRoulette::Static(rr_prob) = self.path.config.russian_roulette {
            if s > 2 && s - 2 > self.path.config.pre_rr_bounces {
                pdf *= rr_prob;
            }
        }
        Some(pdf)
    }

    /// Get the pdf of sampling vertex s from direction of the camera
    fn camera_pdf(&self, s: usize) -> Option<Float> {
        let t = self.s_to_t(s);
        let mut pdf = if s > self.s {
            // Regular sampling of the camera path
            self.path.camera_pdf_fwd[t - 1]?
        } else {
            // Connection vertex sampled from the camera
            if self.t == 1 && s == self.s {
                self.path.camera_vertex.pdf_next(self.get_surface(s))
            // Scattering from the camera direction for the connection vertices.
            } else if s >= self.s - 1 {
                let v1 = self.get_vertex(s + 2);
                let v2 = self.get_surface(s + 1);
                let v3 = self.get_vertex(s);
                if v2.delta_dir() {
                    return None;
                } else {
                    pdf_scatter(v1, v2, v3)?
                }
            // Backwards scattering along the light path
            } else {
                self.path.light_pdf_rev[s - 1]?
            }
        };
        // Check if russian roulette bounce was needed to sample the vertex
        if let RussianRoulette::Static(rr_prob) = self.path.config.russian_roulette {
            if t > 2 && t - 2 > self.path.config.pre_rr_bounces {
                pdf *= rr_prob;
            }
        }
        Some(pdf)
    }
}

pub trait Vertex: std::fmt::Debug {
    /// Get the position of the vertex
    fn pos(&self) -> Point3<Float>;

    /// Get the shadow ray origin for dir
    fn shadow_origin(&self, dir: Vector3<Float>) -> Point3<Float>;

    /// Geometric cosine
    fn cos_g(&self, dir: Vector3<Float>) -> Float;

    /// Shading cosine
    fn cos_s(&self, dir: Vector3<Float>) -> Float;

    fn delta_dir(&self) -> bool;

    /// Evaluate the throughput for a path continuing in dir
    fn path_throughput(&self, dir: Vector3<Float>) -> Color;

    /// Connect vertex to a surface vertex.
    /// Return the shadow ray and total path throughput.
    /// Will panic if other is not a surface vertex.
    fn connect_to(&self, other: &dyn Vertex) -> (Ray, Color) {
        let ray = Ray::shadow(self.shadow_origin(other.pos() - self.pos()), other.pos());
        let beta = self.path_throughput(ray.dir) * other.path_throughput(-ray.dir);
        let g = (self.cos_s(ray.dir) * other.cos_s(ray.dir) / ray.length.powi(2)).abs();
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
/// Return None if pdf is a delta distribution
pub fn pdf_scatter(v1: &dyn Vertex, v2: &SurfaceVertex, v3: &dyn Vertex) -> Option<Float> {
    if v2.delta_dir() {
        return None;
    }
    let (dir_prev, _) = dir_and_dist(v2, v1);
    let (dir_next, dist) = dir_and_dist(v2, v3);
    let pdf_dir = v2.isect.pdf(dir_prev, dir_next);
    Some(sample::to_area_pdf(pdf_dir, dist.powi(2), v3.cos_g(dir_next).abs()))
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
        // let beta = self.camera.we(dir) * self.camera.cos_s(dir).abs() / self.camera.pdf(dir);
        let beta = Color::white();
        (beta, self.ray.clone())
    }

    pub fn pdf_next(&self, next: &SurfaceVertex) -> Float {
        let (dir, dist) = dir_and_dist(self, next);
        let pdf_dir = self.camera.pdf_dir(dir);
        sample::to_area_pdf(pdf_dir, dist.powi(2), next.cos_g(dir).abs())
    }
}

impl Vertex for CameraVertex<'_> {
    fn pos(&self) -> Point3<Float> {
        self.camera.pos
    }

    fn shadow_origin(&self, _dir: Vector3<Float>) -> Point3<Float> {
        // There is no physical camera so no need to care for self shadowing
        self.camera.pos
    }

    fn cos_g(&self, dir: Vector3<Float>) -> Float {
        self.camera.cos_g(dir)
    }

    fn cos_s(&self, dir: Vector3<Float>) -> Float {
        self.cos_g(dir)
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
        Self {
            light,
            pos,
            pdf_pos,
        }
    }

    pub fn sample_next(&self) -> (Color, Ray) {
        let (le, dir, dir_pdf) = self.light.sample_dir();
        let ray = Ray::from_dir(self.pos + consts::EPSILON * dir, dir);
        let beta = le * self.cos_s(ray.dir).abs() / (self.pdf_pos * dir_pdf);
        (beta, ray)
    }

    pub fn pdf_next(&self, next: &SurfaceVertex) -> Float {
        let (dir, dist) = dir_and_dist(self, next);
        let pdf_dir = self.light.pdf_dir(dir);
        sample::to_area_pdf(pdf_dir, dist.powi(2), next.cos_g(dir).abs())
    }
}

impl Vertex for LightVertex<'_> {
    fn pos(&self) -> Point3<Float> {
        self.pos
    }

    fn shadow_origin(&self, _dir: Vector3<Float>) -> Point3<Float> {
        panic!("Shadow rays starting from lights not implemented!");
    }

    fn cos_g(&self, dir: Vector3<Float>) -> Float {
        self.light.cos_g(dir)
    }

    fn cos_s(&self, dir: Vector3<Float>) -> Float {
        self.cos_g(dir)
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
    path_type: PathType,
    pub isect: Interaction<'a>,
}

impl<'a> SurfaceVertex<'a> {
    pub fn new(ray: Ray, beta: Color, path_type: PathType, isect: Interaction<'a>) -> Self {
        Self {
            ray,
            beta,
            isect,
            path_type,
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

    fn shadow_origin(&self, dir: Vector3<Float>) -> Point3<Float> {
        self.isect.ray_origin(dir)
    }

    fn cos_g(&self, dir: Vector3<Float>) -> Float {
        self.isect.cos_g(dir)
    }

    fn cos_s(&self, dir: Vector3<Float>) -> Float {
        self.isect.cos_s(dir)
    }

    fn delta_dir(&self) -> bool {
        self.isect.is_specular()
    }

    fn path_throughput(&self, dir: Vector3<Float>) -> Color {
        self.beta * self.isect.bsdf(-self.ray.dir, dir, self.path_type)
    }
}
