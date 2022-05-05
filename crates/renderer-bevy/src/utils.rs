use bevy::math::{Quat, Vec3};
use bevy::prelude::{Color, Transform};
use bevy_prototype_lyon::prelude::{DrawMode, FillMode, StrokeMode};
use lottie_core::prelude::StyledShape;
use lottie_core::{AnimatedExt, Rgb, Transform as LottieTransform};

pub fn shape_draw_mode(shape: &StyledShape) -> DrawMode {
    let fill = shape.fill.color.initial_value();
    let fill_opacity = (shape.fill.opacity.initial_value() * 255.0) as u8;
    let stroke_width: f32 = shape
        .strokes
        .iter()
        .map(|stroke| stroke.width.initial_value())
        .sum();
    let stroke = shape
        .strokes
        .first()
        .map(|s| s.color.initial_value())
        .unwrap_or(Rgb::new_u8(0, 0, 0));
    let stroke_opacity = shape
        .strokes
        .first()
        .map(|s| s.opacity.initial_value() * 255.0)
        .unwrap_or(0.0) as u8;
    DrawMode::Outlined {
        fill_mode: FillMode::color(Color::rgba_u8(fill.r, fill.g, fill.b, fill_opacity)),
        outline_mode: StrokeMode::new(
            Color::rgba_u8(stroke.r, stroke.g, stroke.b, stroke_opacity),
            stroke_width,
        ),
    }
}

pub fn initial_transform(transform: &LottieTransform) -> Transform {
    let pos = transform.position.initial_value();
    let scale = transform.scale.initial_value();
    let rotation = transform.rotation.initial_value();
    Transform {
        translation: Vec3::new(pos.x, pos.y, 0.0),
        rotation: Quat::from_rotation_z(rotation * std::f32::consts::PI / 180.0),
        scale: Vec3::new(scale.x / 100.0, scale.y / 100.0, 1.0),
    }
}
