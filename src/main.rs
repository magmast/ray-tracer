use core::f32;
use std::fs::File;

use bevy_color::Color;
use bevy_math::Vec3;
use image::{ImageFormat, ImageResult};
use rand::prelude::*;
use ray_tracing::{
    camera::{Camera, CameraConfig},
    material::{Dielectric, Lamberian, Metal},
    mesh::{Sphere, World},
    utils::random_vec,
};

fn main() -> ImageResult<()> {
    let mut world = World::new();

    let ground_material = Lamberian::new(Color::linear_rgb(0.5, 0.5, 0.5));
    world.push(Sphere::new(
        Vec3::new(0., -1000., 0.),
        1000.,
        ground_material,
    ));

    let mut rng = rand::thread_rng();

    (-11..11)
        .into_iter()
        .flat_map(|x| (-11..11).map(move |y| [x as f32, y as f32]))
        .for_each(|[x, y]| {
            let center = Vec3::new(x + 0.9 * rng.gen::<f32>(), 0.2, y + 0.9 * rng.gen::<f32>());

            if (center - Vec3::new(4., 0.2, 0.)).length() <= 0.9 {
                return;
            }

            let choose_mat = rng.gen::<f32>();

            if choose_mat < 0.8 {
                let albedo = random_vec(&mut rng) * random_vec(&mut rng);
                let albedo = Color::linear_rgb(albedo.x, albedo.y, albedo.z);
                let sphere_material = Lamberian::new(albedo);
                world.push(Sphere::new(center, 0.2, sphere_material))
            } else if choose_mat < 0.95 {
                let albedo = random_vec(&mut rng);
                let albedo = Color::linear_rgb(albedo.x, albedo.y, albedo.z);
                let fuzz = random();
                let sphere_material = Metal::new(albedo, fuzz);
                world.push(Sphere::new(center, 0.2, sphere_material))
            } else {
                let material = Dielectric::new(1.5);
                world.push(Sphere::new(center, 0.2, material));
            }
        });

    let material1 = Dielectric::new(1.5);
    world.push(Sphere::new(Vec3::new(0., 1., 0.), 1., material1));

    let material2 = Lamberian::new(Color::linear_rgb(0.4, 0.2, 0.1));
    world.push(Sphere::new(Vec3::new(-4., 1., 0.), 1., material2));

    let material3 = Metal::new(Color::linear_rgb(0.7, 0.6, 0.5), 0.0);
    world.push(Sphere::new(Vec3::new(4., 1., 0.), 1., material3));

    let camera = Camera::new(CameraConfig {
        width: 1200,
        height: 675,
        samples_per_pixel: 500,
        max_depth: 50,
        lookfrom: Vec3::new(13., 2., 3.),
        lookat: Vec3::ZERO,
        vup: Vec3::Y,
        vfov: 20.,
        defocus_angle: 0.6,
        focus_dist: 10.,
        ..Default::default()
    });
    let image = camera.render(&world);

    let mut file = File::create("image.png")?;
    image.write_to(&mut file, ImageFormat::Png)?;

    Ok(())
}
