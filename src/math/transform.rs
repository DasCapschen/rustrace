
use crate::hittables::aabb::AABB;
use crate::hit::HitResult;
use crate::ray::Ray;
use crate::math::quat::Quaternion;
use crate::math::vec3::Vec3;
use crate::hit::Hit;
use std::sync::Arc;

#[derive(Clone)]
pub struct Transform {
    object: Arc<dyn Hit>,
    pub position: Vec3,
    pub rotation: Quaternion,
    pub scale: f32,
}

impl Hit for Transform {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
        let transformed_ray = self.apply_inverse_transform(ray);
        match self.object.hit(&transformed_ray, t_min, t_max) {
            Some(hit) => {
                let transformed_hit = self.apply_transform(&transformed_ray, &hit);
                Some(transformed_hit)
            },
            _ => None,
        }
    }
    fn bounding_box(&self) -> Option<AABB> {
        //TODO: apply scale and rotation
        if let Some(bb) = self.object.bounding_box() {
            Some(AABB::new(
                bb.start + self.position,
                bb.end + self.position,
            ))
        }
        else {
            None
        }
    }
    fn center(&self) -> Vec3 {
        self.object.center() + self.position
    }
}

impl Transform {
    pub fn new(object: Arc<dyn Hit>, position: Vec3, rotation: Quaternion, scale: f32) -> Self {
        Self { object, position, rotation, scale }
    }

    //TODO: does not apply rotation or scale!
    fn apply_transform(&self, ray: &Ray, hit: &HitResult) -> HitResult {
        HitResult {
            ray_param: hit.ray_param,
            hit_position: hit.hit_position + self.position,
            normal: hit.normal,
            material: hit.material.clone(),
            uv_coords: hit.uv_coords,
        }
    }

    //TODO: does not apply rotation or scale!
    fn apply_inverse_transform(&self, ray: &Ray) -> Ray {
        Ray {
            origin: ray.origin - self.position,
            direction: ray.direction
        }
    }
}