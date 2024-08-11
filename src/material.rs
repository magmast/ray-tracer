use std::rc::Rc;

use bevy::math::Vec3;

use crate::{
    mesh::Hit,
    utils::{near_zero, random_unit_vec},
    Ray,
};

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
    fn scatter(&self, r_in: &Ray, hit: &Hit) -> Option<Scatter>;
}

pub struct Scatter {
    pub attenuation: Vec3,
    pub scattered: Ray,
}

pub struct Lamberian {
    albedo: Vec3,
}

impl Lamberian {
    pub fn new(albedo: Vec3) -> Rc<dyn Material> {
        Rc::new(Lamberian { albedo })
    }
}

impl Material for Lamberian {
    fn scatter(&self, _r_in: &Ray, hit: &Hit) -> Option<Scatter> {
        let mut scatter_dir = hit.normal + random_unit_vec();
        if near_zero(&scatter_dir) {
            scatter_dir = hit.normal;
        }
        Some(Scatter {
            attenuation: self.albedo,
            scattered: Ray::new(hit.point, scatter_dir),
        })
    }
}

pub struct Metal {
    albedo: Vec3,
    fuzz: f32,
}

impl Metal {
    pub fn new(albedo: Vec3, fuzz: f32) -> Rc<dyn Material> {
        Rc::new(Metal { albedo, fuzz })
    }
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, hit: &Hit) -> Option<Scatter> {
        let reflected = reflect(&r_in.dir, &hit.normal) + (self.fuzz * random_unit_vec());
        let scattered = Ray::new(hit.point, reflected);
        if scattered.dir.dot(hit.normal) > 0. {
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
    pub fn new(refraction_index: f32) -> Rc<dyn Material> {
        Rc::new(Dielectric { refraction_index })
    }

    fn reflectance(cosine: f32, refraction_index: f32) -> f32 {
        let mut r0 = (1. - refraction_index) / (1. + refraction_index);
        r0 *= r0;
        r0 + (1. - r0) * (1. - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn scatter(&self, r_in: &Ray, hit: &Hit) -> Option<Scatter> {
        let ri = if hit.front_face {
            1. / self.refraction_index
        } else {
            self.refraction_index
        };

        let unit_dir = r_in.dir.normalize();
        let cos_theta = -unit_dir.dot(hit.normal).min(1.0);
        let sin_theta = (1. - cos_theta * cos_theta).sqrt();

        let cannot_refact = ri * sin_theta > 1.;
        let dir = if cannot_refact || Self::reflectance(cos_theta, ri) > rand::random::<f32>() {
            reflect(&unit_dir, &hit.normal)
        } else {
            refract(&unit_dir, &hit.normal, ri)
        };

        Some(Scatter {
            attenuation: Vec3::new(1., 1., 1.),
            scattered: Ray::new(hit.point, dir),
        })
    }
}
