use rand::Rng;
use sdl2::pixels::Color;
use sdl2::render::WindowCanvas;

use crate::camera::Camera;
use crate::hittable::{Hittable};
use crate::light::Light;
use crate::ray::Ray;
use crate::vec3::Vec3;

pub struct Renderer {
    canvas: WindowCanvas,
    camera: Camera,
    objects: Vec<Box<dyn Hittable>>,
    lights: Vec<Light>,
}

impl Renderer {
    pub fn new(canvas: WindowCanvas) -> Self {
        let (width, height) = canvas.window().size();

        Renderer {
            canvas,
            camera: Camera::new(
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
                90.0,
                width,
                height,
            ),
            objects: Vec::new(),
            lights: Vec::new(),
        }
    }

    pub fn add_object(&mut self, object: Box<dyn Hittable>) {
        self.objects.push(object);
    }

    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    pub fn draw_image(&mut self) {
        let (width, height) = self.canvas.window().size();
        let mut rng = rand::thread_rng();

        //draw image
        for x in 0..width as i32 {
            for y in 0..height as i32 {
                let mut final_color = Vec3::new(0.0, 0.0, 0.0);

                for _s in 0..4 {
                    //multisample (4 samples)!
                    let ray = self.camera.get_ray(
                        x as f64 + rng.gen_range(0.0, 1.0),
                        y as f64 + rng.gen_range(0.0, 1.0),
                    );
                    final_color = final_color + self.trace_color(&ray, &self.objects);
                }

                final_color = final_color / 4.0;
                final_color.x = final_color.x.max(0.0).min(255.0);
                final_color.y = final_color.y.max(0.0).min(255.0);
                final_color.z = final_color.z.max(0.0).min(255.0);

                self.canvas.set_draw_color(Color::RGB(
                    final_color.x as u8,
                    final_color.y as u8,
                    final_color.z as u8,
                ));
                self.canvas.draw_point((x, y));
            }
        }

        //show image
        self.canvas.present();
    }

    fn trace_color(&self, ray: &Ray, object: &dyn Hittable) -> Vec3 {
        if let Some(hit) = object.hit(ray, 0.0001, std::f64::MAX) {
            //if no lights, display normals
            if self.lights.is_empty() {
                let r = 127.0 * (hit.normal.x + 1.0);
                let g = 127.0 * (hit.normal.y + 1.0);
                let b = 127.0 * (hit.normal.z + 1.0);
                return Vec3::new(r, g, b);
            } else {
                if let Some((attenuation, scattered_ray)) = hit.material.scatter(ray, &hit) {
                    //FIXME: possible unlimited recursion!
                    return attenuation * self.trace_color(&scattered_ray, object);
                }
                return Vec3::new(0.0, 0.0, 0.0);
            }
        } else {
            //background gradient
            let t = 0.5 * (ray.direction.normalised().y + 1.0);
            return (1.0 - t) * Vec3::new(255.0, 255.0, 255.0) + t * Vec3::new(100.0, 150.0, 255.0);
        }
    }
}
