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
        let r = other.r as f32 + (self.r as f32 - other.r as f32) * t;
        let g = other.g as f32 + (self.g as f32 - other.g as f32) * t;
        let b = other.b as f32 + (self.b as f32 - other.b as f32) * t;
        Rgb::new_u8(r as u8, g as u8, b as u8)
    }
}

impl Lerp for Vec<GradientColor> {
    type Target = Vec<GradientColor>;

    fn lerp(&self, other: &Self, t: f32) -> Self::Target {
        self.iter()
            .zip(other)
            .map(|(x, y)| {
                let r = y.color.r as f32 + (x.color.r as f32 - y.color.r as f32) * t;
                let g = y.color.g as f32 + (x.color.g as f32 - y.color.g as f32) * t;
                let b = y.color.b as f32 + (x.color.b as f32 - y.color.b as f32) * t;
                let a = y.color.a as f32 + (x.color.a as f32 - y.color.a as f32) * t;
                let o = y.offset + (x.offset - y.offset) * t;
                GradientColor {
                    offset: o,
                    color: Rgba::new_u8(r as u8, g as u8, b as u8, a as u8),
                }
            })
            .collect()
    }
}
