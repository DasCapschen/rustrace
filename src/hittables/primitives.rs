use std::clone::Clone;
use std::fmt::Debug;

use crate::hittable::{HitResult, Hittable};
use crate::hittables::aabb::AABB;
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;

#[derive(Debug, Copy, Clone)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
    pub material: Material,
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

    fn bounding_box(&self) -> Option<AABB> {
        Some(AABB::new(
            self.center - Vec3::new(self.radius, self.radius, self.radius),
            self.center + Vec3::new(self.radius, self.radius, self.radius),
        ))
    }

    fn center(&self) -> Vec3 {
        self.center
    }
}

pub struct Plane {
    // +---------+
    // |    ↑    |
    // |   b|    |
    // |    *--->|
    // |       a |
    // |         |
    // +---------+
    // normal = a x b
    // width = 2 * |a|
    // height = 2 * |b|
    pub center: Vec3,
    pub span_a: Vec3,
    pub span_b: Vec3,
    pub infinite: bool,
    pub material: Material,
}

impl Hittable for Plane {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitResult> {
        // (x - center) · normal = 0
        // x => ray  x(t) = origin + t * direction
        // (origin + t * direction - center) · normal = 0
        // (origin - center) · normal + t * direction · normal = 0
        // t * direction · normal = - (origin - center) · normal
        // t = -((origin - center) · normal)/(direction · normal)

        //NORMALISE THE NORMAL!!
        let normal = self.span_a.cross(self.span_b).normalised();
        let parameter = -(ray.origin - self.center).dot(normal) / ray.direction.dot(normal);

        //no hit if outside [min, max]
        if parameter < t_min || parameter > t_max {
            return None;
        }

        let hit_position = ray.origin + parameter * ray.direction;

        let result = HitResult {
            ray_param: parameter,
            hit_position: hit_position,
            normal: normal,
            material: self.material,
        };

        //if not infinite plane, check if in bounds
        if !self.infinite {
            //from center to hit
            let relative_hit = hit_position - self.center;

            let a_normalised = self.span_a.normalised();
            let b_normalised = self.span_b.normalised();

            //calculate only span_a / only span_b "component" of hit vector
            //if span_b was (0,4) and span_a was (7,0) and hit was (3, 2)
            //then, hit_on_span_a = (3,2) - (0,1) * 2 = (3,0)
            //then, hit_on_span_b = (3,2) - (1,0) * 3 = (0,2)
            // see vector_in_plane.ggb (geogebra)
            let hit_on_span_a = relative_hit - b_normalised * (relative_hit.dot(b_normalised));
            let hit_on_span_b = relative_hit - a_normalised * (relative_hit.dot(a_normalised));

            //len squared saves us a sqrt() -> faster
            //also lets us handle + and - direction (because centered)
            let len_span_a = self.span_a.len_squared();
            let len_span_b = self.span_b.len_squared();

            //hit outside
            if hit_on_span_a.len_squared() > len_span_a || hit_on_span_b.len_squared() > len_span_b
            {
                return None;
            }
        }

        Some(result)
    }

    fn bounding_box(&self) -> Option<AABB> {
        Some(AABB::new(
            self.center - self.span_a - self.span_b,
            self.center + self.span_a + self.span_b,
        ))
    }

    fn center(&self) -> Vec3 {
        self.center
    }
}

pub struct Triangle {
    // +
    // ↑ \
    // b   \
    // |     \
    // *--a-->+
    // normal = a x b
    pub center: Vec3,
    pub span_a: Vec3,
    pub span_b: Vec3,
    pub material: Material,
}

impl Hittable for Triangle {
    fn hit(&self, _ray: &Ray, _t_min: f64, _t_max: f64) -> Option<HitResult> {
        None
    }

    fn bounding_box(&self) -> Option<AABB> {
        None
    }

    fn center(&self) -> Vec3 {
        self.center
    }
}
