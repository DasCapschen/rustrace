use std::ops::{Mul, Add, Sub, Div};

//auto-implement printing
#[derive(Debug, Copy, Clone)]
pub struct Vec3 {
    pub x : f64,
    pub y : f64,
    pub z : f64
}

impl Vec3 {
    /// ctor
    pub fn new(x : f64, y : f64, z : f64) -> Self {
        Vec3 { x, y, z }
    }

    /// get the length of the vector
    pub fn len(&self) -> f64 {
        (self.x*self.x + self.y*self.y + self.z*self.z).sqrt()
    }

    pub fn len_squared(&self) -> f64 {
        self.x*self.x + self.y*self.y + self.z*self.z
    }

    /// normalise the vector (length = 1)
    pub fn normalised(&self) -> Vec3 {
        let len = self.len();
        Vec3 {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len
        }
    }

    /// dot product between self and rhs
    pub fn dot(&self, rhs : Vec3) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    /// cross product between self and rhs
    pub fn cross(&self, rhs : Vec3) -> Vec3 {
        Vec3 {
            x: self.y*rhs.z + self.z*rhs.y, //xyzzy
            y: self.z*rhs.x + self.x*rhs.z, //yzxxz
            z: self.x*rhs.y + self.y*rhs.x  //zxyyx
        }
    }
}

//multiply vector with scalar
impl Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs
        }
    }
}

//multiply scalar with vector
impl Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z
        }
    }
}

//multiply with vector
impl Mul<Vec3> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z
        }
    }
}

//divide by scalar
impl Div<f64> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f64) -> Self::Output {
        Vec3 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs
        }
    }
}

//divide by vector
impl Div<Vec3> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z
        }
    }
}

//add vector
impl Add<Vec3> for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z
        }
    }
}

//subtract vector
impl Sub<Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z
        }
    }
}