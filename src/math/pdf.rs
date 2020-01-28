use crate::math::onb::ONB;
use crate::math::vec3::Vec3;

/// trait describing a probability density function
trait PDF<T> {
    /// generates a random value distributed with this PDF
    /// this is the inverse of the distribution, P(x)
    fn generate(&self) -> T;

    /// returns the value of the pdf at the given input
    /// this is density, p(x)
    fn value_at(&self, p: T) -> f32;
}

struct CosinePDF {
    onb: ONB,
}

impl CosinePDF {
    pub fn new(normal: Vec3) -> Self {
        CosinePDF {
            onb: ONB::from_w(normal),
        }
    }
}

impl PDF<Vec3> for CosinePDF {
    fn generate(&self) -> Vec3 {
        self.onb.to_local(Vec3::random_cosine_direction())
    }

    fn value_at(&self, p: Vec3) -> f32 {
        let cosine = self.onb.w.dot(p.normalised());
        if cosine < 0.0 {
            0.0
        } else {
            cosine / std::f32::consts::PI
        }
    }
}

struct MixturePDF<'a, T> {
    a: &'a dyn PDF<T>,
    b: &'a dyn PDF<T>,
}

impl<'a> MixturePDF<'a, Vec3> {
    pub fn new(a: &'a dyn PDF<Vec3>, b: &'a dyn PDF<Vec3>) -> Self {
        MixturePDF { a, b }
    }
}

impl<'a> PDF<Vec3> for MixturePDF<'a, Vec3> {
    fn generate(&self) -> Vec3 {
        if rand::random() {
            self.a.generate()
        } else {
            self.b.generate()
        }
    }

    fn value_at(&self, p: Vec3) -> f32 {
        0.5 * self.a.value_at(p) + 0.5 * self.b.value_at(p)
    }
}
