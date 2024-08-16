use core::f32;
use std::fs::File;

use bevy_color::Color;
use bevy_math::Vec3;
use image::{ImageFormat, ImageResult};
use rand::prelude::*;
use ray_tracing::{
    camera::{Camera, CameraConfig},
    material::{Dielectric, DiffuseLight, Lambertian, Metal},
    mesh::{Bvh, Cube, Quad, RotateY, Sphere, Translate, World},
    texture::{CheckerTexture, ImageTexture, NoiseTexture, SolidTexture},
    utils::random_vec,
};

const ASPECT_RATIO: f32 = 16. / 9.;
const IMAGE_WIDTH: u32 = 1920;

#[allow(unused)]
fn bouncing_spheres() -> ImageResult<(CameraConfig, World)> {
    let mut world = World::new();

    let ground_material = Lambertian::new(CheckerTexture::new(
        0.32,
        SolidTexture::new(Color::linear_rgb(0.2, 0.3, 0.1)),
        SolidTexture::new(Color::linear_rgb(0.9, 0.9, 0.9)),
    ));
    world.push(Sphere::stationary(
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
                let sphere_material = Lambertian::from_color(albedo);
                let to_center = center + Vec3::Y * rng.gen_range(0.0..0.5);
                world.push(Sphere::moving(center, to_center, 0.2, sphere_material))
            } else if choose_mat < 0.95 {
                let albedo = random_vec(&mut rng);
                let albedo = Color::linear_rgb(albedo.x, albedo.y, albedo.z);
                let fuzz = random();
                let sphere_material = Metal::new(albedo, fuzz);
                world.push(Sphere::stationary(center, 0.2, sphere_material))
            } else {
                let material = Dielectric::new(1.5);
                world.push(Sphere::stationary(center, 0.2, material));
            }
        });

    let material1 = Dielectric::new(1.5);
    world.push(Sphere::stationary(Vec3::new(0., 1., 0.), 1., material1));

    let material2 = Lambertian::from_color(Color::linear_rgb(0.4, 0.2, 0.1));
    world.push(Sphere::stationary(Vec3::new(-4., 1., 0.), 1., material2));

    let material3 = Metal::new(Color::linear_rgb(0.7, 0.6, 0.5), 0.0);
    world.push(Sphere::stationary(Vec3::new(4., 1., 0.), 1., material3));

    Ok((
        CameraConfig {
            width: IMAGE_WIDTH,
            height: (IMAGE_WIDTH as f32 / ASPECT_RATIO) as u32,
            samples_per_pixel: 100,
            max_depth: 50,
            lookfrom: Vec3::new(13., 2., 3.),
            lookat: Vec3::ZERO,
            vup: Vec3::Y,
            vfov: 20.,
            defocus_angle: 0.6,
            focus_dist: 10.,
            ..Default::default()
        },
        world,
    ))
}

#[allow(unused)]
fn checkered_spheres() -> ImageResult<(CameraConfig, World)> {
    let mut world = World::new();

    let checker = Lambertian::new(CheckerTexture::new(
        0.32,
        SolidTexture::new(Color::linear_rgb(0.2, 0.3, 0.1)),
        SolidTexture::new(Color::linear_rgb(0.9, 0.9, 0.9)),
    ));

    world.push(Sphere::stationary(Vec3::Y * -10., 10., checker.clone()));
    world.push(Sphere::stationary(Vec3::Y * 10., 10., checker));

    Ok((
        CameraConfig {
            width: IMAGE_WIDTH,
            height: (IMAGE_WIDTH as f32 / ASPECT_RATIO) as u32,
            samples_per_pixel: 100,
            max_depth: 50,
            vfov: 20.,
            lookfrom: Vec3::new(13., 2., 3.),
            lookat: Vec3::ZERO,
            vup: Vec3::Y,
            defocus_angle: 0.,
            ..Default::default()
        },
        world,
    ))
}

#[allow(unused)]
fn earth() -> ImageResult<(CameraConfig, World)> {
    let earth_texture = ImageTexture::open("images/earthmap.jpg")?;
    let earth_surface = Lambertian::new(earth_texture);

    let mut world = World::new();
    world.push(Sphere::stationary(Vec3::ZERO, 8., earth_surface));

    Ok((
        CameraConfig {
            width: IMAGE_WIDTH,
            height: (IMAGE_WIDTH as f32 / ASPECT_RATIO) as u32,
            samples_per_pixel: 100,
            max_depth: 50,
            lookfrom: Vec3::new(0., 0., 12.),
            lookat: Vec3::ZERO,
            vup: Vec3::Y,
            defocus_angle: 0.,
            ..Default::default()
        },
        world,
    ))
}

#[allow(unused)]
fn perlin_spheres() -> ImageResult<(CameraConfig, World)> {
    let mut world = World::new();

    let perlin = NoiseTexture::<256>::new(rand::thread_rng(), 4.);
    let perlin_surface = Lambertian::new(perlin);

    world.push(Sphere::stationary(
        Vec3::Y * -1000.,
        1000.,
        perlin_surface.clone(),
    ));
    world.push(Sphere::stationary(Vec3::Y * 2., 2., perlin_surface));

    Ok((
        CameraConfig {
            width: IMAGE_WIDTH,
            height: (IMAGE_WIDTH as f32 / ASPECT_RATIO) as u32,
            samples_per_pixel: 100,
            max_depth: 50,
            vfov: 20.,
            lookfrom: Vec3::new(13., 2., 3.),
            lookat: Vec3::ZERO,
            vup: Vec3::Y,
            defocus_angle: 0.,
            ..Default::default()
        },
        world,
    ))
}

#[allow(unused)]
fn quads() -> ImageResult<(CameraConfig, World)> {
    let left_red = Lambertian::from_color(Color::linear_rgb(1., 0.2, 0.2));
    let back_green = Lambertian::from_color(Color::linear_rgb(0.2, 1., 0.2));
    let right_blue = Lambertian::from_color(Color::linear_rgb(0.2, 0.2, 1.));
    let upper_orange = Lambertian::from_color(Color::linear_rgb(1., 0.5, 0.));
    let lower_teal = Lambertian::from_color(Color::linear_rgb(0.2, 0.8, 0.8));

    let mut world = World::new();
    world.push(Quad::new(
        Vec3::new(-3., -2., 5.),
        Vec3::Z * -4.,
        Vec3::Y * 4.,
        left_red,
    ));
    world.push(Quad::new(
        Vec3::new(-2., -2., 0.),
        Vec3::X * 4.,
        Vec3::Y * 4.,
        back_green,
    ));
    world.push(Quad::new(
        Vec3::new(3., -2., 1.),
        Vec3::Z * 4.,
        Vec3::Y * 4.,
        right_blue,
    ));
    world.push(Quad::new(
        Vec3::new(-2., 3., 1.),
        Vec3::X * 4.,
        Vec3::Z * 4.,
        upper_orange,
    ));
    world.push(Quad::new(
        Vec3::new(-2., -3., 5.),
        Vec3::X * 4.,
        Vec3::Z * -4.,
        lower_teal,
    ));

    Ok((
        CameraConfig {
            width: IMAGE_WIDTH,
            height: IMAGE_WIDTH,
            samples_per_pixel: 100,
            max_depth: 50,
            vfov: 80.,
            lookfrom: Vec3::Z * 9.,
            lookat: Vec3::ZERO,
            vup: Vec3::Y,
            defocus_angle: 0.,
            ..Default::default()
        },
        world,
    ))
}

#[allow(unused)]
fn simple_light() -> ImageResult<(CameraConfig, World)> {
    let pertext = Lambertian::new(NoiseTexture::<256>::new(rand::thread_rng(), 4.));
    let difflight = DiffuseLight::from_color(Color::linear_rgb(4., 4., 4.));

    let mut world = World::new();
    world.push(Sphere::stationary(Vec3::Y * -1000., 1000., pertext.clone()));
    world.push(Sphere::stationary(Vec3::Y * 2., 2., pertext));
    world.push(Quad::new(
        Vec3::new(3., 1., -2.),
        Vec3::X * 2.,
        Vec3::Y * 2.,
        difflight.clone(),
    ));
    world.push(Sphere::stationary(Vec3::Y * 7., 2., difflight));

    Ok((
        CameraConfig {
            width: IMAGE_WIDTH,
            height: (IMAGE_WIDTH as f32 / ASPECT_RATIO) as u32,
            samples_per_pixel: 500,
            max_depth: 50,
            background: Color::BLACK,
            lookfrom: Vec3::new(26., 3., 6.),
            lookat: Vec3::Y * 2.,
            vup: Vec3::Y,
            defocus_angle: 0.,
            ..Default::default()
        },
        world,
    ))
}

#[allow(unused)]
fn cornell_box() -> ImageResult<(CameraConfig, World)> {
    let red = Lambertian::from_color(Color::linear_rgb(0.65, 0.05, 0.05));
    let white = Lambertian::from_color(Color::linear_rgb(0.73, 0.73, 0.73));
    let green = Lambertian::from_color(Color::linear_rgb(0.12, 0.45, 0.15));
    let light = DiffuseLight::from_color(Color::linear_rgb(15.0, 15.0, 15.0));

    let mut world = World::new();
    world.push(Quad::new(
        Vec3::X * 555.,
        Vec3::Y * 555.,
        Vec3::Z * 555.,
        green.clone(),
    ));
    world.push(Quad::new(
        Vec3::ZERO,
        Vec3::Y * 555.,
        Vec3::Z * 555.,
        red.clone(),
    ));
    world.push(Quad::new(
        Vec3::new(343., 554., 332.),
        Vec3::X * -130.0,
        Vec3::Z * -105.0,
        light.clone(),
    ));
    world.push(Quad::new(
        Vec3::ZERO,
        Vec3::X * 555.,
        Vec3::Z * 555.,
        white.clone(),
    ));
    world.push(Quad::new(
        Vec3::ONE * 555.,
        Vec3::X * -555.,
        Vec3::Z * -555.,
        white.clone(),
    ));
    world.push(Quad::new(
        Vec3::Z * 555.,
        Vec3::X * 555.,
        Vec3::Y * 555.,
        white.clone(),
    ));

    world.push(Translate::new(
        RotateY::new(
            Cube::new(Vec3::ZERO, Vec3::new(165., 330., 165.), white.clone()),
            15.,
        ),
        Vec3::new(265., 0., 295.),
    ));
    world.push(Translate::new(
        RotateY::new(
            Cube::new(Vec3::ZERO, Vec3::new(165., 165., 165.), white.clone()),
            -18.,
        ),
        Vec3::new(130., 0., 65.),
    ));

    Ok((
        CameraConfig {
            width: 600,
            height: 600,
            samples_per_pixel: 1000,
            max_depth: 50,
            background: Color::BLACK,
            lookfrom: Vec3::new(278.0, 278.0, -800.0),
            lookat: Vec3::new(278.0, 278.0, 0.0),
            vup: Vec3::Y,
            defocus_angle: 0.0,
            vfov: 40.0,
            ..Default::default()
        },
        world,
    ))
}

fn main() -> ImageResult<()> {
    let (camera_config, world) = cornell_box()?;
    let bvh_world = Bvh::from(&world);

    let camera = Camera::new(camera_config);
    let image = camera.render(&bvh_world);

    let mut file = File::create("image.png")?;
    image.write_to(&mut file, ImageFormat::Png)?;

    Ok(())
}
