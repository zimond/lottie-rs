use bevy::math::{Quat, Vec3};
use bevy::prelude::{Color, Transform};
use bevy_prototype_lyon::prelude::{DrawMode, FillMode, LineCap, LineJoin, StrokeMode};
use lottie_core::prelude::StyledShape;
use lottie_core::{
    AnimatedExt, LineCap as LottieLineCap, LineJoin as LottieLineJoin, Rgb,
    Transform as LottieTransform,
};

pub fn shape_draw_mode(shape: &StyledShape) -> DrawMode {
    let fill = shape.fill.color.initial_value();
    let fill_opacity = (shape.fill.opacity.initial_value() * 255.0) as u8;
    let stroke_width: f32 = shape
        .stroke
        .as_ref()
        .map(|stroke| stroke.width.initial_value())
        .unwrap_or(0.0);
    let stroke = shape
        .stroke
        .as_ref()
        .map(|stroke| stroke.color.initial_value())
        .unwrap_or(Rgb::new_u8(0, 0, 0));
    let stroke_opacity = shape
        .stroke
        .as_ref()
        .map(|stroke| stroke.opacity.initial_value() * 255.0)
        .unwrap_or(0.0) as u8;
    let fill_mode = FillMode::color(Color::rgba_u8(fill.r, fill.g, fill.b, fill_opacity));
    let mut stroke_mode = StrokeMode::new(
        Color::rgba_u8(stroke.r, stroke.g, stroke.b, stroke_opacity),
        stroke_width,
    );
    if let Some(stroke) = shape.stroke.as_ref() {
        let line_cap = match stroke.line_cap {
            LottieLineCap::Butt => LineCap::Butt,
            LottieLineCap::Round => LineCap::Round,
            LottieLineCap::Square => LineCap::Square,
        };
        stroke_mode.options.start_cap = line_cap;
        stroke_mode.options.end_cap = line_cap;
        stroke_mode.options.line_join = match stroke.line_join {
            LottieLineJoin::Miter => LineJoin::Miter,
            LottieLineJoin::Round => LineJoin::Round,
            LottieLineJoin::Bevel => LineJoin::Bevel,
        };
    }
    DrawMode::Outlined {
        fill_mode,
        outline_mode: stroke_mode,
    }
}

pub fn initial_transform_and_anchor(transform: &LottieTransform) -> (Transform, Vec3) {
    let pos = transform.position.initial_value();
    let scale = transform.scale.initial_value();
    let rotation = transform.rotation.initial_value();
    let anchor = transform.anchor.initial_value();
    let anchor = Vec3::new(anchor.x, anchor.y, 0.0);
    (
        Transform {
            translation: Vec3::new(pos.x, pos.y, 0.0) - anchor,
            rotation: Quat::from_rotation_z(rotation * std::f32::consts::PI / 180.0),
            scale: Vec3::new(scale.x / 100.0, scale.y / 100.0, 1.0),
        },
        anchor,
    )
}
