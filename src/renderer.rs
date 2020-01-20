use rand::Rng;
use std::sync::Arc;

use crate::camera::Camera;
use crate::hit::Hit;
use crate::hittables::bvh::BvhNode;
use crate::ray::Ray;
use crate::vec3::Vec3;

#[derive(Debug, Clone)]
pub struct Renderer {
    width: i32,
    height: i32,
    samples: u8,
    pub camera: Camera,
    objects: Vec<Arc<dyn Hit>>,
}

impl Renderer {
    pub fn new(width: i32, height: i32, samples: u8) -> Self {
        let pos = Vec3::new(-7.0, 20.0, -7.0);
        let target = Vec3::new(7.5, 5.0, 7.5); //sphere center
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

    pub fn add_object(&mut self, object: Arc<dyn Hit>) {
        self.objects.push(object);
    }

    fn set_pixel(&self, buf: &mut [f32], x: i32, y: i32, color: Vec3) {
        let x_stride = 3; //because 3 color values
        let y_stride = self.width * x_stride; //because every width pixel has 3 color values

        const R: i32 = 0;
        const G: i32 = 1;
        const B: i32 = 2;

        let position = (x * x_stride) + (y * y_stride);

        buf[(R + position) as usize] = color.x.min(1.0).max(0.0) as f32;
        buf[(G + position) as usize] = color.y.min(1.0).max(0.0) as f32;
        buf[(B + position) as usize] = color.z.min(1.0).max(0.0) as f32;
    }

    pub fn draw_image(&self, buf: &mut [f32], offset: usize) {
        // /width because line width, /3 because RGB
        let y_max = buf.len() / self.width as usize / 3;

        let offset = offset / 3; //RGB
        let y_offset = (offset / self.width as usize) as i32; // /width because line width
        let x_offset = (offset % (self.width as usize)) as i32;

        //draw image
        let mut rng = rand::thread_rng();

        let bvh = BvhNode::from_hittables(&self.objects[..]).unwrap();

        for x in 0..self.width {
            for y in 0..y_max as i32 {
                let mut final_color = Vec3::rgb(0, 0, 0);

                //multisample
                for _s in 0..self.samples {
                    let ray = self.camera.get_ray(
                        (x + x_offset) as f64 + rng.gen_range(0.0, 1.0),
                        (y + y_offset) as f64 + rng.gen_range(0.0, 1.0),
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
                final_color.x = final_color.x.powf(GAMMA);
                final_color.y = final_color.y.powf(GAMMA);
                final_color.z = final_color.z.powf(GAMMA);

                self.set_pixel(buf, x, y, final_color);
            }
        }
    }

    fn trace_color(&self, ray: &Ray, object: &dyn Hit) -> Vec3 {
        let mut ray_to_use = *ray;
        let mut final_attenuation = Vec3::new(1.0, 1.0, 1.0);
        while let Some(hit) = object.hit(&ray_to_use, 0.0001, std::f64::MAX) {
            if let Some(mat) = &hit.material {
                if let Some((attenuation, scattered_ray)) = mat.scatter(&ray_to_use, &hit) {
                    ray_to_use = scattered_ray;
                    final_attenuation = final_attenuation * attenuation;
                    continue;
                }
            }
            //else
            return Vec3::new(0.0, 0.0, 0.0);
        }

        let t = 0.5 * (ray.direction.normalised().y + 1.0);
        return self.background_color(t) * final_attenuation;
    }

    fn background_color(&self, t: f64) -> Vec3 {
        (1.0 - t) * Vec3::rgb(255, 255, 255) + t * Vec3::rgb(128, 179, 255) //day
        //(1.0 - t) * Vec3::rgb(0, 0, 0) + t * Vec3::rgb(2, 4, 8)           //night
    }
}
