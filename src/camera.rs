use bevy_color::{Color, ColorToComponents as _, ColorToPacked};
use bevy_math::Vec3;
use image::{Rgb, RgbImage};
use rand::Rng;

use crate::{
    mesh::Mesh,
    utils::{degrees_to_radians, random_vec_in_unit_disk},
    Ray,
};

pub struct CameraConfig {
    pub width: u32,
    pub height: u32,
    pub samples_per_pixel: usize,
    pub max_depth: usize,
    pub vfov: f32,
    pub lookfrom: Vec3,
    pub lookat: Vec3,
    pub vup: Vec3,
    pub defocus_angle: f32,
    pub focus_dist: f32,
    pub background: Color,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            width: 100,
            height: 100,
            samples_per_pixel: 10,
            max_depth: 10,
            vfov: 90.,
            lookfrom: Vec3::ZERO,
            lookat: Vec3::new(0., 0., -1.),
            vup: Vec3::Y,
            defocus_angle: 0.,
            focus_dist: 10.,
            background: Color::linear_rgb(0.7, 0.8, 1.),
        }
    }
}

pub struct Camera {
    config: CameraConfig,
    center: Vec3,
    pixel00_loc: Vec3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    pixel_samples_scale: f32,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
}

impl Camera {
    pub fn new(config: CameraConfig) -> Self {
        let aspect_ratio = config.width as f32 / config.height as f32;

        let center = config.lookfrom;

        let theta = degrees_to_radians(config.vfov);
        let h = (theta / 2.).tan();
        let viewport_height = 2. * h * config.focus_dist;
        let viewport_width = viewport_height * aspect_ratio;

        let w = (config.lookfrom - config.lookat).normalize();
        let u = config.vup.cross(w);
        let v = w.cross(u);

        let viewport_u = viewport_width * u;
        let viewport_v = viewport_height * -v;

        let pixel_delta_u = viewport_u / config.width as f32;
        let pixel_delta_v = viewport_v / config.height as f32;

        let viewport_upper_left =
            center - (config.focus_dist * w) - viewport_u / 2. - viewport_v / 2.;
        let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        let defocus_radius =
            config.focus_dist * degrees_to_radians(config.defocus_angle / 2.).tan();
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        Self {
            center,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
            pixel_samples_scale: 1. / config.samples_per_pixel as f32,
            config,
            defocus_disk_u,
            defocus_disk_v,
        }
    }

    pub fn render(&self, world: &(impl Mesh + Sync)) -> RgbImage {
        RgbImage::from_par_fn(self.config.width, self.config.height, |x, y| {
            Rgb(self
                .render_pixel(world, x, y)
                .to_srgba()
                .to_u8_array_no_alpha())
        })
    }

    fn render_pixel(&self, world: &impl Mesh, x: u32, y: u32) -> Color {
        let mut rng = rand::thread_rng();

        let color: Vec3 = (0..self.config.samples_per_pixel)
            .map(|_| {
                let ray = self.get_ray(&mut rng, x, y);
                self.ray_color(&ray, world, self.config.max_depth)
                    .to_linear()
                    .to_vec3()
            })
            .sum();

        let color = self.pixel_samples_scale * color;

        Color::linear_rgb(color.x, color.y, color.z)
    }

    fn ray_color(&self, ray: &Ray, world: &impl Mesh, depth: usize) -> Color {
        let color_vec = if depth <= 0 {
            Vec3::ZERO
        } else if let Some(hit) = world.hit(ray, &(0.001..f32::INFINITY).into()) {
            let color_from_emission = hit
                .material
                .emitted(hit.uv, hit.point)
                .to_linear()
                .to_vec3();

            if let Some(scatter) = hit.material.scatter(ray, &hit) {
                let color_from_scatter = scatter.attenuation.to_linear().to_vec3()
                    * self
                        .ray_color(&scatter.scattered, world, depth - 1)
                        .to_linear()
                        .to_vec3();

                color_from_scatter + color_from_emission
            } else {
                color_from_emission
            }
        } else {
            self.config.background.to_linear().to_vec3()
        };

        Color::linear_rgb(color_vec.x, color_vec.y, color_vec.z)
    }

    fn get_ray(&self, mut rng: impl Rng, x: u32, y: u32) -> Ray {
        let offset = Self::sample_square(&mut rng);
        let pixel_sample = self.pixel00_loc
            + ((x as f32 + offset.x) * self.pixel_delta_u)
            + ((y as f32 + offset.y) * self.pixel_delta_v);
        let ray_origin = if self.config.defocus_angle <= 0. {
            self.center
        } else {
            self.defocus_disk_sample(&mut rng)
        };

        Ray::new(ray_origin, pixel_sample - ray_origin, rng.gen())
    }

    fn sample_square(mut rng: impl Rng) -> Vec3 {
        Vec3::new(rng.gen::<f32>() - 0.5, rng.gen::<f32>() - 0.5, 0.)
    }

    fn defocus_disk_sample(&self, rng: impl Rng) -> Vec3 {
        let point = random_vec_in_unit_disk(rng);
        self.center + (point.x * self.defocus_disk_u) + (point.y * self.defocus_disk_v)
    }
}
