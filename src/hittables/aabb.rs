
use crate::ray::Ray;
use crate::vec3::Vec3;
use std::mem::swap;

//axis aligned bounding box
#[derive(Debug, Copy, Clone)]
pub struct AABB {
    start: Vec3, //start < end !!
    end: Vec3,   //end > start !!
}

impl AABB {
    pub fn new(start: Vec3, end: Vec3) -> Self {
        //start < end !
        if start.len_squared() > end.len_squared() {
            let temp = start;
            let _start = end;
            let _end = temp;
        }

        Self { start, end }
    }

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

    pub fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> bool {
        //calculate intersection on YZ-plane
        let mut t0_x = (self.start.x - ray.origin.x) / ray.direction.x;
        let mut t1_x = (self.end.x - ray.origin.x) / ray.direction.x;
        //if direction is negative, gotta swap because t1 is supposed to be the bigger one
        if ray.direction.x < 0.0 {
            swap(&mut t0_x, &mut t1_x);
        }

        //calculate intersection on XZ-plane
        let mut t0_y = (self.start.y - ray.origin.y) / ray.direction.y;
        let mut t1_y = (self.end.y - ray.origin.y) / ray.direction.y;
        //if direction is negative, gotta swap because t1 is supposed to be the bigger one
        if ray.direction.y < 0.0 {
            swap(&mut t0_y, &mut t1_y);
        }

        //calculate intersection on XY-plane
        let mut t0_z = (self.start.z - ray.origin.z) / ray.direction.z;
        let mut t1_z = (self.end.z - ray.origin.z) / ray.direction.z;
        //if direction is negative, gotta swap because t1 is supposed to be the bigger one
        if ray.direction.z < 0.0 {
            swap(&mut t0_z, &mut t1_z);
        }

        //limit our hit interval to x-hits
        //if we are completely inside the AABB, then no change occurs here
        //otherwise limit to found hits, to make sure subsequent hits overlap this one
        let t_min = if t0_x > t_min { t0_x } else { t_min };
        let t_max = if t1_x < t_max { t1_x } else { t_max };

        //limit hit interval to xy-hit
        let t_min = if t0_y > t_min { t0_y } else { t_min };
        let t_max = if t1_y < t_max { t1_y } else { t_max };

        //check if y-hit overlaps x-hit
        if t_max < t_min {
            return false;
        }

        //limit hit interval to xyz-hit and check if it overlaps xy-hit
        let t_min = if t0_z > t_min { t0_z } else { t_min };
        let t_max = if t1_z < t_max { t1_z } else { t_max };
        if t_max < t_min {
            return false;
        }

        true
    }
}
