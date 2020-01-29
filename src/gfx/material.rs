use crate::gfx::texture::Texture;

use std::sync::Arc;

use crate::hit::HitResult;
use crate::math::onb::ONB;
use crate::math::vec3::Vec3;
use crate::ray::Ray;

/*
TODO: refactor Material to trait?

TODO: refactor the "scatter" method, break it into subfunctions and implement it properly!
*/

/*
    http://www.codinglabs.net/article_physically_based_rendering.aspx
    http://www.codinglabs.net/article_physically_based_rendering_cook_torrance.aspx
    https://stackoverflow.com/questions/36401272/how-to-implement-a-metallic-workflow-in-pbr

    ior = index of refraction

    E = irradiance = Bestrahlungsstärke -> gesamte Leistung *eingehender* Energie auf eine Fläche
    L = radiance = Strahldichte -> abgegebene Strahlung (W/m²) -> wie viel Licht in welche Richtung abgegeben

    incidence = Einfallswinkel

    p / x = position wo Lichtstrahl eintrifft
    n = normalen vektor an p

    w_i = eingehender Lichtstrahl (umgekehrt, zeigt zu licht)
    w_o = ausgehender Lichtstrahl (zeigt zu auge)

    L_o = gesamte *ausgehende* spektrale Strahldichte
    L_i = *eingehende* Strahldichte
    L_e = emittierte Strahldichte (für Lichter)

    BRDF Funktion f_r => gewichtung, wie viel von jedem w_i tatsächlich in w_o reflektiert wird
        bsp) Spiegel: BRDF = 0 für alle w_i, außer wenn winkel(w_i) == winkel(w_o)

    dot(w_i, n) bzw. cos( theta_i ) => abschwächen mit einfallswinkel, weil lichtstrahl dann fläche bestrahlt und nicht nur punkt

    L_o = L_e + Integral( BRDF * L_i * dot(w_i, n), w_i )

    Integral => Summe aller unendlich kleinen Lichtstrahlen in einer Halbkugel an p mit n (irradiance!)

    we're doing lambert brdf by sending a bunch of rays in random directions after hit.
    lambert only gives us diffuse surfaces though :/

    we can use cook-terrance brdf, which combines lambert for diffuse and itself for specular.

    f_r = f_lambert * k_d + f_cookterrance * k_s
    where k_d = diffused radiance, k_s specularly reflected radiance
    k_d + k_s <= irradiance

*/

#[derive(Clone)]
pub struct Material {
    pub albedo: Arc<dyn Texture>,
    normalmap: Option<Arc<dyn Texture>>,
    metallic: Metallic,
    refraction: Option<f32>,
}

#[derive(Clone)]
pub enum Metallic {
    Metal(MetalParameters),
    NonMetal,
}

#[derive(Clone)]
pub struct MetalParameters {
    pub metallic: Arc<dyn Texture>,
    pub roughness: Arc<dyn Texture>,
}

impl Material {
    /// Constructs a new Material with the given options
    /// # Arguments
    /// * `albedo` - texture for the "color" of the object
    /// * `normalmap` - normalmap texture, or None if no normal is wished
    /// * `metallic` - whether the material represents a metal, or a nonmetal
    /// * `refraction` - the refractive index of the material, or None if not refracting
    pub fn new(
        albedo: Arc<dyn Texture>,
        normalmap: Option<Arc<dyn Texture>>,
        metallic: Metallic,
        refraction: Option<f32>,
    ) -> Self {
        Material {
            albedo,
            normalmap,
            metallic,
            refraction,
        }
    }

    pub fn emitted(&self) -> Vec3 {
        Vec3::new(0.0, 0.0, 0.0)
    }

    // s(direction) -> directional distribution when light scatters
    pub fn scattering_pdf(&self, _ray: &Ray, hit: &HitResult, scattered_ray: &Ray) -> f32 {
        let uv_coords = hit.uv_coords.unwrap();
        let normal = self.map_normal(hit.normal, uv_coords);

        // lambertian scattering pdf is cos(theta)/pi

        let cosine = normal.dot(scattered_ray.direction.normalised());
        if cosine < 0.0 {
            0.0
        } else {
            cosine / std::f32::consts::PI
        }
    }

    /// Returns Option<Tuple (Attenuation, Normal, Scattered Ray, PDF)>
    /// the pdf here is p(direction) ; the pdf of how we generate the random direction of the scattered ray
    /// this is the pdf we use to approximate the integral, while the scattering_pdf is like the BRDF
    pub fn scatter(&self, _ray: &Ray, hit: &HitResult) -> Option<(Vec3, Vec3, Ray, f32)> {
        let uv_coords = hit.uv_coords.unwrap();

        let normal = self.map_normal(hit.normal, uv_coords);

        //lambert
        //randomly choose a vector in hemisphere above hit with pdf cos(theta)/pi
        //(choosing in hemisphere would be 1/2pi)
        let direction = ONB::from_w(normal).to_local(Vec3::random_cosine_direction());
        let albedo = self.albedo.texture(uv_coords);

        //we generated the direction randomly with cos(t)/pi, so return that as our used pdf
        let pdf = normal.dot(direction) / std::f32::consts::PI;

        //metallic path
        /*
        if let Metal(metal_params) = &self.metallic {
            //.x => red channel ; this texture should be grayscale !
            //idea: combine 3 gray textures into 1 with r, g, b channels?
            let roughness = metal_params.roughness.texture(uv_coords).x;

            let reflected = ray.direction.normalised().reflect(normal)
                + roughness * Vec3::random_in_unit_sphere();

            //if, for some reason, we reflect *into* the object, absorb the ray
            //tutorial says this is correct, but leads to black spots around the edge of the sphere :/
            if reflected.dot(normal) < 0.0 {
                return None;
            }

            //.x => red channel ; this texture should be grayscale !
            let metallic = metal_params.metallic.texture(uv_coords).x;
            direction = Vec3::lerp(direction, reflected, metallic);
        }

        //refraction path
        if let Some(refraction_index) = self.refraction {
            let (refr_normal, n_in, n_out, cosine);
            if ray.direction.dot(normal) > 0.0 {
                //object -> air
                refr_normal = -normal; //outward normal
                n_in = refraction_index; //object
                n_out = 1.0; //air
                cosine = refraction_index * ray.direction.normalised().dot(normal);
            // why refraction * v·n ?
            } else {
                //air -> object
                refr_normal = normal;
                n_in = 1.0;
                n_out = refraction_index;
                cosine = -ray.direction.normalised().dot(normal); // why negative?
            }

            let p = rand::thread_rng().gen_range(0.0, 1.0);
            if p <= self.fresnel_schlick(cosine) {
                //total reflection might occur, in that case, don't refract!
                if let Some(d) = ray.direction.refract(refr_normal, n_in, n_out) {
                    direction = d;
                }
            }
        }*/

        //else, scatter it
        let epsilon = normal * 0.001;
        let scattered = Ray::new(hit.hit_position + epsilon, direction);

        //return final
        Some((albedo, normal, scattered, pdf))
    }

    fn map_normal(&self, normal: Vec3, uv_coords: (f32, f32)) -> Vec3 {
        //calculate new normal from actual normal and normalmap
        if let Some(normalmap) = &self.normalmap {
            // get image normal
            let img_normal = normalmap.texture(uv_coords);

            // scale to [-1,1]
            let img_normal = (2.0 * img_normal) - Vec3::new(1.0, 1.0, 1.0);

            // transform from tangent to world space
            ONB::from_w(normal).to_local(img_normal)
        } else {
            normal
        }
    }

    fn fresnel_schlick(&self, cosine: f32) -> f32 {
        //safe, we don't call this function if we have no refraction
        let refraction = self.refraction.unwrap();

        let mut r0 = (1.0 - refraction) / (1.0 + refraction);
        r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}
