use core::f32;

use bevy::prelude::Vec3;
use rand::prelude::*;
use ray_tracing::{
    camera::{Camera, CameraConfig},
    material::{Dielectric, Lamberian, Metal},
    mesh::{Sphere, World},
    utils::random_vec,
};

fn main() {
    let mut world = World::new();

    let ground_material = Lamberian::new(Vec3::new(0.8, 0.8, 0.));
    world.push(Sphere::new(
        Vec3::new(0., -100.0, 0.),
        1000.,
        ground_material,
    ));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = rand::random::<f32>();
            let center = Vec3::new(
                a as f32 + 0.9 * random::<f32>(),
                0.2,
                b as f32 + 0.9 * random::<f32>(),
            );

            if choose_mat < 0.8 {
                let albedo = random_vec() * random_vec();
                let sphere_material = Lamberian::new(albedo);
                world.push(Sphere::new(center, 0.2, sphere_material))
            } else if choose_mat < 0.95 {
                let albedo = random_vec();
                let fuzz = random();
                let sphere_material = Metal::new(albedo, fuzz);
                world.push(Sphere::new(center, 0.2, sphere_material))
            } else {
                let material = Dielectric::new(1.5);
                world.push(Sphere::new(center, 0.2, material));
            }
        }
    }

    let material1 = Dielectric::new(1.5);
    world.push(Sphere::new(Vec3::new(0., 1., 0.), 1., material1));

    let material2 = Lamberian::new(Vec3::new(0.4, 0.2, 0.1));
    world.push(Sphere::new(Vec3::new(-4., 1., 0.), 1., material2));

    let material3 = Metal::new(Vec3::new(0.7, 0.6, 0.5), 0.0);
    world.push(Sphere::new(Vec3::new(4., 1., 0.), 1., material3));

    let camera = Camera::new(CameraConfig {
        width: 1200,
        height: 675,
        samples_per_pixel: 20,
        max_depth: 50,
        lookfrom: Vec3::new(-13., -2., -3.),
        lookat: Vec3::ZERO,
        vup: Vec3::Y,
        vfov: 20.,
        defocus_angle: 0.6,
        focus_dist: 10.,
        ..Default::default()
    });
    camera.render(&world);
}
