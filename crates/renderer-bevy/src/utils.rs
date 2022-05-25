use bevy::prelude::Color;
use bevy_prototype_lyon::prelude::{DrawMode, FillMode, LineCap, LineJoin, StrokeMode};
use lottie_core::prelude::StyledShape;
use lottie_core::{AnimatedExt, LineCap as LottieLineCap, LineJoin as LottieLineJoin, Rgb};

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
