use crate::gfx::texture::Texture;
use rand::{prelude::ThreadRng, Rng};
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
    incremental: bool,
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
        mut samples: u32,
        incremental: bool,
        camera: Camera,
        sky: Arc<dyn Texture>,
    ) -> Self {
        if incremental {
            samples = 1;
        }

        PathTracer {
            width,
            height,
            samples,
            incremental,
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

    fn get_pixel(&self, buf: &[f32], x: u32, y: u32) -> Vec3 {
        let x_stride = 3;
        let y_stride = self.width * x_stride;

        let position = ((x * x_stride) + (y * y_stride)) as usize;

        Vec3::new(buf[0 + position], buf[1 + position], buf[2 + position])
    }

    pub fn render_pixel(
        &self,
        rng: &mut ThreadRng,
        index: usize,
        frame: u32,
        color_buf: &mut [f32],
        albedo_buf: &mut [f32],
        normal_buf: &mut [f32],
        depth_buf: &mut [f32],
    ) {
        let pixel = index as u32; //divided by 3 because RGB
        let x = pixel % self.width;
        let y = pixel / self.width; //is floored

        //draw image
        let bvh = self.bvh.as_ref().expect("did not call finalise()!");

        let mut final_color = Vec3::rgb(0, 0, 0);
        let mut final_albedo = Vec3::rgb(0, 0, 0);
        let mut final_normal = Vec3::rgb(0, 0, 0);
        let mut final_depth = 0.0;

        //multisample
        for _ in 0..self.samples {
            let ray = self.camera.get_ray(
                x as f32 + rng.gen_range(0.0, 1.0), //the rng is for random multisampling within the pixel
                y as f32 + rng.gen_range(0.0, 1.0),
            );

            let (color, albedo, normal, depth) = self.trace_color(&ray, bvh);

            final_color += color;
            final_albedo += albedo;
            final_normal += normal; //[-1,1]
            final_depth += depth;
        }

        //normalize color after sampling a lot
        final_color /= self.samples as f32;
        final_albedo /= self.samples as f32;
        final_normal /= self.samples as f32;
        final_depth /= self.samples as f32;

        if self.incremental {
            let k = 1.0 / frame as f32;
            let km1 = (frame - 1) as f32 / frame as f32;

            final_color =
                (Vec3::new(color_buf[0], color_buf[1], color_buf[2]) * km1) + (final_color * k);
            final_albedo =
                (Vec3::new(albedo_buf[0], albedo_buf[1], albedo_buf[2]) * km1) + (final_albedo * k);
            final_normal =
                (Vec3::new(normal_buf[0], normal_buf[1], normal_buf[2]) * km1) + (final_normal * k);
            final_depth =
                (Vec3::new(depth_buf[0], depth_buf[1], depth_buf[2]).x * km1) + (final_depth * k);
        }

        color_buf[0] = final_color.x;
        color_buf[1] = final_color.y;
        color_buf[2] = final_color.z;

        albedo_buf[0] = final_albedo.x;
        albedo_buf[1] = final_albedo.y;
        albedo_buf[2] = final_albedo.z;

        normal_buf[0] = final_normal.x;
        normal_buf[1] = final_normal.y;
        normal_buf[2] = final_normal.z;

        depth_buf[0] = final_depth;
        depth_buf[1] = final_depth;
        depth_buf[2] = final_depth;
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
