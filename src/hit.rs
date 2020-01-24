use std::sync::Arc;

use crate::hittables::aabb::AABB;
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;

#[derive(Clone)]
pub struct HitResult {
    pub ray_param: f64,
    pub hit_position: Vec3,
    pub normal: Vec3,
    pub material: Option<Arc<Material>>,
    pub uv_coords: Option<(f64, f64)>,
}

pub trait Hit: Send + Sync {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitResult>;
    fn bounding_box(&self) -> Option<AABB>;
    fn center(&self) -> Vec3;
}

// hit a list of any hittables
// useful for hitting world (all objects) in renderer
impl Hit for Vec<Arc<dyn Hit>> {
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
            Some(bb)
        } else {
            //if first object has no bb, no bb at all!
            None
        }
    }

    fn center(&self) -> Vec3 {
        todo!()
    }
}

impl Hit for [Arc<dyn Hit>] {
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

    fn center(&self) -> Vec3 {
        todo!()
    }
}

//hit a list of specific hittable
//useful for hitting triangles of a mesh
impl<T> Hit for Vec<T>
where
    T: Hit,
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
            Some(bb)
        } else {
            //if first object has no bb, no bb at all!
            None
        }
    }

    fn center(&self) -> Vec3 {
        todo!()
    }
}
