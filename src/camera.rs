use crate::math::vec3::Vec3;
use crate::ray::Ray;

/// implements a camera from which to render from
#[derive(Debug, Copy, Clone)]
pub struct Camera {
    /// position in 3d space
    pub position: Vec3,
    /// the direction the camera is looking in (normalised)
    pub direction: Vec3,
    /// the horizontal field of view
    fov: f32,
    /// the width of the rendered image
    width: i32,
    /// the height of the rendered image
    height: i32,
    /// the distance at which the camera focuses (only if aperture > 0)
    focus_dist: f32,
    /// the aperture of the camera, bigger number leads to more "depth of field" (blurryness)
    aperture: f32,
}

impl Camera {
    /// Returns a new Camera with the specified parameters
    /// # Arguments
    /// * `position` - position in 3d space
    /// * `direction` - the direction the camera is looking in
    /// * `width` - the width of the rendered image
    /// * `height` - the height of the rendered image
    /// * `focus_dist` - the distance at which the camera focuses (only if aperture > 0)
    /// * `aperture` - the aperture of the camera, bigger number leads to more "depth of field" (blurryness)
    pub fn new(
        position: Vec3,
        direction: Vec3,
        fov: f32,
        width: i32,
        height: i32,
        focus_dist: f32,
        aperture: f32,
    ) -> Self {
        Camera {
            position,
            direction: direction.normalised(),
            fov,
            width,
            height,
            focus_dist,
            aperture,
        }
    }

    /// returns the forward vector (direction the camera is looking)
    pub fn forward(&self) -> Vec3 {
        self.direction.normalised()
    }

    /// returns the vector pointing to the right of the cameras look-direction
    pub fn right(&self) -> Vec3 {
        const GLOBAL_UP: Vec3 = Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        };

        GLOBAL_UP.cross(self.forward()).normalised()
    }

    /// returns the vector pointing upwards of the cameras look-direction
    pub fn up(&self) -> Vec3 {
        self.forward().cross(self.right()).normalised()
    }

    /// gets a new ray from the camera at the screen coordinates x and y
    pub fn get_ray(&self, x: f32, y: f32) -> Ray {
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
        let focal_width = 2.0 * (self.fov / 2.0).to_radians().tan() * self.focus_dist;

        //figure out by how much we have to scale real_width and real_height to arrive at focal_width / focal_height
        let scale = focal_width / self.width as f32;

        //HINT: no need to scale by aspect ratio because x and y don't go between 0..1, but 0..width / 0..height!
        //else it would be:
        //aspect = height / width
        //height_scale = aspect * width_scale

        //calculate local coordinate system
        //let forward = (self.target - self.position).normalised();
        let forward = self.forward();
        let right = self.right() * scale;
        let up = self.up() * -scale; //negative because (0,0) is TOP right

        let center = self.position + forward * self.focus_dist; //focus_dist -> move focus plane (Z, depth)

        //position of the pixel on our "screen" in world space
        let pixel_pos = center
            + (x - (self.width / 2) as f32) * right //this is where real_width is scaled down to focal_width
            + (y - (self.height / 2) as f32) * up;

        let lens_pos = Vec3::random_in_unit_disk() * (self.aperture / 2.0); //aperture/2 == lens radius
        let start = self.position + lens_pos; //start ray at random point in "lens"

        //direction of the ray from us to pixel pos
        let pixel_dir = pixel_pos - start;

        Ray::new(start, pixel_dir)
    }
}
