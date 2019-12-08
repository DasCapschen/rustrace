use std::fmt::{Debug};

use rand::Rng;

use crate::hittable::HitResult;
use crate::ray::Ray;
use crate::vec3::Vec3;

//following the tutorials way of making different classes for different materials
//I will probably later change this to a super-material with zeroed values

//the `: Debug + MaterialClone` forces all objects implementing Material to also implement Debug AND MaterialClone!
pub trait Material: Debug + MaterialClone {
    /// Returns Option<Tuple (Attenuation, Scattered Ray)>
    fn scatter(&self, ray: &Ray, hit: &HitResult) -> Option<(Vec3, Ray)>;
}

pub trait MaterialClone {
    fn clone_box(&self) -> Box<dyn Material>;
}
//blanket implementation
impl<T> MaterialClone for T
where
    T: Material + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn Material> {
        Box::new(self.clone())
    }
}

//urgh
impl Clone for Box<dyn Material> {
    fn clone(&self) -> Self {
        self.clone_box()
    }

    fn clone_from(&mut self, _source: &Self) {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
pub struct Lambertian {
    albedo: Vec3, //color
}
impl Lambertian {
    pub fn new(albedo: Vec3) -> Self {
        Lambertian {
            albedo: albedo / 255.0,
        }
    }
}
impl Material for Lambertian {
    fn scatter(&self, _ray: &Ray, hit: &HitResult) -> Option<(Vec3, Ray)> {
        let mut rng = rand::thread_rng();
        let random_dir = Vec3::new(
            rng.gen_range(0.0, 1.0),
            rng.gen_range(0.0, 1.0),
            rng.gen_range(0.0, 1.0),
        );

        let target = hit.hit_position + hit.normal + random_dir;
        let scattered = Ray {
            origin: hit.hit_position,
            direction: target - hit.hit_position,
        };

        Some((self.albedo, scattered))
    }
}

#[derive(Debug, Clone)]
pub struct Metal {
    albedo: Vec3,    //color
    smoothness: f64, // (1-smoothness) = fuzzyness
}
impl Metal {
    pub fn new(albedo: Vec3, smoothness: f64) -> Self {
        Metal {
            albedo: albedo / 255.0,
            smoothness: smoothness.min(1.0).max(0.0),
        }
    }
}
impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit: &HitResult) -> Option<(Vec3, Ray)> {
        let reflected = ray.direction.normalised().reflect(hit.normal);

        //if, for some reason, we reflect *into* the object, absorb the ray
        if reflected.dot(hit.normal) <= 0.0 {
            return None;
        }

        //else, scatter it
        let scattered = Ray {
            origin: hit.hit_position,
            direction: reflected,
        };
        Some((self.albedo, scattered))
    }
}
