use bevy::{color::Color, math::Vec3};
use rand::Rng;

use crate::{mesh::Mesh, utils::random_in_unit_disk, Interval, Ray};

const PI: f32 = 3.1415926535897932385;

fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * PI / 180.
}

fn linear_to_gamma(linear_component: f32) -> f32 {
    if linear_component > 0. {
        linear_component.sqrt()
    } else {
        0.
    }
}

pub struct CameraConfig {
    pub width: i32,
    pub height: i32,
    pub samples_per_pixel: i32,
    pub max_depth: i32,
    pub vfov: f32,
    pub lookfrom: Vec3,
    pub lookat: Vec3,
    pub vup: Vec3,
    pub defocus_angle: f32,
    pub focus_dist: f32,
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

    fn ray_color(&self, ray: &Ray, world: &impl Mesh, depth: i32) -> Vec3 {
        if depth <= 0 {
            Vec3::ZERO
        } else if let Some(hit) = world.hit(ray, &Interval::new(0.001, f32::INFINITY)) {
            if let Some(scatter) = hit.material.scatter(ray, &hit) {
                scatter.attenuation * self.ray_color(&scatter.scattered, world, depth - 1)
            } else {
                Vec3::ZERO
            }
        } else {
            let unit_dir = ray.dir.normalize();
            let a = 0.5 * (unit_dir.y + 1.);
            (1. - a) * Vec3::new(1., 1., 1.) + a * Vec3::new(0.5, 0.7, 1.0)
        }
    }

    pub fn render(&self, world: &impl Mesh) {
        println!("P3\n{} {}\n255", self.config.width, self.config.height);

        let mut rng = rand::thread_rng();

        for y in 0..self.config.height {
            eprintln!("Scanlines remaining: {}.", self.config.height - y);
            for x in 0..self.config.width {
                let mut color = Vec3::ZERO;
                for _ in 0..self.config.samples_per_pixel {
                    let ray = self.get_ray(&mut rng, x, y);
                    color += self.ray_color(&ray, world, self.config.max_depth);
                }

                self.render_color(Color::srgb_from_array(
                    (self.pixel_samples_scale * color).to_array(),
                ));
            }
        }

        eprintln!("Done.");
    }

    fn render_color(&self, color: Color) {
        let intensity = Interval::new(0., 0.999);

        let srgba = color.to_srgba();

        println!(
            "{} {} {}",
            (intensity.clamp(linear_to_gamma(srgba.red)) * 255.) as u8,
            (intensity.clamp(linear_to_gamma(srgba.green)) * 255.) as u8,
            (intensity.clamp(linear_to_gamma(srgba.blue)) * 255.) as u8
        );
    }

    fn get_ray(&self, rng: &mut impl Rng, x: i32, y: i32) -> Ray {
        let offset = Self::sample_square(rng);
        let pixel_sample = self.pixel00_loc
            + ((x as f32 + offset.x) * self.pixel_delta_u)
            + ((y as f32 + offset.y) * self.pixel_delta_v);
        let ray_origin = if self.config.defocus_angle <= 0. {
            self.center
        } else {
            self.defocus_disk_sample()
        };

        Ray::new(self.center, pixel_sample - ray_origin)
    }

    fn sample_square(rng: &mut impl Rng) -> Vec3 {
        Vec3::new(rng.gen::<f32>() - 0.5, rng.gen::<f32>() - 0.5, 0.)
    }

    fn defocus_disk_sample(&self) -> Vec3 {
        let point = random_in_unit_disk();
        self.center + (point.x * self.defocus_disk_u) + (point.y * self.defocus_disk_v)
    }
}
