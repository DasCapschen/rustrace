use crate::hit::{Hit, HitResult};
use crate::math::vec3::Vec3;
use crate::ray::Ray;

/// Implements an Axis-Aligned Bounding-Box
#[derive(Debug, Copy, Clone)]
pub struct AABB {
    /// the "starting" point (lower left front corner) ; must be < end!
    start: Vec3,
    /// the "ending" point (upper right back corner) ; must be > start!
    end: Vec3,
}

#[derive(Debug, Copy, Clone)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl AABB {
    /// Creates a new AABB
    /// # Arguments
    /// * `start` - the lower left front corner of the box
    /// * `end` - the upper right back corner of the box
    pub fn new(mut start: Vec3, mut end: Vec3) -> Self {
        //start < end !
        /*if start.len_squared() > end.len_squared() {
            std::mem::swap(&mut start, &mut end);
        }*/

        let s = Vec3 {
            x: start.x.min(end.x),
            y: start.y.min(end.y),
            z: start.z.min(end.z)
        };
        let e = Vec3 {
            x: end.x.max(start.x),
            y: end.y.max(start.y),
            z: end.z.max(start.z)
        };

        Self { start: s, end: e }
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
        //calculate intersection on YZ-plane
        //if direction.x is 0, because we're using floats, result is `inf`
        let t0 = (self.start.x - ray.origin.x) / ray.direction.x;
        let t1 = (self.end.x - ray.origin.x) / ray.direction.x;

        //limit tmin and tmax to the found interval.
        //if direction was negative, t0.min(t1) will swap the t's
        //note that Rusts impl of max/min NEVER returns NaN
        t_min = t_min.max(t0.min(t1));
        t_max = t_max.min(t1.max(t0));

        //calculate intersection on XZ-plane
        let t0 = (self.start.y - ray.origin.y) / ray.direction.y;
        let t1 = (self.end.y - ray.origin.y) / ray.direction.y;

        //limit to interval
        t_min = t_min.max(t0.min(t1));
        t_max = t_max.min(t1.max(t0));

        //calculate intersection on XY-plane
        let t0 = (self.start.z - ray.origin.z) / ray.direction.z;
        let t1 = (self.end.z - ray.origin.z) / ray.direction.z;

        //limit to interval
        t_min = t_min.max(t0.min(t1));
        t_max = t_max.min(t1.max(t0));

        //check if we actually hit.
        if t_max < t_min {
            return None;
        }

        Some(HitResult {
            ray_param: t_max, //return the BACK side!
            hit_position: ray.origin + t_max * ray.origin,
            normal: Vec3::new(0.0, 0.0, 0.0), //is this okay?
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
