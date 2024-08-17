use std::sync::Arc;

use bevy_color::{Color, LinearRgba};
use bevy_math::{Vec2, Vec3};

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
    fn scatter(&self, ray: &Ray, hit: &Hit) -> Option<Scatter>;

    fn emitted(&self, _uv: Vec2, _point: Vec3) -> LinearRgba {
        LinearRgba::BLACK
    }
}

impl<T: Material> Material for Arc<T> {
    fn scatter(&self, ray: &Ray, hit: &Hit) -> Option<Scatter> {
        self.as_ref().scatter(ray, hit)
    }

    fn emitted(&self, uv: Vec2, point: Vec3) -> LinearRgba {
        self.as_ref().emitted(uv, point)
    }
}

pub struct Scatter {
    pub attenuation: LinearRgba,
    pub scattered: Ray,
}

#[derive(Default)]
pub struct Lambertian<T: Texture> {
    pub texture: T,
}

impl<T: Texture> From<T> for Lambertian<T> {
    fn from(texture: T) -> Self {
        Self { texture }
    }
}

impl From<Color> for Lambertian<SolidTexture> {
    fn from(value: Color) -> Self {
        Self::from(SolidTexture::from(value))
    }
}

impl<T: Texture> Material for Lambertian<T> {
    fn scatter(&self, ray: &Ray, hit: &Hit) -> Option<Scatter> {
        let scatter_dir = self.scatter_direction(hit);
        Some(Scatter {
            attenuation: self.texture.value(hit.uv, hit.point),
            scattered: Ray::new(hit.point, scatter_dir, ray.time),
        })
    }
}

impl Lambertian<SolidTexture> {
    pub fn rgb(red: f32, green: f32, blue: f32) -> Self {
        Self::from(Color::linear_rgb(red, green, blue))
    }
}

impl<T: Texture> Lambertian<T> {
    fn scatter_direction(&self, hit: &Hit) -> Vec3 {
        let rng = rand::thread_rng();
        let dir = hit.normal + random_unit_vec(rng);
        if near_zero(dir) {
            hit.normal
        } else {
            dir
        }
    }
}

#[derive(Default)]
pub struct Metal<T: Texture> {
    pub texture: T,
    pub roughness: f32,
}

impl<T: Texture> Material for Metal<T> {
    fn scatter(&self, ray: &Ray, hit: &Hit) -> Option<Scatter> {
        let rng = rand::thread_rng();
        let reflected =
            reflect(ray.direction, hit.normal) + (self.roughness * random_unit_vec(rng));
        let scattered = Ray::new(hit.point, reflected, ray.time);
        if scattered.direction.dot(hit.normal) > 0. {
            Some(Scatter {
                attenuation: self.texture.value(hit.uv, hit.point),
                scattered,
            })
        } else {
            None
        }
    }
}

impl<T: Texture> Metal<T> {
    pub fn new(texture: T, roughness: f32) -> Self {
        Self { texture, roughness }
    }

    pub fn with_roughness(self, roughness: f32) -> Self {
        Self { roughness, ..self }
    }
}

impl Metal<SolidTexture> {
    pub fn rgb(red: f32, green: f32, blue: f32) -> Self {
        Self::new(SolidTexture::from(Color::linear_rgb(red, green, blue)), 0.0)
    }
}

pub struct Dielectric {
    pub refraction_index: f32,
}

impl Default for Dielectric {
    fn default() -> Self {
        Self {
            refraction_index: 1.5,
        }
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
            attenuation: LinearRgba::WHITE,
            scattered: Ray::new(hit.point, dir, r_in.time),
        })
    }
}

impl Dielectric {
    pub fn new(refraction_index: f32) -> Self {
        Self { refraction_index }
    }
}

#[derive(Default)]
pub struct DiffuseLight<T: Texture> {
    pub texture: T,
}

impl<T: Texture> From<T> for DiffuseLight<T> {
    fn from(texture: T) -> Self {
        Self { texture }
    }
}

impl From<Color> for DiffuseLight<SolidTexture> {
    fn from(value: Color) -> Self {
        Self::from(SolidTexture::from(value))
    }
}

impl<T: Texture> Material for DiffuseLight<T> {
    fn scatter(&self, _ray: &Ray, _hit: &Hit) -> Option<Scatter> {
        None
    }

    fn emitted(&self, uv: Vec2, point: Vec3) -> LinearRgba {
        self.texture.value(uv, point)
    }
}

impl DiffuseLight<SolidTexture> {
    pub fn rgb(red: f32, green: f32, blue: f32) -> Self {
        Self::from(Color::linear_rgb(red, green, blue))
    }
}

#[derive(Default)]
pub struct Isotropic<T: Texture> {
    pub texture: T,
}

impl<T: Texture> From<T> for Isotropic<T> {
    fn from(texture: T) -> Self {
        Self { texture }
    }
}

impl From<Color> for Isotropic<SolidTexture> {
    fn from(value: Color) -> Self {
        Self::from(SolidTexture::from(value))
    }
}

impl<T: Texture> Material for Isotropic<T> {
    fn scatter(&self, ray: &Ray, hit: &Hit) -> Option<Scatter> {
        let rng = rand::thread_rng();

        Some(Scatter {
            scattered: Ray::new(hit.point, random_unit_vec(rng), ray.time),
            attenuation: self.texture.value(hit.uv, hit.point),
        })
    }
}

impl Isotropic<SolidTexture> {
    pub fn rgb(red: f32, green: f32, blue: f32) -> Self {
        Self::from(Color::linear_rgb(red, green, blue))
    }
}
