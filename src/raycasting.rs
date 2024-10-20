use cgmath::dot;
use std::{ops::Range, sync::Arc};
use crate::materials::{Material,Metal};

pub type Color3 = cgmath::Vector3<f64>;
pub type Point3 = cgmath::Vector3<f64>;
pub type Vec3 = cgmath::Vector3<f64>;

pub struct Ray {
    pub orig: Point3,
    pub dir: Vec3,
}

pub struct Sphere {
    center: Point3,
    radius: f64,
    mat: Arc<dyn Material + Send + Sync>
}

pub struct Intersection {
    time: f64,
    pub position: Point3,
    pub norm: Vec3,
    pub mat: Arc<dyn Material + Send + Sync>,
    pub front_face: bool,
}

pub struct HittableList {
    objects: Vec<Arc<dyn Hittable + Send + Sync>>,
}

pub trait Hittable {
    fn hit(&self, r: &Ray, t_range: &Range<f64>, intersection: &mut Intersection) -> bool;
}

impl HittableList {
    pub fn new() -> Self {
        HittableList {
            objects: Vec::new(),
        }
    }

    pub fn add(&mut self, object: Arc<dyn Hittable + Send + Sync>) {
        self.objects.push(object);
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, t_range: &Range<f64>, intersection: &mut Intersection) -> bool {
        let mut hit_anything: bool = false;
        let mut range = t_range.clone();

        for object in &self.objects {
            if object.hit(r, &range, intersection) {
                hit_anything = true;
                range.end = intersection.time;
            }
        }

        hit_anything
    }
}

impl Intersection {
    pub fn new() -> Self {
        Intersection {
            time: 0.0,
            position: Point3::new(0.0, 0.0, 0.0),
            norm: Vec3::new(0.0, 0.0, 0.0),
            mat: Arc::new(Metal::new(Color3::new(0.0, 0.0, 0.0), 0.0)),
            front_face: false,
        }
    }

    fn set_face_normal(&mut self, r: &Ray, outward_norm: Vec3) {
        self.front_face = dot(r.dir, outward_norm) < 0.0;
        if !self.front_face {
            self.norm = -outward_norm;
        }
    }
}

impl Ray {
    pub fn new(orig: Point3, dir: Vec3) -> Self {
        Ray { orig, dir }
    }

    pub fn at(&self, t: f64) -> Point3 {
        self.orig + self.dir * t
    }
}

impl Sphere {
    pub fn new(pos: Point3, rad: f64, mat: Arc<dyn Material + Send + Sync>) -> Self {
        Sphere {
            center: pos,
            radius: rad,
            mat: mat,
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, t_range: &Range<f64>, intersection: &mut Intersection) -> bool {
        let oc = self.center - r.orig;
        let (a, h, c) = (
            dot(r.dir, r.dir),
            dot(r.dir, oc),
            dot(oc, oc) - self.radius * self.radius
        );
        let discriminant = h * h - a * c;

        if discriminant < 0.0 {
            return false;
        }

        let sqrtd = discriminant.sqrt();
        
        let mut time = (h - sqrtd) / a;
        if time <= t_range.start || time >= t_range.end {
            time = (h + sqrtd) / a;
            if time <= t_range.start || time >= t_range.end {
                return false;
            }
        }

        intersection.time = time;
        intersection.position = r.at(time);
        intersection.norm = (intersection.position - self.center) / self.radius;
        intersection.set_face_normal(r, intersection.norm);
        intersection.mat = Arc::clone(&self.mat);

        true
    }
}
