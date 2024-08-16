use std::{ops::Index, sync::Arc};

use bevy_math::Vec3;

use crate::{material::Material, Interval, Ray};

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
}

impl Hit {
    pub fn new(ray: &Ray, distance: f32, normal: Vec3, material: Arc<dyn Material>) -> Self {
        let front_face = ray.direction.dot(normal) < 0.;
        let point = ray.get_point(distance);

        Self {
            point,
            normal: if front_face { normal } else { -normal },
            distance,
            front_face: true,
            material,
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
        Some(Hit::new(ray, root, normal, self.material.clone()))
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
        Self { x, y, z }
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
