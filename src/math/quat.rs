use std::ops::Neg;
use std::ops::Div;
use std::ops::Mul;
use crate::math::vec3::Vec3;

#[derive(Copy, Clone, Debug)]
pub struct Quaternion {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl Quaternion {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    //we first yaw (Y), then pitch (X), then roll (Z)
    pub fn from_euler(yaw: f32, pitch: f32, roll: f32) -> Self {
        //cos and sin expect radians
        let yaw = (yaw/2.0).to_radians();
        let pitch = (pitch/2.0).to_radians();
        let roll = (roll/2.0).to_radians();

        let cos_yaw   = yaw.cos();
        let cos_pitch = pitch.cos();
        let cos_roll  = roll.cos();
        let sin_yaw   = yaw.sin();
        let sin_pitch = pitch.sin();
        let sin_roll  = roll.sin();

        Self {
            x: sin_yaw * cos_pitch * sin_roll + cos_yaw * sin_pitch * cos_roll,
            y: sin_yaw * cos_pitch * cos_roll - cos_yaw * sin_pitch * sin_roll,
            z: cos_yaw * cos_pitch * sin_roll - sin_yaw * sin_pitch * cos_roll,
            w: cos_yaw * cos_pitch * cos_roll + sin_yaw * sin_pitch * sin_roll,
        }
    }

    pub fn len(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }

    pub fn len_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    pub fn normalised(&self) -> Self {
        let len = 1.0 / self.len();
        Self {
            x: self.x * len,
            y: self.y * len,
            z: self.z * len,
            w: self.w * len,
        }
    }

    pub fn conjugate(&self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w
        }
    }

    pub fn inverse(&self) -> Self {
        self.conjugate() / self.len_squared()
    }

    //sometimes you read qpq^-1, sometimes qpq*, so which is right?
    //i think inverse, and qpq* is special case if |q| == 1
    //because q^-1 == q* if |q| == 1
    pub fn rotate_vector(&self, v: Vec3) -> Vec3 {
        let q = (*self) * v * self.inverse();
        Vec3::new(q.x, q.y, q.z) //ignore w
    }
    pub fn unrotate_vector(&self, v: Vec3) -> Vec3 {
        let q = self.inverse() * v * (*self);
        Vec3::new(q.x, q.y, q.z)
    }
}

impl Mul<Quaternion> for Quaternion {
    type Output = Quaternion;
    fn mul(self, rhs: Quaternion) -> Self::Output {
        Quaternion {
            x: self.x * rhs.w + self.w * rhs.x + self.y * rhs.z - self.z * rhs.y,
            y: self.y * rhs.w + self.w * rhs.y + self.z * rhs.x - self.x * rhs.z, 
            z: self.z * rhs.w + self.w * rhs.z + self.x * rhs.y - self.y * rhs.x,
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        }
    }
}

impl Mul<Vec3> for Quaternion {
    type Output = Quaternion;
    fn mul(self, rhs: Vec3) -> Self::Output { 
        Quaternion {
            x:   self.w * rhs.x + self.y * rhs.z - self.z * rhs.y, 
            y:   self.w * rhs.y + self.z * rhs.x - self.x * rhs.z, 
            z:   self.w * rhs.z + self.x * rhs.y - self.y * rhs.x,
            w: - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        }
    }
}
impl Mul<Quaternion> for Vec3 {
    type Output = Quaternion;
    fn mul(self, rhs: Quaternion) -> Quaternion { 
        Quaternion {
            x:   rhs.w * self.x + rhs.y * self.z - rhs.z * self.y, 
            y:   rhs.w * self.y + rhs.z * self.x - rhs.x * self.z, 
            z:   rhs.w * self.z + rhs.x * self.y - rhs.y * self.x,
            w: - rhs.x * self.x - rhs.y * self.y - rhs.z * self.z,
        }
    }
}

impl Mul<f32> for Quaternion {
    type Output = Quaternion;
    fn mul(self, rhs: f32) -> Quaternion { 
        Quaternion {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            w: self.w * rhs,
        }
    }
}

impl Mul<Quaternion> for f32 {
    type Output = Quaternion;
    fn mul(self, rhs: Quaternion) -> Quaternion { 
        Quaternion {
            x: rhs.x * self,
            y: rhs.y * self,
            z: rhs.z * self,
            w: rhs.w * self,
        }
    }
}

impl Div<f32> for Quaternion {
    type Output = Quaternion;
    fn div(self, rhs: f32) -> Quaternion { 
        Quaternion {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
            w: self.w / rhs,
        }
    }
}

impl Neg for Quaternion {
    type Output = Quaternion;
    fn neg(self) -> Quaternion {
        Quaternion {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}