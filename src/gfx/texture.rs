use image2::{Image, ImageBuf, Rgb};
use std::path::Path;

use crate::math::vec3::Vec3;
use std::sync::Arc;

/*
TODO: don't know where to put this, but...
fn luminance() -> f32 {
    //either vec3 as input, or texture & uvcoords as input

    0.2126 * R + 0.7152 * G + 0.0722 * B
}
*/

pub trait Texture: Send + Sync {
    /// returns a color as vec3 from UV coordinates
    fn texture(&self, uv_coords: (f32, f32)) -> Vec3;
}

pub enum TextureFilter {
    Nearest,
    Linear,
}

#[derive(Debug, Copy, Clone)]
pub struct ConstantTexture {
    color: Vec3,
}
impl ConstantTexture {
    pub fn new(color: Vec3) -> Self {
        Self { color }
    }
}
impl Texture for ConstantTexture {
    fn texture(&self, _uv_coords: (f32, f32)) -> Vec3 {
        self.color
    }
}

#[derive(Clone)]
pub struct CheckeredTexture {
    texture1: Arc<dyn Texture>,
    texture2: Arc<dyn Texture>,
}
impl CheckeredTexture {
    pub fn new(texture1: Arc<dyn Texture>, texture2: Arc<dyn Texture>) -> Self {
        CheckeredTexture { texture1, texture2 }
    }
}
impl Texture for CheckeredTexture {
    fn texture(&self, uv_coords: (f32, f32)) -> Vec3 {
        let (u, v) = uv_coords;

        let it = (100.0 * u).sin() * (100.0 * v).sin();
        if it < 0.0 {
            self.texture1.texture(uv_coords)
        } else {
            self.texture2.texture(uv_coords)
        }
    }
}

/*
#[derive(Debug, Copy, Clone)]
pub struct Perlin;
impl Perlin {}
impl Texture for Perlin {
    fn texture(&self, u: f32, v: f32) -> Vec3 {

    }
}
*/

#[derive(Clone)]
pub struct ImageTexture {
    data: ImageBuf<f32, Rgb>,
}
impl ImageTexture {
    pub fn new<P: AsRef<Path>>(filepath: P) -> Self {
        //reads the image as float (64bit) RGB (LDR is "promoted" to HDR! HDR stays HDR)
        //this is *linear* colorspace!
        let ptr = image2::io::read_f32(filepath).expect("failed to load image!");
        let mut buf = ImageBuf::new(ptr.width(), ptr.height());
        ptr.convert_type(&mut buf);
        Self { data: buf }
    }
}
impl Texture for ImageTexture {
    fn texture(&self, uv_coords: (f32, f32)) -> Vec3 {
        let (u, v) = uv_coords;

        //scale u,v from [0,1] to [0,width) or [0,height)
        let u = u * (self.data.width() - 1) as f32;
        let v = v * (self.data.height() - 1) as f32;

        let u_lo = u.floor();
        let u_hi = u.ceil();
        let alpha = u - u_lo;

        let v_lo = v.floor();
        let v_hi = v.ceil();
        let beta = v - v_lo;

        let px1: Vec3 = self.data.at(u_lo as usize, v_lo as usize).into();
        let px2: Vec3 = self.data.at(u_hi as usize, v_lo as usize).into();

        let interp1 = (1.0 - alpha) * px1 + alpha * px2;

        let px1: Vec3 = self.data.at(u_lo as usize, v_hi as usize).into();
        let px2: Vec3 = self.data.at(u_hi as usize, v_hi as usize).into();

        let interp2 = (1.0 - alpha) * px1 + alpha * px2;

        (1.0 - beta) * interp1 + beta * interp2
    }
}
