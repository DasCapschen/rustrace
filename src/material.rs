use std::fmt::Debug;

use crate::hittable::HitResult;
use crate::ray::Ray;
use crate::vec3::Vec3;

#[derive(Debug, Copy, Clone)]
pub struct Material {
    albedo: Vec3,
    metallic: f64,
    roughness: f64,
}

impl Material {
    pub fn new(albedo: Vec3, metal: f64, roughness: f64) -> Self {
        Material {
            albedo,
            metallic: metal.max(0.0).min(1.0),
            roughness: roughness.max(0.0).min(1.0), //clamp() is unstable...
        }
    }

    /// Returns Option<Tuple (Attenuation, Scattered Ray)>
    pub fn scatter(&self, ray: &Ray, hit: &HitResult) -> Option<(Vec3, Ray)> {
        //lambertian path
        let target = hit.hit_position + hit.normal + Vec3::random_in_unit_sphere();
        let mut direction = target - hit.hit_position;

        //metallic path
        if self.metallic > 0.0 {
            let reflected = ray.direction.normalised().reflect(hit.normal)
                + self.roughness * Vec3::random_in_unit_sphere();

            //if, for some reason, we reflect *into* the object, absorb the ray
            if reflected.dot(hit.normal) <= 0.0 {
                return None;
            }

            direction = Vec3::lerp(direction, reflected, self.metallic);
        }

        //else, scatter it
        let scattered = Ray::new(hit.hit_position, direction);

        Some((self.albedo, scattered))
    }
}
