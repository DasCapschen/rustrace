use crate::hittable::{HitResult, Hittable};
use crate::ray::Ray;

//axis aligned bounding box
struct AABB {

}

impl Hittable for AABB {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitResult> {
        None
    }
}