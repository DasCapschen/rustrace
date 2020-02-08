
use crate::hittables::aabb::AABB;
use crate::hit::HitResult;
use crate::ray::Ray;
use crate::math::quat::Quaternion;
use crate::math::vec3::Vec3;
use crate::hit::Hit;
use std::sync::Arc;

// first scale
// then rotate
// lastly translate

#[derive(Clone)]
pub struct Transform {
    object: Arc<dyn Hit>,
    pub position: Vec3,
    pub rotation: Quaternion,
    pub scale: f32,
}

impl Hit for Transform {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
        //first apply inverse, because we transform "the world", not ourselves
        let transformed_ray = self.apply_inverse_transform(ray);
        match self.object.hit(&transformed_ray, t_min, t_max) {
            Some(hit) => {
                //undo the transformation to fix viewpoint
                let transformed_hit = self.apply_transform(&transformed_ray, &hit);
                Some(transformed_hit)
            },
            _ => None,
        }
    }
    fn bounding_box(&self) -> Option<AABB> {
        let bb = self.object.bounding_box()?;

        // p3/p7 +-------+ p2/p6
        //       |       |
        // p0/p4 +-------+ p1/p5

        let dimensions = bb.start - bb.end;
        let x = Vec3::new(1.0, 0.0, 0.0);
        let z = Vec3::new(0.0, 0.0, 1.0);
        let xz = Vec3::new(1.0, 0.0, 1.0);

        //find and rotate all 8 vertices of the AABB
        //they are now NO LONGER AXIS-ALIGNED!
        let p0 = self.rotation.rotate_vector(bb.start);
        let p1 = self.rotation.rotate_vector(bb.start + x * dimensions);
        let p2 = self.rotation.rotate_vector(bb.start + xz * dimensions);
        let p3 = self.rotation.rotate_vector(bb.start + z * dimensions);
        let p4 = self.rotation.rotate_vector(bb.end - xz * dimensions);
        let p5 = self.rotation.rotate_vector(bb.end - z * dimensions);
        let p6 = self.rotation.rotate_vector(bb.end);
        let p7 = self.rotation.rotate_vector(bb.end - x * dimensions);

        let points = [p0, p1, p2, p3, p4, p5, p6, p7];

        //find max values of all points
        let max_x = points.iter().max_by(|u,v| u.x.partial_cmp(&v.x).unwrap()).unwrap().x;
        let max_y = points.iter().max_by(|u,v| u.y.partial_cmp(&v.y).unwrap()).unwrap().y;
        let max_z = points.iter().max_by(|u,v| u.z.partial_cmp(&v.z).unwrap()).unwrap().z;

        //find min values of all points
        let min_x = points.iter().min_by(|u,v| u.x.partial_cmp(&v.x).unwrap()).unwrap().x;
        let min_y = points.iter().min_by(|u,v| u.y.partial_cmp(&v.y).unwrap()).unwrap().y;
        let min_z = points.iter().min_by(|u,v| u.z.partial_cmp(&v.z).unwrap()).unwrap().z;
       
        //get new axis aligned min and max coordinates
        let start = Vec3::new(min_x, min_y, min_z);
        let end = Vec3::new(max_x, max_y, max_z);

        //TODO: apply scale
        Some(AABB::new(
            start + self.position,
            end + self.position,
        ))
    }
    fn center(&self) -> Vec3 {
        //center does not change with scale or rotation
        self.object.center() + self.position
    }
}

impl Transform {
    pub fn new(object: Arc<dyn Hit>, position: Vec3, rotation: Quaternion, scale: f32) -> Self {
        Self { object, position, rotation, scale }
    }

    //TODO: does not apply scale!
    fn apply_transform(&self, ray: &Ray, hit: &HitResult) -> HitResult {
        HitResult {
            ray_param: hit.ray_param,
            hit_position: self.rotation.rotate_vector(hit.hit_position) + self.position,
            normal: self.rotation.rotate_vector(hit.normal),
            material: hit.material.clone(),
            uv_coords: hit.uv_coords,
        }
    }

    //TODO: does not apply scale!
    //invert direction AND ORDER OF OPERATIONS!
    //translate, rotate, scale
    fn apply_inverse_transform(&self, ray: &Ray) -> Ray {
        Ray {
            origin: self.rotation.unrotate_vector(ray.origin - self.position),
            direction: self.rotation.unrotate_vector(ray.direction)
        }
    }
}