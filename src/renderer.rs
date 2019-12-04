use sdl2::render::WindowCanvas;
use sdl2::pixels::Color;
use crate::hittable::{Hittable, HitResult};
use crate::camera::Camera;
use crate::vec3::Vec3;
use crate::light::Light;
use crate::ray::Ray;
use rand::Rng;

pub struct Renderer {
    canvas: WindowCanvas,
    camera: Camera,
    objects: Vec<Box<dyn Hittable>>,
    lights: Vec<Light>
}

impl Renderer {
    pub fn new(canvas : WindowCanvas) -> Self {
        let (width,height) = canvas.window().size();

        Renderer {
            canvas,
            camera: Camera::new(
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
                90.0,
                width,
                height),
            objects: Vec::new(),
            lights: Vec::new()
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

                for s in 0..4 { //multisample (4 samples)!
                    let ray = self.camera.get_ray(
                        x as f64 + rng.gen_range(0.0, 1.0),
                        y as f64 + rng.gen_range(0.0, 1.0));
                    final_color = final_color + self.trace_color(&ray, &self.objects);
                }

                final_color = final_color / 4.0;
                final_color.x = final_color.x.max(0.0).min(255.0);
                final_color.y = final_color.y.max(0.0).min(255.0);
                final_color.z = final_color.z.max(0.0).min(255.0);

                self.canvas.set_draw_color(Color::RGB(final_color.x as u8, final_color.y as u8, final_color.z as u8));
                self.canvas.draw_point((x, y));
            }
        }

        //show image
        self.canvas.present();
    }

    fn trace_color(&self, ray: &Ray, object : &dyn Hittable) -> Vec3 {
        if let Some(hit) = object.hit(ray, 0.0, std::f64::MAX) {
            //if no lights, display normals
            if self.lights.is_empty() {
                let r = (127.0*(hit.normal.x+1.0));
                let g = (127.0*(hit.normal.y+1.0));
                let b = (127.0*(hit.normal.z+1.0));
                return Vec3::new(r, g, b);
            }
            else {
                let mut rng = rand::thread_rng();
                let random_dir = Vec3::new( rng.gen_range(0.0, 1.0), rng.gen_range(0.0, 1.0), rng.gen_range(0.0, 1.0) );
                let target = hit.hit_position + hit.normal + random_dir;
                let bounce_ray = Ray { origin: hit.hit_position, direction: target - hit.hit_position };
                return 0.3 * self.trace_color(&bounce_ray, &self.objects);
            }
        }
        else {
            //background gradient
            let t = 0.5*(ray.direction.normalised().y + 1.0);
            return (1.0-t)*Vec3::new(255.0, 255.0, 255.0) + t*Vec3::new(100.0, 150.0, 255.0);
        }
    }

    fn calc_blinn_phong(&self, hit : &HitResult) -> Vec3 {
        let mut result = Vec3::new(25.0, 25.0, 25.0); //ambient light

        for light in &self.lights {
            let light_dir = light.position - hit.hit_position;

            //if object is in shadow, check next light
            let shadow_ray = Ray::new(hit.hit_position, light_dir);
            if self.objects.hit(&shadow_ray, 0.0, std::f64::MAX).is_some() {
                continue;
            }

            //diffuse
            let mut diffuse = hit.normal.dot(light_dir);
            if diffuse < 0.0 {
                diffuse = 0.0;
            }

            //specular (blinn)
            let view_dir = self.camera.position - hit.hit_position;
            let halfway = (view_dir + light_dir).normalised();
            let mut specular = halfway.dot(hit.normal);
            if specular < 0.0 {
                specular = 0.0;
            }
            else {
                specular = specular.powi(64);
            }

            result = result + diffuse * hit.base_color + specular * light.color;
        }

        Vec3::new(result.x, result.y, result.z)
    }
}