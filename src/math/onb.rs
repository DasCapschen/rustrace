use crate::math::vec3::Vec3;

pub struct ONB {
    pub u: Vec3,
    pub v: Vec3,
    pub w: Vec3,
}

impl ONB {
    pub fn from_axes(u: Vec3, v: Vec3, w: Vec3) -> Self {
        ONB { u, v, w }
    }

    //w should be normalised
    pub fn from_w(w: Vec3) -> Self {
        // 1) permute w to make sure we have a vector pointing *anywhere* else
        let _temp = Vec3::new(w.y, w.z, w.x);
        // 2) calculate any vector perpendicular to w => u (w x temp)
        let u = w.cross(_temp);
        // 3) calculate v (perpendicular to w and u)
        let v = w.cross(u);

        ONB { u, v, w }
    }

    pub fn to_local(&self, n: Vec3) -> Vec3 {
        n.x * self.u + n.y * self.v + n.z * self.w
    }
}
