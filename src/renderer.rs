use rand::Rng;
use std::sync::Arc;

use crate::camera::Camera;
use crate::hittable::Hittable;
use crate::hittables::bvh::BvhNode;
use crate::ray::Ray;
use crate::vec3::Vec3;

pub struct Renderer {
    width: i32,
    height: i32,
    samples: u8,
    pub camera: Camera,
    objects: Vec<Arc<dyn Hittable>>,
}

impl Renderer {
    pub fn new(width: i32, height: i32, samples: u8) -> Self {
        let pos = Vec3::new(-5.0, 2.0, -3.0);
        let target = Vec3::new(0.0, 0.0, 3.0); //sphere center
        let dir = target - pos;

        Renderer {
            width,
            height,
            samples,
            camera: Camera::new(
                pos, dir, 90.0, //hfov
                width, height, 1.0, //if aperture = 0 ; focus dist is irrelevant
                0.0, //perfect camera => aperture = 0 ; => no DoF ; bigger aperture => stronger DoF
            ),
            objects: Vec::new(),
        }
    }

    pub fn add_object(&mut self, object: Arc<dyn Hittable>) {
        self.objects.push(object);
    }

    fn set_pixel(&self, buf: &mut [u8], x_in: i32, y_in: i32, color: Vec3) {
        //it seems, although the surface reports RGB888, it is actually BGRA8888

        //linear upscaling to double res
        // |X|O|X|O|   x => actual pixel
        // |O|O|O|O|   o => automatically set
        // |X|O|X|O|   -> 4 pixels per pixel -> doubled res
        // |O|O|O|O|

        //skip every second pixel because upscaling
        let x = x_in * 2;
        let y = y_in * 2;

        let x_stride = 4; //because 4 color values
        let y_stride = (self.width * 2) * x_stride; //because every width pixel has 4 color values

        const B: i32 = 0;
        const G: i32 = 1;
        const R: i32 = 2;
        const A: i32 = 3;

        let position = (x * x_stride) + (y * y_stride);
        let position_right = ((x + 1) * x_stride) + (y * y_stride);
        let position_below = (x * x_stride) + ((y + 1) * y_stride);
        let position_diagonal = ((x + 1) * x_stride) + ((y + 1) * y_stride);

        //actual pixel
        buf[(B + position) as usize] = color.z.min(255.0).max(0.0) as u8;
        buf[(G + position) as usize] = color.y.min(255.0).max(0.0) as u8;
        buf[(R + position) as usize] = color.x.min(255.0).max(0.0) as u8;
        buf[(A + position) as usize] = 0 as u8;

        //pixel right of it
        buf[(B + position_right) as usize] = color.z.min(255.0).max(0.0) as u8;
        buf[(G + position_right) as usize] = color.y.min(255.0).max(0.0) as u8;
        buf[(R + position_right) as usize] = color.x.min(255.0).max(0.0) as u8;
        buf[(A + position_right) as usize] = 0 as u8;

        //pixel below of it
        buf[(B + position_below) as usize] = color.z.min(255.0).max(0.0) as u8;
        buf[(G + position_below) as usize] = color.y.min(255.0).max(0.0) as u8;
        buf[(R + position_below) as usize] = color.x.min(255.0).max(0.0) as u8;
        buf[(A + position_below) as usize] = 0 as u8;

        //pixel below and right of it
        buf[(B + position_diagonal) as usize] = color.z.min(255.0).max(0.0) as u8;
        buf[(G + position_diagonal) as usize] = color.y.min(255.0).max(0.0) as u8;
        buf[(R + position_diagonal) as usize] = color.x.min(255.0).max(0.0) as u8;
        buf[(A + position_diagonal) as usize] = 0 as u8;
    }

    pub fn draw_image(&self, buf: &mut [u8], offset: usize) {
        // /2*width because line width (2* because upscaling), /4 because RGBA, /2 because upscaling
        let y_max = buf.len() / (2*self.width as usize) / 4 / 2;

        let offset = offset / 4; //RGBA
        let y_offset = (offset / (2*self.width as usize) / 2) as i32; // /2*width because line width (2* => upscaling), / 2 because upscaling
        let x_offset = (offset % (self.width as usize)) as i32; // (x % (2*width)) / 2 => x % width

        //draw image
        let mut rng = rand::thread_rng();

        let bvh = BvhNode::from_hittables(&self.objects[..]).unwrap();

        for x in 0..self.width {
            for y in 0..y_max as i32 {
                let mut final_color = Vec3::rgb(0, 0, 0);

                //multisample
                for _s in 0..self.samples {
                    let ray = self.camera.get_ray(
                        (x+x_offset) as f64 + rng.gen_range(0.0, 1.0),
                        (y+y_offset) as f64 + rng.gen_range(0.0, 1.0),
                    );

                    //*really* hacky, but what gives, BVH confirmed working
                    //final_color = bvh.debug_hit(&ray, 0.0001, std::f64::MAX);

                    //bvh slows us down in small example scenes!
                    final_color = final_color + self.trace_color(&ray, &bvh);
                }

                //normalize color after sampling a lot
                final_color = final_color / self.samples as f64;

                //scale up and gamma correct
                const GAMMA: f64 = 1.0 / 2.2;
                final_color.x = final_color.x.powf(GAMMA) * 255.0;
                final_color.y = final_color.y.powf(GAMMA) * 255.0;
                final_color.z = final_color.z.powf(GAMMA) * 255.0;

                self.set_pixel(buf, x, y, final_color);
            }
        }
    }

    fn trace_color(&self, ray: &Ray, object: &dyn Hittable) -> Vec3 {
        let mut ray_to_use = *ray;
        let mut final_attenuation = Vec3::new(1.0, 1.0, 1.0);
        while let Some(hit) = object.hit(&ray_to_use, 0.0001, std::f64::MAX) {
            if let Some((attenuation, scattered_ray)) = hit.material.scatter(&ray_to_use, &hit) {
                ray_to_use = scattered_ray;
                final_attenuation = final_attenuation * attenuation;
            } else {
                return Vec3::new(0.0, 0.0, 0.0);
            }
        }

        let t = 0.5 * (ray.direction.normalised().y + 1.0);
        return self.background_color(t) * final_attenuation;
    }

    fn background_color(&self, t: f64) -> Vec3 {
        (1.0 - t) * Vec3::rgb(255, 255, 255) + t * Vec3::rgb(128, 179, 255) //day
        //(1.0 - t) * Vec3::rgb(0, 0, 0) + t * Vec3::rgb(2, 4, 8) //night
    }
}
