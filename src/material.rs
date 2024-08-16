use std::sync::Arc;

use bevy_color::Color;
use bevy_math::Vec3;

use crate::{
    mesh::Hit,
    texture::{SolidTexture, Texture},
    utils::{near_zero, random_unit_vec},
    Ray,
};

fn reflectance(cosine: f32, refraction_index: f32) -> f32 {
    let mut r0 = (1. - refraction_index) / (1. + refraction_index);
    r0 *= r0;
    r0 + (1. - r0) * (1. - cosine).powi(5)
}

fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - 2. * v.dot(n) * n
}

fn refract(uv: Vec3, n: Vec3, etai_over_etat: f32) -> Vec3 {
    let cos_theta = -uv.dot(n).min(1.);
    let r_out_prep = etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = -(1. - r_out_prep.length_squared()).abs().sqrt() * n;
    r_out_prep + r_out_parallel
}

pub trait Material {
    fn scatter(&self, r_in: &Ray, hit: &Hit) -> Option<Scatter>;
}

pub struct Scatter {
    pub attenuation: Color,
    pub scattered: Ray,
}

pub struct Lambertian {
    texture: Arc<dyn Texture + Send + Sync>,
}

impl Lambertian {
    pub fn new(texture: Arc<dyn Texture + Send + Sync>) -> Arc<dyn Material + Sync + Send> {
        Arc::new(Self { texture })
    }

    pub fn from_color(color: Color) -> Arc<dyn Material + Sync + Send> {
        Self::new(SolidTexture::new(color))
    }
}

impl Material for Lambertian {
    fn scatter(&self, r_in: &Ray, hit: &Hit) -> Option<Scatter> {
        let rng = rand::thread_rng();
        let mut scatter_dir = hit.normal + random_unit_vec(rng);
        if near_zero(scatter_dir) {
            scatter_dir = hit.normal;
        }
        Some(Scatter {
            attenuation: self.texture.value(hit.uv, hit.point),
            scattered: Ray::new(hit.point, scatter_dir, r_in.time),
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
    fn scatter(&self, r_in: &Ray, hit: &Hit) -> Option<Scatter> {
        let rng = rand::thread_rng();
        let reflected = reflect(r_in.direction, hit.normal) + (self.fuzz * random_unit_vec(rng));
        let scattered = Ray::new(hit.point, reflected, r_in.time);
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
    fn scatter(&self, r_in: &Ray, hit: &Hit) -> Option<Scatter> {
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
            reflect(unit_dir, hit.normal)
        } else {
            refract(unit_dir, hit.normal, ri)
        };

        Some(Scatter {
            attenuation: Color::WHITE,
            scattered: Ray::new(hit.point, dir, r_in.time),
        })
    }
}
