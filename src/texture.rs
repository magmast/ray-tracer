use std::{path::Path, sync::Arc};

use bevy_color::Color;
use bevy_math::{Vec2, Vec3};
use image::{ImageResult, RgbImage};
use rand::Rng;

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

        let i = (u * (self.image.width() as f32 - 1.)) as u32;
        let j = (v * (self.image.height() as f32 - 1.)) as u32;
        let pixel = self.image.get_pixel(i, j);

        let color_scale = 1. / 255.;
        Color::linear_rgb(
            pixel[0] as f32 * color_scale,
            pixel[1] as f32 * color_scale,
            pixel[2] as f32 * color_scale,
        )
    }
}

pub struct NoiseTexture<const N: usize = 256> {
    perlin: Perlin<N>,
    scale: f32,
}

impl<const N: usize> NoiseTexture<N> {
    pub fn new(rng: impl Rng, scale: f32) -> Arc<dyn Texture + Send + Sync> {
        Arc::new(Self {
            perlin: Perlin::new(rng),
            scale,
        })
    }
}

impl<const N: usize> Texture for NoiseTexture<N> {
    fn value(&self, _uv: Vec2, point: Vec3) -> Color {
        let color = Vec3::ONE
            * 0.5
            * (1. + (self.scale * point.z + 10. * self.perlin.turb(point, 7)).sin());
        Color::linear_rgb(color.x, color.y, color.z)
    }
}

struct Perlin<const N: usize> {
    values: [Vec3; N],
    perm_x: [usize; N],
    perm_y: [usize; N],
    perm_z: [usize; N],
}

impl<const N: usize> Perlin<N> {
    fn new(mut rng: impl Rng) -> Self {
        let mut values = [Vec3::ZERO; N];
        for i in 0..N {
            values[i] = Vec3::new(
                rng.gen_range(-1.0..1.),
                rng.gen_range(-1.0..1.),
                rng.gen_range(-1.0..1.),
            );
        }

        let perm_x = Self::generate_perm(&mut rng);
        let perm_y = Self::generate_perm(&mut rng);
        let perm_z = Self::generate_perm(&mut rng);

        Self {
            values,
            perm_x,
            perm_y,
            perm_z,
        }
    }

    fn generate_perm(mut rng: impl Rng) -> [usize; N] {
        let mut perm = [0; N];

        for i in 0..N {
            perm[i] = i;
        }

        for i in (1..N).rev() {
            let target = rng.gen_range(0..i);
            let tmp = perm[i];
            perm[i] = perm[target];
            perm[target] = tmp;
        }

        perm
    }

    fn noise(&self, point: Vec3) -> f32 {
        let u = point.x - point.x.floor();
        let v = point.y - point.y.floor();
        let w = point.z - point.z.floor();

        let i = point.x.floor() as i32;
        let j = point.y.floor() as i32;
        let k = point.z.floor() as i32;
        let mut c = [[[Vec3::ZERO; 2]; 2]; 2];

        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    c[di][dj][dk] = self.values[self.perm_x[((i + di as i32) & 255) as usize]
                        ^ self.perm_y[((j + dj as i32) & 255) as usize]
                        ^ self.perm_z[((k + dk as i32) & 255) as usize]];
                }
            }
        }

        self.perlin_interp(c, u, v, w)
    }

    fn perlin_interp(&self, c: [[[Vec3; 2]; 2]; 2], u: f32, v: f32, w: f32) -> f32 {
        let uu = u * u * (3. - 2. * u);
        let vv = v * v * (3. - 2. * v);
        let ww = w * w * (3. - 2. * w);
        let mut accum = 0.;

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let weight_v = Vec3::new(u - i as f32, v - j as f32, w - k as f32);
                    accum += (i as f32 * uu + (1. - i as f32) * (1. - uu))
                        * (j as f32 * vv + (1. - j as f32) * (1. - vv))
                        * (k as f32 * ww + (1. - k as f32) * (1. - ww))
                        * c[i][j][k].dot(weight_v);
                }
            }
        }

        accum
    }

    fn turb(&self, point: Vec3, depth: usize) -> f32 {
        let mut accum = 0.;
        let mut temp_p = point;
        let mut weight = 1.;

        for _ in 0..depth {
            accum += weight * self.noise(temp_p);
            temp_p *= 2.;
            weight *= 0.5;
        }

        accum.abs()
    }
}
