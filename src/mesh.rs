use core::f32;
use std::{
    ops::{Add, Index},
    sync::Arc,
};

use bevy_color::Color;
use bevy_math::{Vec2, Vec3};

use crate::{
    material::{Isotropic, Material},
    texture::{SolidTexture, Texture},
    utils::{degrees_to_radians, PI},
    Interval, Ray,
};

pub trait Mesh {
    fn hit(&self, ray: &Ray, ray_t: &Interval) -> Option<Hit>;

    fn bounding_box(&self) -> &Aabb;
}

pub struct Hit {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub front_face: bool,
    pub material: Arc<dyn Material>,
    pub uv: Vec2,
}

impl Hit {
    pub fn new(
        ray: &Ray,
        distance: f32,
        normal: Vec3,
        material: Arc<dyn Material>,
        uv: Vec2,
    ) -> Self {
        let front_face = ray.direction.dot(normal) < 0.;
        let point = ray.get_point(distance);

        Self {
            point,
            normal: if front_face { normal } else { -normal },
            distance,
            front_face: true,
            material,
            uv,
        }
    }
}

pub struct World {
    meshes: Vec<Arc<dyn Mesh + Sync + Send>>,
    bbox: Aabb,
}

impl World {
    pub fn new() -> Self {
        Self {
            meshes: Vec::new(),
            bbox: Aabb::default(),
        }
    }

    pub fn push(&mut self, mesh: impl Mesh + Sync + Send + 'static) {
        self.bbox = self.bbox.merge(&mesh.bounding_box());
        self.meshes.push(Arc::new(mesh));
    }
}

impl AsRef<[Arc<dyn Mesh + Sync + Send>]> for World {
    fn as_ref(&self) -> &[Arc<dyn Mesh + Sync + Send>] {
        self.meshes.as_slice()
    }
}

impl Mesh for World {
    fn hit(&self, ray: &Ray, ray_t: &Interval) -> Option<Hit> {
        let mut current_hit: Option<Hit> = None;

        for mesh in &self.meshes {
            let tmax = current_hit
                .as_ref()
                .map(|hit| hit.distance)
                .unwrap_or(ray_t.end());
            let t = (ray_t.start()..tmax).into();

            if let Some(hit) = mesh.hit(ray, &t) {
                current_hit = Some(hit);
            }
        }

        current_hit
    }

    fn bounding_box(&self) -> &Aabb {
        &self.bbox
    }
}

#[derive(Clone)]
pub struct Sphere {
    initial_center: Vec3,
    center_delta: Vec3,
    radius: f32,
    material: Arc<dyn Material + Sync + Send>,
    bbox: Aabb,
}

impl Sphere {
    pub fn stationary(
        center: Vec3,
        radius: f32,
        material: Arc<dyn Material + Sync + Send>,
    ) -> Self {
        assert!(radius >= 0., "Radius cannot be less than 0.");

        let rvec = Vec3::new(radius, radius, radius);

        Self {
            initial_center: center,
            center_delta: Vec3::ZERO,
            radius,
            material,
            bbox: Aabb::from_extremes(center - rvec, center + rvec),
        }
    }

    pub fn moving(
        from: Vec3,
        to: Vec3,
        radius: f32,
        material: Arc<dyn Material + Sync + Send>,
    ) -> Self {
        assert!(radius >= 0., "Radius cannot be less than 0.");

        let rvec = Vec3::new(radius, radius, radius);
        let box1 = Aabb::from_extremes(from - rvec, from + rvec);
        let box2 = Aabb::from_extremes(to - rvec, to + rvec);
        let bbox = box1.merge(&box2);

        Self {
            initial_center: from,
            center_delta: to - from,
            radius,
            material,
            bbox,
        }
    }

    pub fn center(&self, time: f32) -> Vec3 {
        self.initial_center + self.center_delta * time
    }

    fn uv(&self, point: Vec3) -> Vec2 {
        let theta = (-point.y).acos();
        let phi = (-point.z).atan2(point.x) + PI;

        Vec2::new(phi / (2. * PI), theta / PI)
    }
}

impl Mesh for Sphere {
    fn hit(&self, ray: &Ray, ray_t: &Interval) -> Option<Hit> {
        let center = self.center(ray.time);
        let oc = center - ray.origin;
        let a = ray.direction.length_squared();
        let h = ray.direction.dot(oc);
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = h * h - a * c;
        if discriminant < 0. {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        let mut root = (h - sqrtd) / a;
        if !ray_t.contains(root) {
            root = (h + sqrtd) / a;
            if !ray_t.contains(root) {
                return None;
            }
        }

        let point = ray.get_point(root);
        let normal = (point - center) / self.radius;
        let uv = self.uv(normal);
        Some(Hit::new(ray, root, normal, self.material.clone(), uv))
    }

    fn bounding_box(&self) -> &Aabb {
        &self.bbox
    }
}

pub struct Quad {
    translation: Vec3,
    u: Vec3,
    v: Vec3,
    material: Arc<dyn Material + Sync + Send>,
    bbox: Aabb,
    normal: Vec3,
    d: f32,
    w: Vec3,
}

impl Quad {
    pub fn new(
        translation: Vec3,
        u: Vec3,
        v: Vec3,
        material: Arc<dyn Material + Sync + Send>,
    ) -> Self {
        let bbox_diagonal1 = Aabb::from_extremes(translation, translation + u + v);
        let bbox_diagonal2 = Aabb::from_extremes(translation + u, translation + v);

        let n = u.cross(v);
        let normal = n.normalize();

        Self {
            translation,
            u,
            v,
            material,
            bbox: bbox_diagonal1.merge(&bbox_diagonal2),
            normal,
            d: normal.dot(translation),
            w: n / n.dot(n),
        }
    }

    fn uv(&self, a: f32, b: f32) -> Option<Vec2> {
        let unit_interval = Interval::from(0.0..1.);

        if unit_interval.contains(a) && unit_interval.contains(b) {
            Some(Vec2::new(a, b))
        } else {
            None
        }
    }
}

impl Mesh for Quad {
    fn hit(&self, ray: &Ray, ray_t: &Interval) -> Option<Hit> {
        let denom = self.normal.dot(ray.direction);

        if denom.abs() < 1e-8 {
            return None;
        }

        let t = (self.d - self.normal.dot(ray.origin)) / denom;
        if !ray_t.contains(t) {
            return None;
        }

        let intersection = ray.get_point(t);
        let planar_hitpt_vec = intersection - self.translation;
        let alpha = self.w.dot(planar_hitpt_vec.cross(self.v));
        let beta = self.w.dot(self.u.cross(planar_hitpt_vec));
        let uv = self.uv(alpha, beta)?;

        Some(Hit::new(ray, t, self.normal, self.material.clone(), uv))
    }

    fn bounding_box(&self) -> &Aabb {
        &self.bbox
    }
}

pub struct Cube(World);

impl Cube {
    pub fn new(a: Vec3, b: Vec3, material: Arc<dyn Material + Sync + Send>) -> Self {
        let min = Vec3::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z));
        let max = Vec3::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z));

        let dx = Vec3::X * (max.x - min.x);
        let dy = Vec3::Y * (max.y - min.y);
        let dz = Vec3::Z * (max.z - min.z);

        let mut world = World::new();
        world.push(Quad::new(
            Vec3::new(min.x, min.y, max.z),
            dx,
            dy,
            material.clone(),
        ));
        world.push(Quad::new(
            Vec3::new(max.x, min.y, max.z),
            -dz,
            dy,
            material.clone(),
        ));
        world.push(Quad::new(
            Vec3::new(max.x, max.y, min.z),
            -dx,
            -dy,
            material.clone(),
        ));
        world.push(Quad::new(
            Vec3::new(min.x, min.y, min.z),
            dz,
            dy,
            material.clone(),
        ));
        world.push(Quad::new(
            Vec3::new(min.x, max.y, max.z),
            dx,
            -dz,
            material.clone(),
        ));
        world.push(Quad::new(
            Vec3::new(min.x, min.y, min.z),
            dx,
            dz,
            material.clone(),
        ));

        Self(world)
    }
}

impl Mesh for Cube {
    fn hit(&self, ray: &Ray, ray_t: &Interval) -> Option<Hit> {
        self.0.hit(ray, ray_t)
    }

    fn bounding_box(&self) -> &Aabb {
        &self.0.bbox
    }
}

pub struct Translate<T: Mesh> {
    bbox: Aabb,
    offset: Vec3,
    object: T,
}

impl<T: Mesh> Translate<T> {
    pub fn new(object: T, offset: Vec3) -> Self {
        Self {
            bbox: object.bounding_box() + offset,
            object,
            offset,
        }
    }
}

impl<T: Mesh> Mesh for Translate<T> {
    fn hit(&self, ray: &Ray, ray_t: &Interval) -> Option<Hit> {
        let ray = Ray::new(ray.origin - self.offset, ray.direction, ray.time);
        if let Some(mut hit) = self.object.hit(&ray, ray_t) {
            hit.point += self.offset;
            Some(hit)
        } else {
            None
        }
    }

    fn bounding_box(&self) -> &Aabb {
        &self.bbox
    }
}

pub struct RotateY<T: Mesh> {
    object: T,
    sin_theta: f32,
    cos_theta: f32,
    bbox: Aabb,
}

impl<T: Mesh> RotateY<T> {
    pub fn new(object: T, angle: f32) -> Self {
        let radians = degrees_to_radians(angle);
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();
        let bbox = object.bounding_box().clone();

        let mut min = Vec3::INFINITY;
        let mut max = Vec3::NEG_INFINITY;

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let x = i as f32 * bbox.x.end() + (1. - i as f32) * bbox.x.start();
                    let y = j as f32 * bbox.y.end() + (1. - j as f32) * bbox.y.start();
                    let z = k as f32 * bbox.z.end() + (1. - k as f32) * bbox.z.start();

                    let newx = cos_theta * x + sin_theta * z;
                    let newz = -sin_theta * x + cos_theta * z;

                    let tester = Vec3::new(newx, y, newz);

                    for c in 0..3 {
                        min[c] = min[c].min(tester[c]);
                        max[c] = max[c].max(tester[c]);
                    }
                }
            }
        }

        Self {
            object,
            sin_theta,
            cos_theta,
            bbox: Aabb::from_extremes(min, max),
        }
    }
}

impl<T: Mesh> Mesh for RotateY<T> {
    fn hit(&self, ray: &Ray, ray_t: &Interval) -> Option<Hit> {
        let mut origin = ray.origin;
        let mut direction = ray.direction;

        origin[0] = self.cos_theta * ray.origin[0] - self.sin_theta * ray.origin[2];
        origin[2] = self.sin_theta * ray.origin[0] + self.cos_theta * ray.origin[2];

        direction[0] = self.cos_theta * ray.direction[0] - self.sin_theta * ray.direction[2];
        direction[2] = self.sin_theta * ray.direction[0] + self.cos_theta * ray.direction[2];

        if let Some(mut hit) = self
            .object
            .hit(&Ray::new(origin, direction, ray.time), ray_t)
        {
            hit.point[0] = self.cos_theta * hit.point[0] - self.sin_theta * hit.point[2];
            hit.point[2] = self.sin_theta * hit.point[0] + self.cos_theta * hit.point[2];

            hit.normal[0] = self.cos_theta * hit.normal[0] - self.sin_theta * hit.normal[2];
            hit.normal[2] = self.sin_theta * hit.normal[0] + self.cos_theta * hit.normal[2];

            Some(hit)
        } else {
            None
        }
    }

    fn bounding_box(&self) -> &Aabb {
        &self.bbox
    }
}

#[derive(Default, Clone)]
pub struct Aabb {
    x: Interval,
    y: Interval,
    z: Interval,
}

impl Index<usize> for Aabb {
    type Output = Interval;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Invalid axis"),
        }
    }
}

impl Aabb {
    pub fn new(x: Interval, y: Interval, z: Interval) -> Self {
        Self {
            x: Self::pad_to_minimus(x),
            y: Self::pad_to_minimus(y),
            z: Self::pad_to_minimus(z),
        }
    }

    fn pad_to_minimus(interval: Interval) -> Interval {
        let delta = 0.0001;
        if interval.size() < delta {
            interval.expand(delta)
        } else {
            interval
        }
    }

    pub fn from_extremes(a: Vec3, b: Vec3) -> Self {
        Self::new(
            if a.x <= b.x {
                (a.x..b.x).into()
            } else {
                (b.x..a.x).into()
            },
            if a.y <= b.y {
                (a.y..b.y).into()
            } else {
                (b.y..a.y).into()
            },
            if a.z <= b.z {
                (a.z..b.z).into()
            } else {
                (b.z..a.z).into()
            },
        )
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self::new(
            self.x.join(&other.x),
            self.y.join(&other.y),
            self.z.join(&other.z),
        )
    }

    pub fn hit(&self, ray: &Ray, ray_t: &Interval) -> bool {
        let mut ray_t = ray_t.clone();

        for n in 0..3 {
            let interval = &self[n];
            let adinv = 1.0 / ray.direction[n];

            let (t0, t1) = {
                let t0 = (interval.start() - ray.origin[n]) * adinv;
                let t1 = (interval.end() - ray.origin[n]) * adinv;
                if t0 < t1 {
                    (t0, t1)
                } else {
                    (t1, t0)
                }
            };

            ray_t.set_start(ray_t.start().max(t0));
            ray_t.set_end(ray_t.end().min(t1));

            if ray_t.end() <= ray_t.start() {
                return false;
            }
        }

        true
    }

    pub fn longest_axis(&self) -> usize {
        let x = self.x.end() - self.x.start();
        let y = self.y.end() - self.y.start();
        let z = self.z.end() - self.z.start();
        if x > y && x > z {
            0
        } else if y > z {
            1
        } else {
            2
        }
    }
}

impl Add<Vec3> for &Aabb {
    type Output = Aabb;

    fn add(self, rhs: Vec3) -> Self::Output {
        Aabb {
            x: &self.x + rhs.x,
            y: &self.y + rhs.y,
            z: &self.z + rhs.z,
        }
    }
}

pub struct Bvh {
    left: Arc<dyn Mesh + Sync + Send>,
    right: Arc<dyn Mesh + Sync + Send>,
    bbox: Aabb,
}

impl<T> From<T> for Bvh
where
    T: AsRef<[Arc<dyn Mesh + Sync + Send>]>,
{
    fn from(value: T) -> Self {
        let value = value.as_ref();

        let bbox = value
            .iter()
            .fold(Aabb::default(), |acc, mesh| acc.merge(&mesh.bounding_box()));
        let longest_axis = bbox.longest_axis();

        if value.len() == 1 {
            Self {
                left: value[0].clone(),
                right: value[0].clone(),
                bbox,
            }
        } else if value.len() == 2 {
            Self {
                left: value[0].clone(),
                right: value[1].clone(),
                bbox,
            }
        } else {
            let mut value = Vec::from(value);

            value.sort_by(|a, b| {
                a.bounding_box()[longest_axis]
                    .start()
                    .total_cmp(&b.bounding_box()[longest_axis].start())
            });

            let mid = value.len() / 2;

            let left = Bvh::from(&value[..mid]);
            let right = Bvh::from(&value[mid..]);

            Self {
                bbox,
                left: Arc::new(left),
                right: Arc::new(right),
            }
        }
    }
}

impl Mesh for Bvh {
    fn hit(&self, ray: &Ray, ray_t: &Interval) -> Option<Hit> {
        if self.bbox.hit(ray, ray_t) {
            match self.left.hit(ray, ray_t) {
                Some(hit) => self
                    .right
                    .hit(ray, &(ray_t.start()..hit.distance).into())
                    .or(Some(hit)),
                None => self.right.hit(ray, ray_t),
            }
        } else {
            None
        }
    }

    fn bounding_box(&self) -> &Aabb {
        &self.bbox
    }
}

pub struct ConstantMedium<T: Mesh> {
    boundary: T,
    density: f32,
    material: Arc<dyn Material + Send + Sync>,
}

impl<T: Mesh> ConstantMedium<T> {
    pub fn new(boundary: T, density: f32, texture: Arc<dyn Texture + Send + Sync>) -> Self {
        Self {
            boundary,
            density: -1. / density,
            material: Isotropic::new(texture),
        }
    }

    pub fn from_color(boundary: T, density: f32, color: Color) -> Self {
        Self::new(boundary, density, SolidTexture::new(color))
    }
}

impl<T: Mesh> Mesh for ConstantMedium<T> {
    fn hit(&self, ray: &Ray, ray_t: &Interval) -> Option<Hit> {
        let mut hit1 = self.boundary.hit(ray, &Interval::UNIVERSE)?;
        let mut hit2 = self
            .boundary
            .hit(ray, &(hit1.distance + 0.0001..f32::INFINITY).into())?;

        if hit1.distance < ray_t.start() {
            hit1.distance = ray_t.start();
        }

        if hit2.distance > ray_t.end() {
            hit2.distance = ray_t.end();
        }

        if hit1.distance >= hit2.distance {
            return None;
        }

        if hit1.distance < 0. {
            hit1.distance = 0.;
        }

        let ray_length = ray.direction.length();
        let distance_inside_boundary = (hit2.distance - hit1.distance) * ray_length;
        let hit_distance = self.density * rand::random::<f32>().ln();

        if hit_distance > distance_inside_boundary {
            return None;
        }

        let distance = hit1.distance + hit_distance / ray_length;

        Some(Hit {
            point: ray.get_point(distance),
            distance,
            normal: Vec3::X,
            front_face: true,
            material: self.material.clone(),
            uv: Vec2::ZERO,
        })
    }

    fn bounding_box(&self) -> &Aabb {
        &self.boundary.bounding_box()
    }
}
