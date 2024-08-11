use bevy::math::Vec3;

pub mod camera;
pub mod material;
pub mod mesh;
pub mod utils;

pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, dir: Vec3) -> Self {
        Ray { origin, dir }
    }

    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + t * self.dir
    }
}
