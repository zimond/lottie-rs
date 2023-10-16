use crate::model::{Bezier, GradientColor, Rgb, Vector2D};

pub trait Lerp {
    type Target;
    fn lerp(&self, other: &Self, t: f32) -> Self::Target;
}

impl Lerp for Vector2D {
    type Target = Vector2D;

    fn lerp(&self, other: &Self, t: f32) -> Self::Target {
        (*self - *other) * t + other
    }
}

impl Lerp for f32 {
    type Target = f32;

    fn lerp(&self, other: &Self, t: f32) -> Self::Target {
        (*self - *other) * t + *other
    }
}

impl Lerp for Vec<Bezier> {
    type Target = Vec<Bezier>;

    fn lerp(&self, other: &Self, t: f32) -> Self::Target {
        let mut result = self.clone();
        for (bezier, other) in result.iter_mut().zip(other.iter()) {
            for (v, other_v) in bezier.verticies.iter_mut().zip(other.verticies.iter()) {
                *v = v.lerp(*other_v, t);
            }
            for (v, other_v) in bezier.in_tangent.iter_mut().zip(other.in_tangent.iter()) {
                *v = v.lerp(*other_v, t);
            }
            for (v, other_v) in bezier.out_tangent.iter_mut().zip(other.out_tangent.iter()) {
                *v = v.lerp(*other_v, t);
            }
        }
        result
    }
}

impl Lerp for Rgb {
    type Target = Rgb;

    fn lerp(&self, other: &Self, t: f32) -> Self::Target {
        todo!()
    }
}

impl Lerp for Vec<GradientColor> {
    type Target = Vec<GradientColor>;

    fn lerp(&self, other: &Self, t: f32) -> Self::Target {
        todo!()
    }
}
