use std::sync::Arc;

use bevy_color::Color;
use bevy_math::{Ray3d, Vec3};

use crate::{
    mesh::Hit,
    utils::{near_zero, random_unit_vec},
};

fn reflectance(cosine: f32, refraction_index: f32) -> f32 {
    let mut r0 = (1. - refraction_index) / (1. + refraction_index);
    r0 *= r0;
    r0 + (1. - r0) * (1. - cosine).powi(5)
}

fn reflect(v: &Vec3, n: &Vec3) -> Vec3 {
    *v - 2. * v.dot(*n) * *n
}

fn refract(uv: &Vec3, n: &Vec3, etai_over_etat: f32) -> Vec3 {
    let cos_theta = -uv.dot(*n).min(1.);
    let r_out_prep = etai_over_etat * (*uv + cos_theta * *n);
    let r_out_parallel = -(1. - r_out_prep.length_squared()).abs().sqrt() * *n;
    r_out_prep + r_out_parallel
}

pub trait Material {
    fn scatter(&self, r_in: &Ray3d, hit: &Hit) -> Option<Scatter>;
}

pub struct Scatter {
    pub attenuation: Color,
    pub scattered: Ray3d,
}

pub struct Lamberian {
    albedo: Color,
}

impl Lamberian {
    pub fn new(albedo: Color) -> Arc<dyn Material + Sync + Send> {
        Arc::new(Lamberian { albedo })
    }
}

impl Material for Lamberian {
    fn scatter(&self, _r_in: &Ray3d, hit: &Hit) -> Option<Scatter> {
        let rng = rand::thread_rng();
        let mut scatter_dir = hit.normal + random_unit_vec(rng);
        if near_zero(&scatter_dir) {
            scatter_dir = hit.normal;
        }
        Some(Scatter {
            attenuation: self.albedo,
            scattered: Ray3d::new(hit.point, scatter_dir),
        })
    }
}

pub struct Metal {
    albedo: Color,
    fuzz: f32,
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f32) -> Arc<dyn Material + Sync + Send> {
        Arc::new(Metal { albedo, fuzz })
    }
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray3d, hit: &Hit) -> Option<Scatter> {
        let rng = rand::thread_rng();
        let reflected = reflect(&r_in.direction, &hit.normal) + (self.fuzz * random_unit_vec(rng));
        let scattered = Ray3d::new(hit.point, reflected);
        if scattered.direction.dot(hit.normal) > 0. {
            Some(Scatter {
                attenuation: self.albedo,
                scattered,
            })
        } else {
            None
        }
    }
}

pub struct Dielectric {
    refraction_index: f32,
}

impl Dielectric {
    pub fn new(refraction_index: f32) -> Arc<dyn Material + Sync + Send> {
        Arc::new(Dielectric { refraction_index })
    }
}

impl Material for Dielectric {
    fn scatter(&self, r_in: &Ray3d, hit: &Hit) -> Option<Scatter> {
        let ri = if hit.front_face {
            1. / self.refraction_index
        } else {
            self.refraction_index
        };

        let unit_dir = r_in.direction.normalize();
        let cos_theta = -unit_dir.dot(hit.normal).min(1.0);
        let sin_theta = (1. - cos_theta * cos_theta).sqrt();

        let cannot_refact = ri * sin_theta > 1.;
        let dir = if cannot_refact || reflectance(cos_theta, ri) > rand::random::<f32>() {
            reflect(&unit_dir, &hit.normal)
        } else {
            refract(&unit_dir, &hit.normal, ri)
        };

        Some(Scatter {
            attenuation: Color::WHITE,
            scattered: Ray3d::new(hit.point, dir),
        })
    }
}
