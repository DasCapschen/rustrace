use crate::hit::{Hit, HitResult};
use crate::math::vec3::Vec3;
use crate::ray::Ray;

/// Implements an Axis-Aligned Bounding-Box
#[derive(Debug, Copy, Clone)]
pub struct AABB {
    /// the "starting" point (lower left front corner) ; must be < end!
    pub start: Vec3,
    /// the "ending" point (upper right back corner) ; must be > start!
    pub end: Vec3,
}

#[derive(Debug, Copy, Clone)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl AABB {
    /// Creates a new AABB
    /// You MUST make sure that start < end!
    /// # Arguments
    /// * `start` - the lower left front corner of the box
    /// * `end` - the upper right back corner of the box
    pub fn new(start: Vec3, end: Vec3) -> Self {
        Self { start, end }
    }

    /// Returns a new AABB which surrounds both given AABBs
    pub fn surrounding_box(box1: &Self, box2: &Self) -> Self {
        Self {
            start: Vec3::new(
                box1.start.x.min(box2.start.x),
                box1.start.y.min(box2.start.y),
                box1.start.z.min(box2.start.z),
            ),
            end: Vec3::new(
                box1.end.x.max(box2.end.x),
                box1.end.y.max(box2.end.y),
                box1.end.z.max(box2.end.z),
            ),
        }
    }

    pub fn longest_axis(&self) -> Axis {
        let dim = self.end - self.start;

        if dim.x > dim.y && dim.x > dim.z {
            Axis::X
        } else if dim.y > dim.z {
            Axis::Y
        } else {
            Axis::Z
        }
    }
}

impl Hit for AABB {
    fn hit(&self, ray: &Ray, mut t_min: f32, mut t_max: f32) -> Option<HitResult> {
        //instead of dividing by direction, multiply by its inverse
        let inverse_dx = 1.0 / ray.direction.x;
        let inverse_dy = 1.0 / ray.direction.y;
        let inverse_dz = 1.0 / ray.direction.z;

        //calculate intersection on YZ-plane
        //if direction.x is 0, because we're using floats, result is `inf`
        let t0 = (self.start.x - ray.origin.x) * inverse_dx;
        let t1 = (self.end.x - ray.origin.x) * inverse_dx;

        //limit tmin and tmax to the found interval.
        //if direction was negative, t0.min(t1) will swap the t's
        //note that Rusts impl of max/min NEVER returns NaN
        t_min = t_min.max(t0.min(t1));
        t_max = t_max.min(t1.max(t0));

        //calculate intersection on XZ-plane
        let t0 = (self.start.y - ray.origin.y) * inverse_dy;
        let t1 = (self.end.y - ray.origin.y) * inverse_dy;

        //limit to interval
        t_min = t_min.max(t0.min(t1));
        t_max = t_max.min(t1.max(t0));

        //calculate intersection on XY-plane
        let t0 = (self.start.z - ray.origin.z) * inverse_dz;
        let t1 = (self.end.z - ray.origin.z) * inverse_dz;

        //limit to interval
        t_min = t_min.max(t0.min(t1));
        t_max = t_max.min(t1.max(t0));

        //check if we actually hit.
        if t_max < t_min {
            return None;
        }

        Some(HitResult {
            ray_param: t_min,                  //front hit
            hit_position: ray.point_at(t_max), //back hit
            normal: Vec3::new(0.0, 0.0, 0.0),  //is this okay?
            material: None,
            uv_coords: None,
        })
    }

    fn bounding_box(&self) -> Option<AABB> {
        Some(*self)
    }

    fn center(&self) -> Vec3 {
        self.start + 0.5 * (self.end - self.start)
    }
}
