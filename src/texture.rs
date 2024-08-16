use std::{path::Path, sync::Arc};

use bevy_color::Color;
use bevy_math::{Vec2, Vec3};
use image::{ImageResult, RgbImage};

use crate::Interval;

pub trait Texture {
    fn value(&self, uv: Vec2, point: Vec3) -> Color;
}

pub struct SolidTexture(Color);

impl SolidTexture {
    pub fn new(color: Color) -> Arc<dyn Texture + Send + Sync> {
        Arc::new(SolidTexture(color))
    }
}

impl Texture for SolidTexture {
    fn value(&self, _uv: Vec2, _point: Vec3) -> Color {
        self.0
    }
}

pub struct CheckerTexture {
    scale: f32,
    even: Arc<dyn Texture + Send + Sync>,
    odd: Arc<dyn Texture + Send + Sync>,
}

impl CheckerTexture {
    pub fn new(
        scale: f32,
        even: Arc<dyn Texture + Send + Sync>,
        odd: Arc<dyn Texture + Send + Sync>,
    ) -> Arc<dyn Texture + Send + Sync> {
        Arc::new(Self {
            scale: 1.0 / scale,
            even,
            odd,
        })
    }
}

impl Texture for CheckerTexture {
    fn value(&self, uv: Vec2, point: Vec3) -> Color {
        let x = (point.x * self.scale).floor() as i32;
        let y = (point.y * self.scale).floor() as i32;
        let z = (point.z * self.scale).floor() as i32;
        let sum = x + y + z;

        if sum % 2 == 0 {
            self.even.value(uv, point)
        } else {
            self.odd.value(uv, point)
        }
    }
}

pub struct ImageTexture {
    image: RgbImage,
}

impl ImageTexture {
    pub fn new(image: RgbImage) -> Arc<dyn Texture + Send + Sync> {
        Arc::new(Self { image })
    }

    pub fn open(path: impl AsRef<Path>) -> ImageResult<Arc<dyn Texture + Send + Sync>> {
        let image = image::open(path)?.to_rgb8();
        Ok(Arc::new(Self { image }))
    }
}

impl Texture for ImageTexture {
    fn value(&self, uv: Vec2, _point: Vec3) -> Color {
        let u = Interval::from(0.0..1.).clamp(uv.x);
        let v = 1. - Interval::from(0.0..1.).clamp(uv.y);

        let i = (u * self.image.width() as f32) as u32;
        let j = (v * self.image.height() as f32) as u32;
        let pixel = self.image.get_pixel(i, j);

        let color_scale = 1. / 255.;
        Color::linear_rgb(
            pixel[0] as f32 * color_scale,
            pixel[1] as f32 * color_scale,
            pixel[2] as f32 * color_scale,
        )
    }
}
