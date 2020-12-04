use crate::math::vec3::Vec3;
use crate::ray::Ray;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct CropFactor(f32);
impl CropFactor {
    pub const FULL_FORMAT: Self = Self(1.0_f32);
    pub const APSC: Self = Self(1.525_f32);
    pub const APSC_CANON: Self = Self(1.595_f32);
    pub fn custom(cf: f32) -> Self {
        Self(cf)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum Focus {
    AutoFocus,
    Distance(f32),
}

/// implements a camera from which to render from
#[derive(Debug, Copy, Clone)]
pub struct Camera {
    /// position in 3d space
    pub position: Vec3,
    /// the direction the camera is looking in (normalised)
    pub direction: Vec3,
    /// right vector, calculated
    pub right: Vec3,
    /// up vector, calculated
    pub up: Vec3,
    /// the horizontal field of view
    tan_half_fov: f32,
    /// the width of the rendered image
    width: u32,
    /// the height of the rendered image
    height: u32,
    /// either AutoFocus, or distance at which the camera focuses
    focus: Focus,
    /// the aperture of the camera, bigger number leads to more "depth of field" (blurryness)
    aperture: f32,
    pub fstop: i32,
    /// the focal length of the camera. This is not the distance to the object which should be in focus! (see `focus_dist`)
    pub focal_length: f32,
    pub crop_factor: CropFactor,
}

fn calculate_aperture(fstop: i32, focal_length: f32) -> f32 {
    focal_length / std::f32::consts::SQRT_2.powi(fstop)
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
    pub fn new_virtual(position: Vec3, direction: Vec3, fov: f32, width: u32, height: u32) -> Self {
        let fwd = direction.normalised();
        let right = Camera::calc_right(fwd);
        let up = Camera::calc_up(fwd, right);
        Camera {
            position,
            direction: fwd,
            right: right,
            up: up,
            tan_half_fov: (fov / 2.0).to_radians().tan(),
            width,
            height,
            focus: Focus::AutoFocus,
            fstop: 0,
            aperture: 0.0,
            focal_length: 0.0,
            crop_factor: CropFactor::FULL_FORMAT,
        }
    }

    pub fn new_physical(
        position: Vec3,
        direction: Vec3,
        width: u32,
        height: u32,
        focus: Focus,
        focal_length: f32,
        fstop: i32,
        crop_factor: CropFactor,
    ) -> Self {
        let fwd = direction.normalised();
        let right = Camera::calc_right(fwd);
        let up = Camera::calc_up(fwd, right);

        let tan_half_fov = 18.0f32 / (focal_length * crop_factor.0);

        Camera {
            position,
            direction: fwd,
            right,
            up,
            tan_half_fov,
            width,
            height,
            focus,
            aperture: calculate_aperture(fstop, focal_length),
            fstop,
            focal_length,
            crop_factor,
        }
    }

    /// not recommended, directly sets aperture!
    /// use set_fstop instead
    pub fn set_aperture(&mut self, aperture: f32) {
        self.aperture = aperture;
    }
    pub fn set_fstop(&mut self, fstop: i32) {
        self.fstop = fstop;
        self.update_aperture();
    }

    pub fn set_focal_length(&mut self, focal_length: f32) {
        self.focal_length = focal_length.max(1.0);
        self.update_aperture();
        self.update_fov();
    }

    pub fn set_focus_dist(&mut self, focus: Focus) {
        self.focus = focus;
    }

    pub fn set_crop_factor(&mut self, crop_factor: CropFactor) {
        self.crop_factor = crop_factor;
        self.update_fov();
    }

    fn update_fov(&mut self) {
        self.tan_half_fov = 18.0f32 / (self.focal_length * self.crop_factor.0);
    }
    fn update_aperture(&mut self) {
        self.aperture = calculate_aperture(self.fstop, self.focal_length);
    }

    /// returns the vector pointing to the right of the cameras look-direction
    fn calc_right(fwd: Vec3) -> Vec3 {
        const GLOBAL_UP: Vec3 = Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        };

        GLOBAL_UP.cross(fwd)
    }

    /// returns the vector pointing upwards of the cameras look-direction
    fn calc_up(fwd: Vec3, right: Vec3) -> Vec3 {
        fwd.cross(right)
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

        let focus_dist = match self.focus {
            Focus::AutoFocus => {
                2.0 //TODO!
            }
            Focus::Distance(d) => d,
        };

        //width of our screen at focal distance
        let focal_width = 2.0 * self.tan_half_fov * focus_dist;

        //figure out by how much we have to scale real_width and real_height to arrive at focal_width / focal_height
        let scale = focal_width / self.width as f32;

        //HINT: no need to scale by aspect ratio because x and y don't go between 0..1, but 0..width / 0..height!
        //else it would be:
        //aspect = height / width
        //height_scale = aspect * width_scale

        //calculate local coordinate system
        //let forward = (self.target - self.position).normalised();
        let forward = self.direction;
        let right = self.right * scale;
        let up = self.up * -scale; //negative because (0,0) is TOP right

        let center = self.position + forward * focus_dist; //focus_dist -> move focus plane (Z, depth)

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
