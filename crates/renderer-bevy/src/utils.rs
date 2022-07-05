use bevy::prelude::Color;
use lottie_core::prelude::StyledShape;
use lottie_core::{
    AnimatedExt, FillRule as LottieFillRule, LineCap as LottieLineCap, LineJoin as LottieLineJoin,
};
use lyon::path::FillRule;
use lyon::tessellation::{FillOptions, LineCap, LineJoin, StrokeOptions};

use crate::shape::{DrawMode, Fill, Stroke};

pub fn shape_draw_mode(shape: &StyledShape) -> DrawMode {
    let fill = shape.fill.color.initial_value();
    let fill_opacity = (shape.fill.opacity.initial_value() * 255.0) as u8;
    DrawMode {
        fill: if fill_opacity == 0 {
            None
        } else {
            let mut fill = Fill {
                color: Color::rgba_u8(fill.r, fill.g, fill.b, fill_opacity),
                options: FillOptions::default(),
            };
            fill.options.fill_rule = match shape.fill.fill_rule {
                LottieFillRule::NonZero => FillRule::NonZero,
                LottieFillRule::EvenOdd => FillRule::EvenOdd,
            };
            Some(fill)
        },
        stroke: shape.stroke.as_ref().map(|stroke| {
            let stroke_width: f32 = stroke.width.initial_value();
            let color = stroke.color.initial_value();
            let stroke_opacity = (stroke.opacity.initial_value() * 255.0) as u8;

            let mut result = Stroke {
                color: Color::rgba_u8(color.r, color.g, color.b, stroke_opacity),
                options: StrokeOptions::default().with_line_width(stroke_width),
            };
            let line_cap = match stroke.line_cap {
                LottieLineCap::Butt => LineCap::Butt,
                LottieLineCap::Round => LineCap::Round,
                LottieLineCap::Square => LineCap::Square,
            };
            result.options.start_cap = line_cap;
            result.options.end_cap = line_cap;
            result.options.line_join = match stroke.line_join {
                LottieLineJoin::Miter => LineJoin::Miter,
                LottieLineJoin::Round => LineJoin::Round,
                LottieLineJoin::Bevel => LineJoin::Bevel,
            };
            result
        }),
    }
}
