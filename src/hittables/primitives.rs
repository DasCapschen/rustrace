use std::clone::Clone;

use std::sync::Arc;

use crate::gfx::material::Material;
use crate::hit::{Hit, HitResult};
use crate::hittables::aabb::AABB;
use crate::math::vec3::Vec3;
use crate::ray::Ray;

/*
TODO: refactor this to something like...

struct Object {
    position: Vec3,
    material: Arc<Material>,

    shape: Shape
}

impl Hit for Object {
    fn hit(...) {
        ray.origin - center; //move ray because shape has no position
        match self.shape {
            Sphere(s) => s.hit(...),
            ...
        }
    }
    ...
}

enum Shape {
    Sphere(Sphere),
    Plane(Plane),
    Triangle(Triangle),
}

struct Sphere{
    radius: f32
}

struct Plane {
    span_a: Vec3,
    span_b: Vec3,
    infinite: bool,
}

struct Triangle {
    span_a: Vec3,
    span_b: Vec3,
}

*/

/// Represents a Sphere in 3D space
#[derive(Clone)]
pub struct Sphere {
    /// the center (or position) of the sphere
    pub center: Vec3,
    /// the radius of the sphere
    pub radius: f32,
    /// the material (color, etc) of the sphere
    pub material: Arc<Material>,
}

impl Hit for Sphere {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
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
            None
        } else {
            //check smaller t first, but if its out of range, check bigger t
            let mut t = (-b - root.sqrt()) / (a);
            if t > t_max || t < t_min {
                t = (-b + root.sqrt()) / (a);
            }

            //if t is out of range, no hit
            if t > t_max || t < t_min {
                return None;
            }

            let hit_position = ray.point_at(t);

            //divide by radius instead of .normalise() => can invert normals with negative radius
            let normal = (hit_position - self.center) / self.radius;

            let u = 1.0
                - ((normal.z.atan2(normal.x) + std::f32::consts::PI)
                    / (2.0 * std::f32::consts::PI));

            //negative because our y axis (image) is flipped
            let v = ((-normal.y).asin() + std::f32::consts::FRAC_PI_2) / std::f32::consts::PI;

            Some(HitResult {
                ray_param: t,
                hit_position,
                normal,
                material: Some(self.material.clone()),
                uv_coords: Some((u, v)),
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

/// represents a flat plane in 3d space
/// infinite planes no longer work after introduction of BVH
#[derive(Clone)]
pub struct Plane {
    // +------+
    // ↑ \    |
    // b   \  |
    // |     \|
    // *--a-->+
    // normal = a x b
    // width = 2 * |a|
    // height = 2 * |b|
    /// the center (or position) of the plane
    pub llc: Vec3,
    /// the first spanning vector
    pub span_a: Vec3,
    /// the second spanning vector
    pub span_b: Vec3,
    /// the material (color, etc) of the plane
    pub material: Arc<Material>,
    /// whether or not this plane is a triangle
    pub triangle: bool,
}

impl Hit for Plane {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
        // (x - llc) · normal = 0
        // x => ray(t) = origin + t * direction
        // (origin + t * direction - llc) · normal = 0
        // ((origin - llc) · normal) + (t * direction · normal) = 0
        // t * direction · normal = - (origin - llc) · normal
        // t = -((origin - llc) · normal)/(direction · normal)

        //NORMALISE THE NORMAL!!
        let normal = self.span_a.cross(self.span_b).normalised();
        let parameter = -(ray.origin - self.llc).dot(normal) / ray.direction.dot(normal);

        //no hit if outside [min, max]
        if parameter < t_min || parameter > t_max {
            return None;
        }

        let hit_position = ray.origin + parameter * ray.direction;

        //from lower left corner to hit
        let relative_hit = hit_position - self.llc;


        //the way we used to do it here was the right idea, but not fully correct
        //it failed for non-orthogonal spanning vectors!
        // http://geomalgorithms.com/a06-_intersect-2.html

        let ada = self.span_a.dot(self.span_a);
        let bdb = self.span_b.dot(self.span_b);
        let rda = relative_hit.dot(self.span_a);
        let rdb = relative_hit.dot(self.span_b);
        let adb = self.span_a.dot(self.span_b);

        let denom = 1.0 / ((adb * adb) - (ada * bdb));

        let u = ((adb * rdb) - (bdb * rda)) * denom;
        let v = ((adb * rda) - (ada * rdb)) * denom;

        // u, v must be positive, smaller 1, and if a triangle, their sum must by < 1 too
        if u < 0.0 || u > 1.0 
        || v < 0.0 || v > 1.0 
        || (self.triangle && (u+v) > 1.0) {
            None
        }
        else {
            Some(HitResult {
                ray_param: parameter,
                hit_position,
                normal,
                material: Some(self.material.clone()),
                uv_coords: Some((u, v)),
            })
        }
    }

    fn bounding_box(&self) -> Option<AABB> {
        //give the bb some height!
        let normal = self.span_a.cross(self.span_b);
        let epsilon = normal;

        Some(AABB::new(
            self.llc - epsilon,
            self.llc + self.span_a + self.span_b + epsilon,
        ))
    }

    fn center(&self) -> Vec3 {
        self.llc
    }
}
