use std::sync::Arc;

use crate::gfx::material::Material;
use crate::hittables::aabb::AABB;
use crate::math::vec3::Vec3;
use crate::ray::Ray;

#[derive(Clone)]
pub struct HitResult {
    pub ray_param: f32,
    pub hit_position: Vec3,
    pub normal: Vec3,
    pub material: Option<Arc<dyn Material>>,
    pub uv_coords: Option<(f32, f32)>,
}

pub trait Hit: Send + Sync {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult>;
    fn bounding_box(&self) -> Option<AABB>;
    fn center(&self) -> Vec3;
}

//hit a list of specific hittable
//useful for hitting triangles of a mesh
impl<T: Hit> Hit for Vec<T> {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
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
            for obj in &self[1..] {
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
        self.bounding_box().unwrap().center()
    }
}

/// simply calls Hit on the object in the Arc
impl Hit for Arc<dyn Hit> {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
        self.as_ref().hit(ray, t_min, t_max)
    }
    fn bounding_box(&self) -> Option<AABB> {
        self.as_ref().bounding_box()
    }
    fn center(&self) -> Vec3 {
        self.as_ref().center()
    }
}
