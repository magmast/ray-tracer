use core::f32;
use std::{
    fmt::{self, Display, Formatter},
    rc::Rc,
};

use nalgebra::Vector3;
use rand::prelude::*;
use rgb::Rgb;

const PI: f32 = 3.1415926535897932385;

fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * PI / 180.
}

fn random_vec_in_unit_sphere() -> Vector3<f32> {
    let mut rng = rand::thread_rng();

    loop {
        let p = Vector3::new(
            rng.gen_range(-1.0..1.),
            rng.gen_range(-1.0..1.),
            rng.gen_range(-1.0..1.),
        );

        if p.magnitude_squared() < 1. {
            break p;
        }
    }
}

fn random_unit_vec() -> Vector3<f32> {
    random_vec_in_unit_sphere().normalize()
}

fn random_vec_on_hemisphere(normal: &Vector3<f32>) -> Vector3<f32> {
    let on_unit_sphere = random_vec_in_unit_sphere().normalize();

    if on_unit_sphere.dot(normal) > 0. {
        on_unit_sphere
    } else {
        -on_unit_sphere
    }
}

fn random_in_unit_disk() -> Vector3<f32> {
    loop {
        let p = Vector3::new(rand::random::<f32>(), rand::random::<f32>(), 0.);
        if p.magnitude_squared() < 1. {
            break p;
        }
    }
}

fn linear_to_gamma(linear_component: f32) -> f32 {
    if linear_component > 0. {
        linear_component.sqrt()
    } else {
        0.
    }
}

fn near_zero(v: &Vector3<f32>) -> bool {
    let s = 1e-8;
    v.x.abs() < s && v.y.abs() < s && v.z.abs() < s
}

fn reflect(v: &Vector3<f32>, n: &Vector3<f32>) -> Vector3<f32> {
    v - 2. * v.dot(n) * n
}

fn refract(uv: &Vector3<f32>, n: &Vector3<f32>, etai_over_etat: f32) -> Vector3<f32> {
    let cos_theta = -uv.dot(&n).min(1.);
    let r_out_prep = etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = -(1. - r_out_prep.magnitude_squared()).abs().sqrt() * n;
    r_out_prep + r_out_parallel
}

fn main() {
    let mut world = Hittables::new();

    let ground_material: Rc<Box<dyn Material>> = Rc::new(Box::new(Lamberian {
        albedo: Vector3::new(0.8, 0.8, 0.),
    }));
    world.push(Sphere::new(
        Vector3::new(0., -100.0, 0.),
        1000.,
        ground_material,
    ));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = rand::random::<f32>();
            let center = Vector3::new(
                a as f32 + 0.9 * random::<f32>(),
                0.2,
                b as f32 + 0.9 * random::<f32>(),
            );

            if choose_mat < 0.8 {
                let albedo = Vector3::new_random().component_mul(&Vector3::new_random());
                let sphere_material: Rc<Box<dyn Material>> =
                    Rc::new(Box::new(Lamberian { albedo }));
                world.push(Sphere::new(center, 0.2, sphere_material))
            } else if choose_mat < 0.95 {
                let albedo = Vector3::new_random();
                let fuzz = random();
                let sphere_material: Rc<Box<dyn Material>> =
                    Rc::new(Box::new(Metal { albedo, fuzz }));
                world.push(Sphere::new(center, 0.2, sphere_material))
            } else {
                let material: Rc<Box<dyn Material>> = Rc::new(Box::new(Dielectric {
                    refraction_index: 1.5,
                }));
                world.push(Sphere::new(center, 0.2, material));
            }
        }
    }

    let material1: Rc<Box<dyn Material>> = Rc::new(Box::new(Dielectric {
        refraction_index: 1.5,
    }));
    world.push(Sphere::new(Vector3::new(0., 1., 0.), 1., material1));

    let material2: Rc<Box<dyn Material>> = Rc::new(Box::new(Lamberian {
        albedo: Vector3::new(0.4, 0.2, 0.1),
    }));
    world.push(Sphere::new(Vector3::new(-4., 1., 0.), 1., material2));

    let material3: Rc<Box<dyn Material>> = Rc::new(Box::new(Metal {
        albedo: Vector3::new(0.7, 0.6, 0.5),
        fuzz: 0.0,
    }));
    world.push(Sphere::new(Vector3::new(4., 1., 0.), 1., material3));

    let camera = Camera::new(CameraConfig {
        width: 1200,
        height: 675,
        samples_per_pixel: 20,
        max_depth: 50,
        lookfrom: Vector3::new(-13., -2., -3.),
        lookat: Vector3::zeros(),
        vup: Vector3::y(),
        vfov: 20.,
        defocus_angle: 0.6,
        focus_dist: 10.,
        ..Default::default()
    });
    camera.render(&world);
}

struct PpmRgbDisplay(Rgb<u8>);

impl Display for PpmRgbDisplay {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.0.r, self.0.g, self.0.b)?;
        Ok(())
    }
}

trait PpmRgbDisplayExt {
    fn ppm(self) -> PpmRgbDisplay;
}

impl PpmRgbDisplayExt for Rgb<f32> {
    fn ppm(self) -> PpmRgbDisplay {
        let intensity = Interval::new(0., 0.999);

        PpmRgbDisplay(Rgb::new(
            (256. * intensity.clamp(linear_to_gamma(self.r))) as u8,
            (256. * intensity.clamp(linear_to_gamma(self.g))) as u8,
            (256. * intensity.clamp(linear_to_gamma(self.b))) as u8,
        ))
    }
}

struct Ray {
    origin: Vector3<f32>,
    dir: Vector3<f32>,
}

impl Ray {
    fn new(origin: Vector3<f32>, dir: Vector3<f32>) -> Self {
        Ray { origin, dir }
    }

    fn at(&self, t: f32) -> Vector3<f32> {
        self.origin + t * self.dir
    }
}

struct Hit {
    point: Vector3<f32>,
    normal: Vector3<f32>,
    t: f32,
    front_face: bool,
    material: Rc<Box<dyn Material>>,
}

impl Hit {
    fn new(
        point: Vector3<f32>,
        normal: Vector3<f32>,
        t: f32,
        material: Rc<Box<dyn Material>>,
    ) -> Self {
        Self {
            point,
            normal,
            t,
            front_face: true,
            material,
        }
    }

    fn set_face_normal(&mut self, ray: &Ray, outward_normal: &Vector3<f32>) {
        self.front_face = ray.dir.dot(&outward_normal) < 0.;
        self.normal = if self.front_face {
            outward_normal.clone()
        } else {
            -outward_normal
        };
    }
}

trait Hittable {
    fn hit(&self, ray: &Ray, ray_t: &Interval) -> Option<Hit>;
}

struct Sphere {
    center: Vector3<f32>,
    radius: f32,
    material: Rc<Box<dyn Material>>,
}

impl Sphere {
    fn new(center: Vector3<f32>, radius: f32, material: Rc<Box<dyn Material>>) -> Self {
        assert!(radius >= 0., "Radius cannot be less than 0.");
        Self {
            center,
            radius,
            material,
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, ray_t: &Interval) -> Option<Hit> {
        let oc = self.center - ray.origin;
        let a = ray.dir.magnitude_squared();
        let h = ray.dir.dot(&oc);
        let c = oc.magnitude_squared() - self.radius * self.radius;

        let discriminant = h * h - a * c;
        if discriminant < 0. {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        let mut root = (h - sqrtd) / a;
        if !ray_t.surrounds(root) {
            root = (h + sqrtd) / a;
            if !ray_t.surrounds(root) {
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

struct Hittables(Vec<Box<dyn Hittable>>);

impl Hittables {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn push(&mut self, hittable: impl Hittable + 'static) {
        self.0.push(Box::new(hittable))
    }
}

impl Hittable for Hittables {
    fn hit(&self, ray: &Ray, ray_t: &Interval) -> Option<Hit> {
        let mut current_hit: Option<Hit> = None;

        for hittable in &self.0 {
            let tmax = current_hit.as_ref().map(|hit| hit.t).unwrap_or(ray_t.max);
            let t = Interval::new(ray_t.min, tmax);

            if let Some(hit) = hittable.hit(ray, &t) {
                current_hit = Some(hit);
            }
        }

        current_hit
    }
}

struct Interval {
    min: f32,
    max: f32,
}

impl Interval {
    const EMPTY: Self = Self::new(f32::INFINITY, f32::NEG_INFINITY);

    const UNIVERSE: Self = Self::new(f32::NEG_INFINITY, f32::INFINITY);

    const fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    fn len(&self) -> f32 {
        self.max - self.min
    }

    fn contains(&self, x: f32) -> bool {
        self.min <= x && self.max >= x
    }

    fn surrounds(&self, x: f32) -> bool {
        self.min < x && self.max > x
    }

    fn clamp(&self, x: f32) -> f32 {
        x.clamp(self.min, self.max)
    }
}

struct CameraConfig {
    width: i32,
    height: i32,
    samples_per_pixel: i32,
    max_depth: i32,
    vfov: f32,
    lookfrom: Vector3<f32>,
    lookat: Vector3<f32>,
    vup: Vector3<f32>,
    defocus_angle: f32,
    focus_dist: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            width: 100,
            height: 100,
            samples_per_pixel: 10,
            max_depth: 10,
            vfov: 90.,
            lookfrom: Vector3::zeros(),
            lookat: Vector3::new(0., 0., -1.),
            vup: Vector3::y(),
            defocus_angle: 0.,
            focus_dist: 10.,
        }
    }
}

struct Camera {
    config: CameraConfig,
    center: Vector3<f32>,
    pixel00_loc: Vector3<f32>,
    pixel_delta_u: Vector3<f32>,
    pixel_delta_v: Vector3<f32>,
    pixel_samples_scale: f32,
    defocus_disk_u: Vector3<f32>,
    defocus_disk_v: Vector3<f32>,
}

impl Camera {
    fn new(config: CameraConfig) -> Self {
        let aspect_ratio = config.width as f32 / config.height as f32;

        let center = config.lookfrom;

        let theta = degrees_to_radians(config.vfov);
        let h = (theta / 2.).tan();
        let viewport_height = 2. * h * config.focus_dist;
        let viewport_width = viewport_height * aspect_ratio;

        let w = (config.lookfrom - config.lookat).normalize();
        let u = config.vup.cross(&w);
        let v = w.cross(&u);

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

    fn ray_color(&self, ray: &Ray, world: &impl Hittable, depth: i32) -> Vector3<f32> {
        if depth <= 0 {
            Vector3::zeros()
        } else if let Some(hit) = world.hit(ray, &Interval::new(0.001, f32::INFINITY)) {
            if let Some(scatter) = hit.material.scatter(ray, &hit) {
                scatter.attenuation.component_mul(&self.ray_color(
                    &scatter.scattered,
                    world,
                    depth - 1,
                ))
            } else {
                Vector3::zeros()
            }
        } else {
            let unit_dir = ray.dir.normalize();
            let a = 0.5 * (unit_dir.y + 1.);
            (1. - a) * Vector3::new(1., 1., 1.) + a * Vector3::new(0.5, 0.7, 1.0)
        }
    }

    fn render(&self, world: &impl Hittable) {
        println!("P3\n{} {}\n255", self.config.width, self.config.height);

        let mut rng = rand::thread_rng();

        for y in 0..self.config.height {
            eprintln!("Scanlines remaining: {}.", self.config.height - y);
            for x in 0..self.config.width {
                let mut color = Vector3::zeros();
                for _ in 0..self.config.samples_per_pixel {
                    let ray = self.get_ray(&mut rng, x, y);
                    color += self.ray_color(&ray, world, self.config.max_depth);
                }

                println!("{}", (self.pixel_samples_scale * color).to_color().ppm());
            }
        }

        eprintln!("Done.");
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

    fn sample_square(rng: &mut impl Rng) -> Vector3<f32> {
        Vector3::new(rng.gen::<f32>() - 0.5, rng.gen::<f32>() - 0.5, 0.)
    }

    fn defocus_disk_sample(&self) -> Vector3<f32> {
        let point = random_in_unit_disk();
        self.center + (point.x * self.defocus_disk_u) + (point.y * self.defocus_disk_v)
    }
}

trait ToColor {
    fn to_color(&self) -> Rgb<f32>;
}

impl ToColor for Vector3<f32> {
    fn to_color(&self) -> Rgb<f32> {
        Rgb::new(self.x, self.y, self.z)
    }
}

trait Material {
    fn scatter(&self, r_in: &Ray, hit: &Hit) -> Option<Scatter>;
}

struct Scatter {
    attenuation: Vector3<f32>,
    scattered: Ray,
}

struct Lamberian {
    albedo: Vector3<f32>,
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

struct Metal {
    albedo: Vector3<f32>,
    fuzz: f32,
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, hit: &Hit) -> Option<Scatter> {
        let reflected = reflect(&r_in.dir, &hit.normal) + (self.fuzz * random_unit_vec());
        let scattered = Ray::new(hit.point, reflected);
        if scattered.dir.dot(&hit.normal) > 0. {
            Some(Scatter {
                attenuation: self.albedo,
                scattered,
            })
        } else {
            None
        }
    }
}

struct Dielectric {
    refraction_index: f32,
}

impl Dielectric {
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
        let cos_theta = -unit_dir.dot(&hit.normal).min(1.0);
        let sin_theta = (1. - cos_theta * cos_theta).sqrt();

        let cannot_refact = ri * sin_theta > 1.;
        let dir = if cannot_refact || Self::reflectance(cos_theta, ri) > rand::random::<f32>() {
            reflect(&unit_dir, &hit.normal)
        } else {
            refract(&unit_dir, &hit.normal, ri)
        };

        Some(Scatter {
            attenuation: Vector3::new(1., 1., 1.),
            scattered: Ray::new(hit.point, dir),
        })
    }
}
