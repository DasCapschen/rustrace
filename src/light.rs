use crate::vec3::Vec3;

pub struct Light {
    pub position: Vec3,
    pub color: Vec3,
}

impl Light {
    pub fn new(position: Vec3, color: Vec3) -> Self {
        Light { position, color }
    }
}
