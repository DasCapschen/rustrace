use rand::Rng;
use std::iter::Sum;
use std::ops::DivAssign;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};

//auto-implement printing
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    /// Creates a new vector the given components
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }

    /// Creates a new Vector, converting from 0..255 to 0.0..1.0
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Vec3 {
            x: r as f64 / 255.0,
            y: g as f64 / 255.0,
            z: b as f64 / 255.0,
        }
    }

    /// get the length of the vector
    pub fn len(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn len_squared(&self) -> f64 {
        (self.x * self.x) + (self.y * self.y) + (self.z * self.z)
    }

    /// normalise the vector (length = 1)
    pub fn normalised(&self) -> Vec3 {
        let len = self.len();
        Vec3 {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len,
        }
    }

    /// dot product between self and rhs
    pub fn dot(&self, rhs: Vec3) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    /// cross product between self and rhs
    pub fn cross(&self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * rhs.z - self.z * rhs.y, //xyzzy
            y: self.z * rhs.x - self.x * rhs.z, //yzxxz
            z: self.x * rhs.y - self.y * rhs.x, //zxyyx
        }
    }

    pub fn reflect(&self, normal: Vec3) -> Vec3 {
        //        \   n   ↗
        //       in\  ↑  /   reflected
        //          ↘ | /
        //   ---------+----+----
        //             \   ↑
        //      also in \  | - n * in·n
        //               ↘ |
        // => in + 2*(-n * in·n)

        // self.dot(normal) => cos(angle between self and normal)
        // normal * ^ => normal scaled to the "height" of self
        // 2 * ^

        let normal = normal.normalised(); //just in case someone forgot
        *self - 2.0 * self.dot(normal) * normal
    }

    pub fn refract(&self, normal: Vec3, n_in: f64, n_out: f64) -> Option<Vec3> {
        //    ＼     n
        //   in ＼   ↑
        //         ↘ |    n_in
        //-----------+-------------
        //   n_out   |\
        //           | \ out
        //           |  ↘
        //sin(alpha_in) * n_in == sin(alpha_out) * n_out

        //this could be written MUCH shorter, but because it is pretty hard to understand the formulas
        //and what is going on, I have decided to make it very verbose
        // if you don't understand what is happening, try these steps in geogebra (2d is enough)

        //code from https://raytracing.github.io/books/RayTracingInOneWeekend.html#dielectrics
        // vec3 uv = unit_vector(v); //v_in
        // float dt = dot(uv, n);    //cos_in
        // float discriminant = 1.0 - ni_over_nt*ni_over_nt*(1-dt*dt); //cos_out_squared
        // if (discriminant > 0) {
        //    refracted = ni_over_nt*(uv - n*dt) - n*sqrt(discriminant);
        //    return true;
        // }

        let v_in = self.normalised(); // |v| == 1
        let normal = normal.normalised(); // |n| == 1

        let scale = n_in / n_out; //scale of the angle

        let cos_in = v_in.dot(normal); // v · n == |v|*|n|*cos(angle(v,n))

        let sin_in_squared = 1.0 - (cos_in * cos_in); // 1 - cos²(a) == sin²(a)

        let sin_out_squared = (scale * scale) * sin_in_squared; // sin²(alpha_in) * (n_in/n_out)² == sin²(alpha_out)

        let cos_out_squared = 1.0 - sin_out_squared; // 1 - sin²(a) == cos²(a) == refracted.dot(normal)²

        //no refraction possible, total reflection
        if cos_out_squared < 0.0 {
            return None;
        }

        let normal_scaled_in = normal * cos_in; //normal scaled to be on same "height" as v_in
        let normal_scaled_out = normal * cos_out_squared.sqrt(); //normal scaled to be on same "height" as v_out

        let direction = v_in - normal_scaled_in; //the "direction" v_in is pointing along the surface
        let scaled_direction = scale * direction; //scaled by n_in / n_out to correct angle

        Some(scaled_direction - normal_scaled_out) //final refracted vector
    }

    pub fn random_in_unit_sphere() -> Vec3 {
        let mut rng = rand::thread_rng();
        let mut random_dir;
        loop {
            random_dir = Vec3::new(
                rng.gen_range(-1.0, 1.0),
                rng.gen_range(-1.0, 1.0),
                rng.gen_range(-1.0, 1.0),
            );

            if random_dir.len() <= 1.0 {
                break;
            }
        }
        random_dir
    }

    pub fn random_in_unit_disk() -> Vec3 {
        let mut rng = rand::thread_rng();
        let mut random_dir;
        loop {
            random_dir = Vec3::new(rng.gen_range(-1.0, 1.0), rng.gen_range(-1.0, 1.0), 0.0);
            if random_dir.len() <= 1.0 {
                break;
            }
        }
        random_dir
    }

    pub fn lerp(lhs: Vec3, rhs: Vec3, alpha: f64) -> Vec3 {
        (1.0 - alpha) * lhs + alpha * rhs
    }
}

//multiply vector with scalar
impl Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

//multiply scalar with vector
impl Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
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
            z: self.z * rhs.z,
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
            z: self.z / rhs,
        }
    }
}

//divide by scalar
impl DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, rhs: f64) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

//divide by vector
impl Div<Vec3> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
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
            z: self.z + rhs.z,
        }
    }
}
impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

//subtract vector
impl Sub<Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}
impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

//negate vector
impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl<'a> Sum<&'a Vec3> for Vec3 {
    fn sum<I: Iterator<Item = &'a Vec3>>(iter: I) -> Vec3 {
        let mut result = Vec3::new(0.0, 0.0, 0.0);
        for v in iter {
            result = result + *v;
        }
        result
    }
}

impl From<&[f64]> for Vec3 {
    fn from(slice: &[f64]) -> Self {
        if slice.len() < 3 {
            todo!("handle error");
        }
        Vec3 {
            x: slice[0],
            y: slice[1],
            z: slice[2],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        assert_eq!(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3 {
                x: 1.0,
                y: 2.0,
                z: 3.0
            }
        )
    }

    #[test]
    fn test_rgb() {
        assert_eq!(
            Vec3::rgb(255, 204, 153),
            Vec3 {
                x: 1.0,
                y: 0.8,
                z: 0.6
            }
        )
    }

    #[test]
    fn test_len() {
        assert!((Vec3::new(3.0, 4.0, 0.0).len() - 5.0).abs() < std::f64::EPSILON)
    }

    #[test]
    fn test_len_squared() {
        assert!((Vec3::new(3.0, 4.0, 0.0).len_squared() - 25.0).abs() < std::f64::EPSILON)
    }

    #[test]
    fn test_dot() {
        let u = Vec3::new(1.0, 2.0, 3.0);
        let v = Vec3::new(4.0, -1.0, 0.0);
        assert!((u.dot(v) - 2.0).abs() < std::f64::EPSILON)
    }

    #[test]
    fn test_cross() {
        let u = Vec3::new(1.0, 0.0, 0.0);
        let v = Vec3::new(0.0, 1.0, 0.0);
        assert_eq!(u.cross(v), Vec3::new(0.0, 0.0, 1.0));

        let u = Vec3::new(1.0, 2.0, -3.0);
        let v = Vec3::new(5.0, 0.0, 1.0);
        assert_eq!(u.cross(v), Vec3::new(2.0, -16.0, -10.0));
    }

    #[test]
    fn test_reflect() {
        let u = Vec3::new(-1.0, -1.0, 0.0);
        let n = Vec3::new(0.0, 1.0, 0.0);
        assert_eq!(u.reflect(n), Vec3::new(-1.0, 1.0, 0.0));

        let u = Vec3::new(-1.0, -1.0, 0.0);
        let n = Vec3::new(0.0, 2.0, 0.0);
        assert_eq!(u.reflect(n), Vec3::new(-1.0, 1.0, 0.0));
    }

    #[test]
    fn test_refract() {}

    #[test]
    fn test_add() {
        let u = Vec3::new(1.0, 2.0, 3.0);
        let v = Vec3::new(4.0, 3.0, 2.0);
        assert_eq!(u + v, Vec3::new(5.0, 5.0, 5.0));

        let u = Vec3::new(1.0, 2.0, 3.0);
        let v = Vec3::new(-1.0, -2.0, -3.0);
        assert_eq!(u + v, Vec3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_mul_scalar() {
        let u = Vec3::new(0.0, 2.0, -5.0);
        assert_eq!(2.0 * u, Vec3::new(0.0, 4.0, -10.0));
        assert_eq!(u * 2.0, Vec3::new(0.0, 4.0, -10.0));
    }
    #[test]
    fn test_mul_vec() {
        let u = Vec3::new(3.0, 1.0, -2.0);
        let v = Vec3::new(0.0, -1.0, 2.0);
        assert_eq!(u * v, Vec3::new(0.0, -1.0, -4.0));
        assert_eq!(v * u, Vec3::new(0.0, -1.0, -4.0));
    }

    #[test]
    fn test_div_scalar() {
        let u = Vec3::new(3.0, 1.0, -2.0);
        assert_eq!(u / 2.0, Vec3::new(1.5, 0.5, -1.0));
    }
    #[test]
    fn test_div_vec() {
        let v = Vec3::new(0.0, -1.0, 4.0);
        let u = Vec3::new(3.0, 2.0, -2.0);
        assert_eq!(v / u, Vec3::new(0.0, -0.5, -2.0));
        assert_eq!(u / v, Vec3::new(std::f64::INFINITY, -2.0, -0.5)); //oh god why
    }

    #[test]
    fn test_sub() {
        let v = Vec3::new(5.0, 7.0, 2.0);
        let u = Vec3::new(7.0, 5.0, 2.0);
        assert_eq!(v - u, Vec3::new(-2.0, 2.0, 0.0))
    }

    #[test]
    fn test_sub_assign() {
        let mut v = Vec3::new(5.0, 7.0, 2.0);
        v -= Vec3::new(7.0, 5.0, 2.0);
        assert_eq!(v, Vec3::new(-2.0, 2.0, 0.0))
    }

    #[test]
    fn test_neg() {
        assert_eq!(-Vec3::new(1.0, 0.0, -2.0), Vec3::new(-1.0, 0.0, 2.0))
    }

    #[test]
    fn test_sum() {
        let a = vec![
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(2.0, 1.0, 3.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(-1.0, 2.0, -6.0),
        ];

        let sum: Vec3 = a.iter().sum();

        assert_eq!(sum, Vec3::new(2.0, 5.0, 0.0))
    }
}
