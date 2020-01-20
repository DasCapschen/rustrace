use rand::Rng;
use std::fmt::Debug;

use crate::hit::HitResult;
use crate::ray::Ray;
use crate::vec3::Vec3;

#[derive(Debug, Copy, Clone)]
pub struct Material {
    albedo: Vec3,

    metallic: f64,
    roughness: f64,

    refraction: f64,
}

impl Material {
    pub fn new(albedo: Vec3, metal: f64, roughness: f64, refraction: f64) -> Self {
        Material {
            albedo,
            metallic: metal.max(0.0).min(1.0),
            roughness: roughness.max(0.0).min(1.0), //clamp() is unstable...
            refraction,                             //refraction index
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
            //tutorial says this is correct, but leads to black spots around the edge of the sphere :/
            if reflected.dot(hit.normal) < 0.0 {
                return None;
            }
            direction = Vec3::lerp(direction, reflected, self.metallic);
        }

        //refraction path
        if self.refraction > 0.0 {
            let (normal, n_in, n_out, cosine);
            if ray.direction.dot(hit.normal) > 0.0 {
                //object -> air
                normal = -hit.normal; //outward normal
                n_in = self.refraction; //object
                n_out = 1.0; //air
                cosine = self.refraction * ray.direction.normalised().dot(hit.normal);
            // why refraction * v·n ?
            } else {
                //air -> object
                normal = hit.normal;
                n_in = 1.0;
                n_out = self.refraction;
                cosine = -ray.direction.normalised().dot(hit.normal); // why negative?
            }

            let p = rand::thread_rng().gen_range(0.0, 1.0);
            if p <= self.schlick(cosine) {
                //total reflection might occur, in that case, don't refract!
                if let Some(d) = ray.direction.refract(normal, n_in, n_out) {
                    direction = d;
                }
            }
        }

        //else, scatter it
        let scattered = Ray::new(hit.hit_position, direction);

        Some((self.albedo, scattered))
    }

    fn schlick(&self, cosine: f64) -> f64 {
        let mut r0 = (1.0 - self.refraction) / (1.0 + self.refraction);
        r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}
