use rand::Rng;



use crate::camera::Camera;
use crate::hittable::Hittable;
use crate::light::Light;
use crate::ray::Ray;
use crate::vec3::Vec3;



pub struct Renderer {
    pixels: Vec<u8>,
    width: i32,
    height: i32,
    samples: u8,
    camera: Camera,
    objects: Vec<Box<dyn Hittable>>,
    lights: Vec<Light>,
}

impl Renderer {
    pub fn new(width: i32, height: i32, samples: u8) -> Self {
        Renderer {
            pixels: vec![0; (width * height * 4) as usize], // * 4 because R, G, B, A!
            width,
            height,
            samples,
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

    fn set_pixel(&mut self, x: i32, y: i32, color: Vec3) {
        self.pixels[(0 + 4 * x + y * self.width * 4) as usize] = color.x as u8;
        self.pixels[(1 + 4 * x + y * self.width * 4) as usize] = color.y as u8;
        self.pixels[(2 + 4 * x + y * self.width * 4) as usize] = color.z as u8;
        self.pixels[(3 + 4 * x + y * self.width * 4) as usize] = 1 as u8;
    }

    //TODO: Multithreading!
    //hint: rwlock for vecs etc.
    //also, move the Canvas OUT of the renderer!
    //return a [u8] of all pixels or something and set them to canvas elsewhere
    //FIXME: threading is just as fast, or even slower
    pub fn draw_image(&mut self) -> &[u8] {
        //draw image
        let mut rng = rand::thread_rng();

        for x in 0..self.width {
            for y in 0..self.height {
                let mut final_color = Vec3::rgb(0, 0, 0);

                //multisample
                for _s in 0..self.samples {
                    let ray = self.camera.get_ray(
                        x as f64 + rng.gen_range(0.0, 1.0),
                        y as f64 + rng.gen_range(0.0, 1.0),
                    );
                    //do expensive calculations
                    final_color = final_color + self.trace_color(&ray, &self.objects);
                }

                //normalize color after sampling a lot
                final_color = final_color / self.samples as f64;

                //scale up and gamma correct
                const GAMMA: f64 = 1.0 / 2.2;
                final_color.x = final_color.x.powf(GAMMA) * 255.0;
                final_color.y = final_color.y.powf(GAMMA) * 255.0;
                final_color.z = final_color.z.powf(GAMMA) * 255.0;

                self.set_pixel(x, y, final_color);
            }
        }

        &self.pixels[..]
    }

    fn trace_color(&self, ray: &Ray, object: &dyn Hittable) -> Vec3 {
        if let Some(hit) = object.hit(ray, 0.0001, std::f64::MAX) {
            //if no lights, display normals
            if self.lights.is_empty() {
                let r = 0.5 * (hit.normal.x + 1.0);
                let g = 0.5 * (hit.normal.y + 1.0);
                let b = 0.5 * (hit.normal.z + 1.0);
                return Vec3::new(r, g, b);
            } else {
                if let Some((attenuation, scattered_ray)) = hit.material.scatter(ray, &hit) {
                    //FIXME: possible unlimited recursion!
                    return attenuation * self.trace_color(&scattered_ray, object);
                }
                return Vec3::rgb(0, 0, 0);
            }
        } else {
            //background gradient
            let t = 0.6 * (ray.direction.normalised().y + 1.2);
            return (1.0 - t) * Vec3::rgb(255, 255, 255) + t * Vec3::rgb(100, 150, 255);
        }
    }
}
