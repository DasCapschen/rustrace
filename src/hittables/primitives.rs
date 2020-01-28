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
    /// the center (or position) of the plane
    pub center: Vec3,
    /// the first spanning vector
    pub span_a: Vec3,
    /// the second spanning vector
    pub span_b: Vec3,
    /// the material (color, etc) of the plane
    pub material: Arc<Material>,
}

impl Hit for Plane {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
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

        //TODO: calculate UV coordinates

        //from lower left corner to hit
        let llc = self.center - self.span_a - self.span_b;
        let relative_hit = hit_position - llc;

        let a_normalised = self.span_a.normalised();
        let b_normalised = self.span_b.normalised();

        //calculate only span_a / only span_b "component" of hit vector
        //if span_b was (0,4) and span_a was (7,0) and hit was (3, 2)
        //then, hit_on_span_a = (3,2) - (0,1) * 2 = (3,0)
        //then, hit_on_span_b = (3,2) - (1,0) * 3 = (0,2)
        // see vector_in_plane.ggb (geogebra)
        let hit_on_span_a = relative_hit - b_normalised * (relative_hit.dot(b_normalised));
        let hit_on_span_b = relative_hit - a_normalised * (relative_hit.dot(a_normalised));

        // 2.0* because it's relative to lower left corner, not center!
        // alpha and beta are in [0;1] if inside the plane
        let mut u = hit_on_span_a.len() / (2.0 * self.span_a.len());
        let mut v = hit_on_span_b.len() / (2.0 * self.span_b.len());

        let mut hit_outside_bounds = false;

        if u > 1.0 {
            u = u.fract();
            hit_outside_bounds = true;
        }
        if v > 1.0 {
            v = v.fract();
            hit_outside_bounds = true;
        }

        let result = HitResult {
            ray_param: parameter,
            hit_position,
            normal,
            material: Some(self.material.clone()),
            uv_coords: Some((u, v)),
        };

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

/// represents a triangle in 3d space
#[derive(Clone)]
pub struct Triangle {
    // +
    // ↑ \
    // b   \
    // |     \
    // *--a-->+
    // normal = a x b
    /// the position of a corner or the triangle
    pub center: Vec3,
    /// a vector pointing from center to another corner of the triangle
    pub span_a: Vec3,
    /// a vector pointing from center to another corner of the triangle
    pub span_b: Vec3,
    /// the material (color, etc) of the triangle
    pub material: Arc<Material>,
}

impl Hit for Triangle {
    fn hit(&self, _ray: &Ray, _t_min: f32, _t_max: f32) -> Option<HitResult> {
        None
    }

    fn bounding_box(&self) -> Option<AABB> {
        None
    }

    fn center(&self) -> Vec3 {
        self.center
    }
}
