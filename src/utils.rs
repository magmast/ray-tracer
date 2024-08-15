use bevy_math::Vec3;
use rand::Rng;

pub const PI: f32 = 3.1415926535897932385;

pub fn random_vec(mut rng: impl Rng) -> Vec3 {
    Vec3::new(
        rng.gen_range(-1.0..1.),
        rng.gen_range(-1.0..1.),
        rng.gen_range(-1.0..1.),
    )
}

pub fn random_vec_in_unit_sphere(mut rng: impl Rng) -> Vec3 {
    loop {
        let p = Vec3::new(
            rng.gen_range(-1.0..1.),
            rng.gen_range(-1.0..1.),
            rng.gen_range(-1.0..1.),
        );

        if p.length_squared() < 1. {
            break p;
        }
    }
}

pub fn random_vec_in_unit_disk(mut rng: impl Rng) -> Vec3 {
    loop {
        let p = Vec3::new(rng.gen_range(-1.0..1.), rng.gen_range(-1.0..1.), 0.);
        if p.length_squared() < 1. {
            break p;
        }
    }
}

pub fn random_unit_vec(rng: impl Rng) -> Vec3 {
    random_vec_in_unit_sphere(rng).normalize()
}

pub fn near_zero(v: Vec3) -> bool {
    let s = 1e-8;
    v.x.abs() < s && v.y.abs() < s && v.z.abs() < s
}

pub fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * PI / 180.
}
