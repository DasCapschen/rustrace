use std::clone::Clone;
use std::fmt::Debug;

use crate::hittable::{HitResult, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;

#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
    pub material: Box<dyn Material>,
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitResult> {
        // x² + y² + z² = radius  | sphere at 0,0,0
        // (x-c_x)² + (y-c_y)² + (z-c_z)² = radius | sphere at c
        // => (p-c)² == (p-c)·(p-c) = radius  | dot product!
        // use ray as p

        let oc = ray.origin - self.center;

        let a = ray.direction.dot(ray.direction);
        let b = oc.dot(ray.direction);
        let c = oc.dot(oc) - (self.radius * self.radius);

        let root = b * b - a * c;

        //cannot take sqrt of negative, no hit
        if root < 0.0 {
            return None;
        } else {
            let mut t = (-b + root.sqrt()) / (a);

            //if root is 0, only 1 hit (tangent on sphere), no need to check both
            if root != 0.0 {
                t = t.min((-b - root.sqrt()) / (a));
            }

            //if t is out of range, no hit
            if t > t_max || t < t_min {
                return None;
            }

            let p = ray.point_at(t);
            let n = (p - self.center).normalised();

            Some(HitResult {
                ray_param: t,
                hit_position: p,
                normal: n,
                material: self.material.clone(),
            })
        }
    }
}
