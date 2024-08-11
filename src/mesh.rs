use std::{ops::Range, rc::Rc};

use bevy::math::Vec3;

use crate::{material::Material, Ray};

pub trait Mesh {
    fn hit(&self, ray: &Ray, ray_t: Range<f32>) -> Option<Hit>;
}

pub struct Hit {
    pub point: Vec3,
    pub normal: Vec3,
    pub t: f32,
    pub front_face: bool,
    pub material: Rc<dyn Material>,
}

impl Hit {
    pub fn new(point: Vec3, normal: Vec3, t: f32, material: Rc<dyn Material>) -> Self {
        Self {
            point,
            normal,
            t,
            front_face: true,
            material,
        }
    }

    pub fn set_face_normal(&mut self, ray: &Ray, outward_normal: &Vec3) {
        self.front_face = ray.dir.dot(*outward_normal) < 0.;
        self.normal = if self.front_face {
            outward_normal.clone()
        } else {
            -*outward_normal
        };
    }
}

pub struct World(Vec<Box<dyn Mesh>>);

impl World {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, mesh: impl Mesh + 'static) {
        self.0.push(Box::new(mesh))
    }
}

impl Mesh for World {
    fn hit(&self, ray: &Ray, ray_t: Range<f32>) -> Option<Hit> {
        let mut current_hit: Option<Hit> = None;

        for mesh in &self.0 {
            let tmax = current_hit.as_ref().map(|hit| hit.t).unwrap_or(ray_t.end);
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
    material: Rc<dyn Material>,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material: Rc<dyn Material>) -> Self {
        assert!(radius >= 0., "Radius cannot be less than 0.");
        Self {
            center,
            radius,
            material,
        }
    }
}

impl Mesh for Sphere {
    fn hit(&self, ray: &Ray, ray_t: Range<f32>) -> Option<Hit> {
        let oc = self.center - ray.origin;
        let a = ray.dir.length_squared();
        let h = ray.dir.dot(oc);
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

        let point = ray.at(root);
        let outward_normal = (point - self.center) / self.radius;
        let mut hit = Hit::new(point, outward_normal, root, self.material.clone());
        hit.set_face_normal(ray, &outward_normal);

        Some(hit)
    }
}
