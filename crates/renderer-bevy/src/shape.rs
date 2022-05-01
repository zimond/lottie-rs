use bevy::math::{Quat, Vec3};
use bevy::prelude::{Color, Transform};
use bevy_prototype_lyon::prelude::{DrawMode, FillMode, StrokeMode};
use lottie_core::prelude::StyledShape;
use lottie_core::AnimatedExt;

pub trait StyledShapeExt {
    fn draw_mode(&self) -> DrawMode;
    fn initial_transform(&self) -> Transform;
}

impl StyledShapeExt for StyledShape {
    fn draw_mode(&self) -> DrawMode {
        let fill = self.fill.color.initial_value();
        let fill_opacity = (self.fill.opacity.initial_value() * 255.0) as u8;
        DrawMode::Outlined {
            fill_mode: FillMode::color(Color::rgba_u8(fill.r, fill.g, fill.b, fill_opacity)),
            outline_mode: StrokeMode::new(Color::BLACK, 0.0),
        }
    }

    fn initial_transform(&self) -> Transform {
        let pos = self.transform.position.initial_value();
        let scale = self.transform.scale.initial_value();
        Transform {
            translation: Vec3::new(pos.x, pos.y, 0.0),
            rotation: Quat::default(),
            scale: Vec3::new(scale.x / 100.0, scale.y / 100.0, 1.0),
        }
    }
}
