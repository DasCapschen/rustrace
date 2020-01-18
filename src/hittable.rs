use std::fmt::Debug;
use std::sync::Arc;

use crate::hittables::aabb::AABB;
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;

#[derive(Debug, Copy, Clone)]
pub struct HitResult {
    pub ray_param: f64,
    pub hit_position: Vec3,
    pub normal: Vec3,
    pub material: Material,
}

pub trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitResult>;
    fn bounding_box(&self) -> Option<AABB>;
}

/* make hittable cloneable
trait HittableClone {
    fn box_clone(&self) -> Box<dyn Hittable>;
}

impl<T> HittableClone for T
    where T: Hittable + 'static + Clone {
        fn box_clone(&self) -> Box<dyn Hittable> {
            Box::new(self.clone())
        }
}

impl Clone for Box<dyn Hittable> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}
*/

// hit a list of any hittables
// useful for hitting world (all objects) in renderer
impl Hittable for Vec<Arc<dyn Hittable>> {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitResult> {
        let mut closest = t_max;
        let mut result = None;

        for object in self {
            if let Some(hit) = object.hit(ray, t_min, closest) {
                closest = hit.ray_param;
                result = Some(hit);
            }
        }

        result
    }

    fn bounding_box(&self) -> Option<AABB> {
        //don't run if no objects!
        if self.is_empty() {
            return None;
        }

        //if only 1 object, return its bb
        if self.len() == 1 {
            return self[0].bounding_box();
        }
        //else...

        //get bounding box of first object
        if let Some(mut bb) = self.first().unwrap().bounding_box() {
            //now, for every other object, get its bounding box
            for obj in self {
                if let Some(bb2) = obj.bounding_box() {
                    //and then make a new bounding box containing both bounding boxes!
                    bb = AABB::surrounding_box(&bb, &bb2);
                } else {
                    //if something has no bounding box, we don't have a bounding box at all!
                    return None;
                }
            }

            //return bb containing all bbs.
            return Some(bb);
        } else {
            //if first object has no bb, no bb at all!
            return None;
        }
    }
}

impl Hittable for [Arc<dyn Hittable>] {
    fn hit(&self, _ray: &Ray, _t_min: f64, _t_max: f64) -> Option<HitResult> {
        None
    }

    fn bounding_box(&self) -> Option<AABB> {
        if self.len() == 0 {
            None
        } else if self.len() == 1 {
            self[0].bounding_box()
        } else {
            if let Some(mut bb) = self[0].bounding_box() {
                for h in &self[1..] {
                    if let Some(bb2) = h.bounding_box() {
                        bb = AABB::surrounding_box(&bb, &bb2);
                    } else {
                        return None;
                    }
                }
                return Some(bb);
            }
            None
        }
    }
}

//hit a list of specific hittable
//useful for hitting triangles of a mesh
impl<T> Hittable for Vec<T>
where
    T: Hittable,
{
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitResult> {
        let mut closest = t_max;
        let mut result = None;

        for object in self {
            if let Some(hit) = object.hit(ray, t_min, closest) {
                closest = hit.ray_param;
                result = Some(hit);
            }
        }

        result
    }

    fn bounding_box(&self) -> Option<AABB> {
        //don't run if no objects!
        if self.is_empty() {
            return None;
        }

        //if only 1 object, return its bb
        if self.len() == 1 {
            return self[0].bounding_box();
        }
        //else...

        //get bounding box of first object
        if let Some(mut bb) = self.first().unwrap().bounding_box() {
            //now, for every other object, get its bounding box
            for obj in self {
                if let Some(bb2) = obj.bounding_box() {
                    //and then make a new bounding box containing both bounding boxes!
                    bb = AABB::surrounding_box(&bb, &bb2);
                } else {
                    //if something has no bounding box, we don't have a bounding box at all!
                    return None;
                }
            }

            //return bb containing all bbs.
            return Some(bb);
        } else {
            //if first object has no bb, no bb at all!
            return None;
        }
    }
}
