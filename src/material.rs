use crate::texture::Texture;
use std::sync::Arc;
use rand::Rng;
use std::fmt::Debug;

use crate::hit::HitResult;
use crate::ray::Ray;
use crate::vec3::Vec3;

//TODO: Normal Maps

#[derive(Clone)]
pub struct Material {
    albedo: Arc<dyn Texture>,
    metallic: Metallic,
    refraction: Option<f64>,
}

#[derive(Clone)]
pub enum Metallic {
    Metal(MetalParameters),
    NonMetal
}

#[derive(Clone)]
pub struct MetalParameters {
    pub metallic: Arc<dyn Texture>,
    pub roughness: Arc<dyn Texture>,
}

impl Material {
    pub fn new(albedo: Arc<dyn Texture>, metallic: Metallic, refraction: Option<f64>) -> Self {
        Material {
            albedo,
            metallic,
            refraction,
        }
    }

    /// Returns Option<Tuple (Attenuation, Scattered Ray)>
    pub fn scatter(&self, ray: &Ray, hit: &HitResult) -> Option<(Vec3, Ray)> {
        let uv_coords = hit.uv_coords.unwrap();

        //lambertian path
        let target = hit.hit_position + hit.normal + Vec3::random_in_unit_sphere();
        let mut direction = target - hit.hit_position;
        let albedo = self.albedo.texture(uv_coords);

        use Metallic::Metal;

        //metallic path
        if let Metal(metal_params) = &self.metallic {
            //.x => red channel ; this texture should be grayscale !
            //idea: combine 3 gray textures into 1 with r, g, b channels?
            let roughness = metal_params.roughness.texture(uv_coords).x;

            let reflected = ray.direction.normalised().reflect(hit.normal)
                + roughness * Vec3::random_in_unit_sphere();

            //if, for some reason, we reflect *into* the object, absorb the ray
            //tutorial says this is correct, but leads to black spots around the edge of the sphere :/
            if reflected.dot(hit.normal) < 0.0 {
                return None;
            }

            //.x => red channel ; this texture should be grayscale !
            let metallic = metal_params.metallic.texture(uv_coords).x;
            direction = Vec3::lerp(direction, reflected, metallic);
        }

        //refraction path
        if let Some(refraction_index) = self.refraction {
            let (normal, n_in, n_out, cosine);
            if ray.direction.dot(hit.normal) > 0.0 {
                //object -> air
                normal = -hit.normal; //outward normal
                n_in = refraction_index; //object
                n_out = 1.0; //air
                cosine = refraction_index * ray.direction.normalised().dot(hit.normal);
            // why refraction * v·n ?
            } else {
                //air -> object
                normal = hit.normal;
                n_in = 1.0;
                n_out = refraction_index;
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

        //return final
        Some((albedo, scattered))
    }

    fn schlick(&self, cosine: f64) -> f64 {
        //safe, we don't call this function if we have no refraction
        let refraction = self.refraction.unwrap();

        let mut r0 = (1.0 - refraction) / (1.0 + refraction);
        r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}
