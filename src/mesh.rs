use std::{ops::Range, sync::Arc};

use bevy_math::{Ray3d, Vec3};

use crate::material::Material;

pub trait Mesh {
    fn hit(&self, ray: &Ray3d, ray_t: Range<f32>) -> Option<Hit>;
}

pub struct Hit {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub front_face: bool,
    pub material: Arc<dyn Material>,
}

impl Hit {
    pub fn new(ray: &Ray3d, distance: f32, normal: Vec3, material: Arc<dyn Material>) -> Self {
        let front_face = ray.direction.dot(normal) < 0.;
        let point = ray.get_point(distance);

        Self {
            point,
            normal: if front_face { normal } else { -normal },
            distance,
            front_face: true,
            material,
        }
    }
}

pub struct World(Vec<Box<dyn Mesh + Sync + Send>>);

impl World {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, mesh: impl Mesh + Sync + Send + 'static) {
        self.0.push(Box::new(mesh))
    }
}

impl Mesh for World {
    fn hit(&self, ray: &Ray3d, ray_t: Range<f32>) -> Option<Hit> {
        let mut current_hit: Option<Hit> = None;

        for mesh in &self.0 {
            let tmax = current_hit
                .as_ref()
                .map(|hit| hit.distance)
                .unwrap_or(ray_t.end);
            let t = ray_t.start..tmax;

            if let Some(hit) = mesh.hit(ray, t) {
                current_hit = Some(hit);
            }
        }

        current_hit
    }
}

pub struct Sphere {
    center: Vec3,
    radius: f32,
    material: Arc<dyn Material + Sync + Send>,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material: Arc<dyn Material + Sync + Send>) -> Self {
        assert!(radius >= 0., "Radius cannot be less than 0.");
        Self {
            center,
            radius,
            material,
        }
    }
}

impl Mesh for Sphere {
    fn hit(&self, ray: &Ray3d, ray_t: Range<f32>) -> Option<Hit> {
        let oc = self.center - ray.origin;
        let a = ray.direction.length_squared();
        let h = ray.direction.dot(oc);
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = h * h - a * c;
        if discriminant < 0. {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        let mut root = (h - sqrtd) / a;
        if !ray_t.contains(&root) {
            root = (h + sqrtd) / a;
            if !ray_t.contains(&root) {
                return None;
            }
        }

        let point = ray.get_point(root);
        let normal = (point - self.center) / self.radius;
        Some(Hit::new(ray, root, normal, self.material.clone()))
    }
}
