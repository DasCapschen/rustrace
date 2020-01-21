use image::DynamicImage;
use image::GenericImageView;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use crate::vec3::Vec3;

/*
TODO: implement filtering
*/

pub trait Texture: Send + Sync {
    /// returns a color as vec3 from UV coordinates
    fn texture(&self, uv_coords: (f64,f64)) -> Vec3;
}

pub enum TextureFilter {
    Nearest,
    Linear,
}

#[derive(Debug, Copy, Clone)]
pub struct ConstantTexture {
    color: Vec3
}
impl ConstantTexture {
    pub fn new(color: Vec3) -> Self {
        Self {
            //copy color
            color: color
        }
    }
}
impl Texture for ConstantTexture {
    fn texture(&self, uv_coords: (f64,f64)) -> Vec3 {
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
        CheckeredTexture {
            texture1,
            texture2,
        }
    }
}
impl Texture for CheckeredTexture {
    fn texture(&self, uv_coords: (f64,f64)) -> Vec3 {
        let (u,v) = uv_coords;

        let it = (10.0*u).sin() * (10.0*v).sin();
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
    fn texture(&self, u: f64, v: f64) -> Vec3 {
        
    }
}
*/

#[derive(Clone)]
pub struct ImageTexture {
    data: DynamicImage,
}
impl ImageTexture {
    pub fn new(filepath: PathBuf) -> Self {
        Self {
            data: image::open(filepath).expect("failed to load image")
        }
    }
}
impl Texture for ImageTexture {
    fn texture(&self, uv_coords: (f64,f64)) -> Vec3 {
        let (u,v) = uv_coords;

        //scale u,v from [0,1] to [0,width) or [0,height)
        let u = u * (self.data.width()-1) as f64;
        let v = v * (self.data.height()-1) as f64;

        let u_lo = u.floor();
        let u_hi = u.ceil();
        let alpha = u - u_lo;

        let v_lo = v.floor();
        let v_hi = v.ceil();
        let beta = v - v_lo;

        let px1: Vec3 = self.data.get_pixel(u_lo as u32, v_lo as u32).into();
        let px2: Vec3 = self.data.get_pixel(u_hi as u32, v_lo as u32).into();

        let interp1 = (1.0 - alpha) * px1 + alpha * px2;

        let px1: Vec3 = self.data.get_pixel(u_lo as u32, v_hi as u32).into();
        let px2: Vec3 = self.data.get_pixel(u_hi as u32, v_hi as u32).into();

        let interp2 = (1.0 - alpha) * px1 + alpha * px2;

        (1.0 - beta) * interp1 + beta * interp2
    }
}