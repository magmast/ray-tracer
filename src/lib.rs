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

pub struct Interval {
    pub min: f32,
    pub max: f32,
}

impl Interval {
    pub const fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    pub fn surrounds(&self, x: f32) -> bool {
        self.min < x && self.max > x
    }

    pub fn clamp(&self, x: f32) -> f32 {
        x.clamp(self.min, self.max)
    }
}
