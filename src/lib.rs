use std::ops::{Add, Range};

use bevy_math::Vec3;

pub mod camera;
pub mod material;
pub mod mesh;
pub mod texture;
pub mod utils;

#[derive(Default)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub time: f32,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3, time: f32) -> Self {
        Self {
            origin,
            direction,
            time,
        }
    }

    pub fn get_point(&self, distance: f32) -> Vec3 {
        self.origin + self.direction * distance
    }
}

#[derive(Clone)]
pub struct Interval(Range<f32>);

impl Default for Interval {
    fn default() -> Self {
        Self(0.0..0.)
    }
}

impl From<Range<f32>> for Interval {
    fn from(value: Range<f32>) -> Self {
        Self(value)
    }
}

impl Interval {
    const UNIVERSE: Self = Self(f32::NEG_INFINITY..f32::INFINITY);

    pub fn start(&self) -> f32 {
        self.0.start
    }

    pub fn set_start(&mut self, value: f32) {
        self.0.start = value;
    }

    pub fn end(&self) -> f32 {
        self.0.end
    }

    pub fn set_end(&mut self, value: f32) {
        self.0.end = value;
    }

    pub fn contains(&self, value: f32) -> bool {
        self.0.contains(&value)
    }

    pub fn expand(&self, delta: f32) -> Self {
        let padding = delta / 2.;
        (self.start() - padding..self.end() + padding).into()
    }

    pub fn join(&self, other: &Self) -> Self {
        let start = self.start().min(other.start());
        let end = self.end().max(other.end());
        (start..end).into()
    }

    pub fn clamp(&self, value: f32) -> f32 {
        value.clamp(self.start(), self.end())
    }

    pub fn size(&self) -> f32 {
        self.end() - self.start()
    }
}

impl Add<f32> for &Interval {
    type Output = Interval;

    fn add(self, rhs: f32) -> Self::Output {
        Interval(self.start() + rhs..self.end() + rhs)
    }
}

impl Add<f32> for Interval {
    type Output = Interval;

    fn add(self, rhs: f32) -> Self::Output {
        &self + rhs
    }
}
