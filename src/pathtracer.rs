use crate::gfx::texture::Texture;
use rand::Rng;
use std::sync::Arc;

use crate::camera::Camera;
use crate::hit::Hit;
use crate::hittables::bvh::BvhTree;
use crate::math::vec3::Vec3;
use crate::ray::Ray;

#[derive(Clone)]
pub struct PathTracer {
    width: u32,
    height: u32,
    samples: u32,
    pub camera: Camera,
    objects: Vec<Arc<dyn Hit>>,
    sky: Arc<dyn Texture>,
    pub bvh: Option<BvhTree<Arc<dyn Hit>>>,
    pub debug_index: Option<usize>,
}

impl PathTracer {
    pub fn new(
        width: u32,
        height: u32,
        samples: u32,
        camera: Camera,
        sky: Arc<dyn Texture>,
    ) -> Self {
        PathTracer {
            width,
            height,
            samples,
            camera,
            objects: Vec::new(),
            sky,
            bvh: None,
            debug_index: None,
        }
    }

    pub fn add_object(&mut self, object: Arc<dyn Hit>) {
        self.objects.push(object);
    }

    //noinspection RsBorrowChecker
    //TODO: make it so that finalise leaves renderer immutable?
    //-> builder pattern?
    pub fn finalise(mut self) -> Self {
        //build the bvh from our objects (MOVED!!!)
        self.bvh = Some(BvhTree::from_hittables(self.objects));

        //replace moved value with new empty value
        self.objects = vec![];
        self
    }

    fn set_pixel(&self, buf: &mut [f32], x: u32, y: u32, color: Vec3) {
        let x_stride = 3; //because 3 color values
        let y_stride = self.width * x_stride; //because every width pixel has 3 color values

        const R: u32 = 0;
        const G: u32 = 1;
        const B: u32 = 2;

        let position = (x * x_stride) + (y * y_stride);

        buf[(R + position) as usize] = color.x.min(1.0).max(0.0) as f32;
        buf[(G + position) as usize] = color.y.min(1.0).max(0.0) as f32;
        buf[(B + position) as usize] = color.z.min(1.0).max(0.0) as f32;
    }

    pub fn draw_image(
        &self,
        color_buf: &mut [f32],
        albedo_buf: &mut [f32],
        normal_buf: &mut [f32],
        depth_buf: &mut [f32],
        offset: usize,
    ) {
        // /width because line width, /3 because RGB
        let y_max = color_buf.len() / self.width as usize / 3;

        let offset = offset / 3; //RGB
        let y_offset = (offset / self.width as usize) as u32; // /width because line width
        let x_offset = (offset % (self.width as usize)) as u32;

        //draw image
        let mut rng = rand::thread_rng();

        let bvh = self.bvh.as_ref().expect("did not call finalise()!");

        for x in 0..self.width {
            for y in 0..y_max as u32 {
                let mut final_color = Vec3::rgb(0, 0, 0);
                let mut final_albedo = Vec3::rgb(0, 0, 0);
                let mut final_normal = Vec3::rgb(0, 0, 0);
                let mut final_depth = 0.0;

                //multisample
                for _ in 0..self.samples {
                    let ray = self.camera.get_ray(
                        (x + x_offset) as f32 + rng.gen_range(0.0, 1.0),
                        (y + y_offset) as f32 + rng.gen_range(0.0, 1.0),
                    );

                    let (color, albedo, normal, depth) = if let Some(idx) = self.debug_index {
                        bvh.debug_hit(idx, &ray, 0.0001, std::f32::MAX)
                    } else {
                        self.trace_color(&ray, bvh)
                    };

                    final_color += color;

                    //I am not sure if these should be sampled multiple times...
                    final_albedo += albedo;
                    final_normal += normal; //[-1,1]
                    final_depth += depth;
                }

                //normalize color after sampling a lot
                final_color /= self.samples as f32;
                final_albedo /= self.samples as f32;
                final_normal /= self.samples as f32;
                final_depth /= self.samples as f32;

                self.set_pixel(color_buf, x, y, final_color);
                self.set_pixel(albedo_buf, x, y, final_albedo);
                self.set_pixel(normal_buf, x, y, final_normal);
                self.set_pixel(
                    depth_buf,
                    x,
                    y,
                    Vec3::new(final_depth, final_depth, final_depth),
                );
            }
        }
    }

    /// # Return Value
    /// Returns Tuple of (Color, Albedo, Normal, Depth)
    fn trace_color(&self, ray: &Ray, object: &dyn Hit) -> (Vec3, Vec3, Vec3, f32) {
        // recursively, this was:
        // return emitted + attenuation * scattering_pdf() * trace_color() / pdf
        // -> e1 + a1 * s1 * (1/pdf1) * ( e2 + a2 * s2 * (1/pdf2) * (...) )
        // -> 1 * (...)
        // -> 0 + 1*e1 + (a1*s1*(1/pdf1))*e2 + (a1*s1*(1/pdf1))*(a2*s2*(1/pdf2)) ...
        // that's a sum!

        let mut ray_to_use = *ray;
        let mut final_attenuation = Vec3::new(1.0, 1.0, 1.0);

        let mut bounces: u32 = 0;
        const MAX_BOUNCES: u32 = 100;

        let mut out_color = Vec3::new(0.0, 0.0, 0.0);
        let mut out_albedo = None;
        let mut out_normal = None;
        let mut out_depth = None;

        while let Some(hit) = object.hit(&ray_to_use, 0.0001, std::f32::MAX) {
            if bounces > MAX_BOUNCES {
                break;
            }
            bounces += 1;

            let mat = hit
                .material
                .as_ref()
                .expect("How did you manage to not have a material?!");

            //emitted is even added if we do not scatter!
            let emitted = mat.emitted(&hit);
            out_color += final_attenuation * emitted;

            if let Some((albedo, normal, scattered_ray, pdf)) = mat.scattered(&ray_to_use, &hit) {
                let brdf = albedo * mat.scattering_pdf(&ray, &hit, &scattered_ray);
                final_attenuation *= brdf / pdf;
                ray_to_use = scattered_ray;

                if out_albedo.is_none() {
                    out_albedo = Some(albedo)
                }
                if out_normal.is_none() {
                    out_normal = Some(normal)
                }
                if out_depth.is_none() {
                    out_depth = Some(1.0 / hit.ray_param)
                } // x/0 = inf !
            }
        }

        //calculate uv coords from ray direction
        let x = ray_to_use.direction.x;
        let z = ray_to_use.direction.z;
        let u = 1.0 - ((z.atan2(x) + std::f32::consts::PI) / (2.0 * std::f32::consts::PI));

        //clamp to [-1, 1] just in case (asin might return nan)
        let y = -ray_to_use.direction.y.min(1.0).max(-1.0);
        let v = (y.asin() + std::f32::consts::FRAC_PI_2) / std::f32::consts::PI;

        let skycolor = self.sky.texture((u, v));

        if out_albedo.is_none() {
            out_albedo = Some(skycolor)
        }
        if out_normal.is_none() {
            out_normal = Some(-ray_to_use.direction)
        }
        if out_depth.is_none() {
            out_depth = Some(0.0)
        }

        out_color += skycolor * final_attenuation;
        (
            out_color,
            out_albedo.unwrap(),
            out_normal.unwrap(),
            out_depth.unwrap(),
        )
    }
}
