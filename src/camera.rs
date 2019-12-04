use crate::ray::Ray;
use crate::vec3::Vec3;

pub struct Camera {
    pub position: Vec3,
    direction: Vec3,
    focal_dist : f64,
    fov: f64,
    width : u32,
    height: u32
}

// FOV = 90°
//     width
//  *----X----*
//  \    |    /
//   \   |45°/
//    \ f|  /
//     \ | /
//      \|/
//       O
// => tan 45 = (width/2) / f
// tan 45 * f = width/2
// 2 * tan 45 * f = width
// f =  width / 2 * tan45

impl Camera {
    pub fn new(pos : Vec3, dir : Vec3, fov : f64, width : u32, height : u32) -> Self {
        Camera {
            position: pos,
            direction: dir.normalised(),
            focal_dist: (2.0 * (fov/2.0).to_radians().tan()) / width as f64, //inverted!
            fov,
            width,
            height
        }
    }

    pub fn get_ray(&self, x: f64, y: f64) -> Ray {
        let center = self.position + self.direction;

        let mut span_x = Vec3::new(1.0, 0.0, 0.0);

        if self.direction.z != 0.0 {
            let a = self.direction.x / self.direction.z;
            span_x.x = 1.0 / (1.0 + a * a).sqrt();
            span_x.z = (span_x.x * self.direction.x) / self.direction.z;
        }
        else if self.direction.x != 0.0 {
            span_x.x = 0.0;
            span_x.y = 0.0;
            span_x.z = 1.0;
        }

        let mut span_y = span_x.cross(self.direction);
        span_y = span_y.normalised();
        if span_y.y > 0.0 {
            span_y.y *= -1.0;
        }

        if self.direction.z < 0.0 {
            span_x.x *= -1.0;
        }

        span_x = span_x * self.focal_dist;
        span_y = span_y * self.focal_dist;

        let pixel_dir = center +
            (x - (self.width/2) as f64) * span_x +
            (y - (self.height/2) as f64) * span_y;

        Ray {
            origin: self.position,
            direction: pixel_dir
        }
    }

}