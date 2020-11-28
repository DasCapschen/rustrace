use crate::gfx::material::Material;
use crate::gfx::texture::Texture;
use crate::hit::Hit;
use crate::hit::HitResult;
use crate::hittables::aabb::AABB;
use crate::math::vec3::Vec3;
use crate::ray::Ray;
use std::sync::Arc;

pub struct ConstantVolume {
    boundary: Arc<dyn Hit>,
    density: f32,
    material: Arc<dyn Material>,
}

impl ConstantVolume {
    pub fn new(boundary: Arc<dyn Hit>, density: f32, material: Arc<dyn Material>) -> Self {
        Self {
            boundary,
            density,
            material,
        }
    }
}

impl Hit for ConstantVolume {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
        let t_min = std::f32::MIN;
        let t_max = std::f32::MAX;

        if let Some(hit1) = self.boundary.hit(ray, t_min, t_max) {
            let t_min = hit1.ray_param + 0.0001;
            if let Some(hit2) = self.boundary.hit(ray, t_min, t_max) {
                let t1 = hit1.ray_param.max(t_min);
                let t2 = hit2.ray_param.min(t_max);

                if t1 > t2 {
                    return None;
                }

                let t1 = hit1.ray_param.max(0.0);

                let distance = t2 - t1;
                let hit_distance = -(1.0 / self.density) * rand::random::<f32>().ln();

                if hit_distance < distance {
                    let ray_param = t1 + hit_distance;
                    return Some(HitResult {
                        ray_param,
                        hit_position: ray.point_at(ray_param),
                        normal: Vec3::new(0.0, 0.0, 0.0),
                        material: Some(self.material.clone()),
                        uv_coords: None,
                    });
                }
            }
        }

        None
    }
    fn center(&self) -> Vec3 {
        self.boundary.center()
    }
    fn bounding_box(&self) -> Option<AABB> {
        self.boundary.bounding_box()
    }
}

pub struct Isotropic {
    albedo: Arc<dyn Texture>,
}

impl Isotropic {
    pub fn new(albedo: Arc<dyn Texture>) -> Self {
        Self { albedo }
    }
}

impl Material for Isotropic {
    fn scattered(&self, ray: &Ray, hit: &HitResult) -> Option<(Vec3, Vec3, Ray, f32)> {
        let albedo = self.albedo.texture((0.0, 0.0));
        let normal = hit.normal;
        let scattered_ray = Ray::new(hit.hit_position, Vec3::random_in_unit_sphere());

        //1 over 4 pi, because generated randomly in unit sphere (area = 4pi)
        let pdf = 1.0 / (4.0 * std::f32::consts::PI);

        Some((albedo, normal, scattered_ray, pdf))
    }
    fn scattering_pdf(&self, ray: &Ray, hit: &HitResult, scattered_ray: &Ray) -> f32 {
        //1 over 4 pi, because chance to scatter was same in every direction
        1.0 / (4.0 * std::f32::consts::PI)
    }
}
