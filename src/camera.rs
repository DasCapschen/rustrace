use crate::ray::Ray;
use crate::vec3::Vec3;

pub struct Camera {
    pub position: Vec3,
    pub direction: Vec3,
    fov: f64,
    width: i32,
    height: i32,
}

impl Camera {
    pub fn new(position: Vec3, direction: Vec3, fov: f64, width: i32, height: i32) -> Self {
        Camera {
            position,
            direction: direction.normalised(),
            fov,
            width,
            height,
        }
    }


    pub fn get_ray(&self, x: f64, y: f64) -> Ray {
        const global_up: Vec3 = Vec3{x: 0.0, y: 1.0, z: 0.0};

        //yes, this is very verbose on purpose, I know it can be optimised
        //but tbh, the compiler probably does that for us

        //   ^ *-----X-----*  real_width
        //   |  \    |    /
        //   |   \   |   / 
        // 1 +    \--+--/ projected width
        //   |     \ | /
        //   |      \|/
        // 0 +       O
        // depth
        //
        // depth = 1
        // angle = 90° (fov)
        //
        // => tan(fov/2) = (real_width/2) / depth
        // tan 45 = width/2
        // 2 * tan 45 = width

        //width of our screen at focal distance
        let focal_width = 2.0*(self.fov/2.0).to_radians().tan();

        //figure out by how much we have to scale real_width and real_height to arrive at focal_width / focal_height
        let scale = focal_width / self.width as f64;

        //HINT: no need to scale by aspect ratio because x and y don't go between 0..1, but 0..width / 0..height!
        //else it would be:
        //aspect = height / width
        //height_scale = aspect * width_scale

        //calculate local coordinate system
        //let forward = (self.target - self.position).normalised();
        let forward = self.direction.normalised();
        let right = global_up.cross(forward).normalised() * scale;
        let up = forward.cross(right).normalised() * -scale; //negative because (0,0) is TOP right

        let center = self.position + forward;

        //position of the pixel on our "screen" in world space
        let pixel_pos = center
            + (x - (self.width / 2) as f64) * right //this is where real_width is scaled down to focal_width
            + (y - (self.height / 2) as f64) * up;

        //direction of the ray from us to pixel pos
        let pixel_dir = pixel_pos - self.position;

        Ray::new(self.position, pixel_dir)
    }
}
