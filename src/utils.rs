use bevy::math::Vec3;
use rand::Rng as _;

pub fn random_vec() -> Vec3 {
    let mut rng = rand::thread_rng();

    Vec3::new(
        rng.gen_range(-1.0..1.),
        rng.gen_range(-1.0..1.),
        rng.gen_range(-1.0..1.),
    )
}

pub fn random_vec_in_unit_sphere() -> Vec3 {
    let mut rng = rand::thread_rng();

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

pub fn random_in_unit_disk() -> Vec3 {
    loop {
        let p = Vec3::new(rand::random::<f32>(), rand::random::<f32>(), 0.);
        if p.length_squared() < 1. {
            break p;
        }
    }
}

pub fn random_unit_vec() -> Vec3 {
    random_vec_in_unit_sphere().normalize()
}

pub fn near_zero(v: &Vec3) -> bool {
    let s = 1e-8;
    v.x.abs() < s && v.y.abs() < s && v.z.abs() < s
}
