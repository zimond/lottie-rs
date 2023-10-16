use bevy::prelude::Color;
use lottie_core::prelude::{
    AnyFill, AnyStroke, FillRule as LottieFillRule, LineCap as LottieLineCap,
    LineJoin as LottieLineJoin, Rgb, StyledShape,
};
use lyon::path::FillRule;
use lyon::tessellation::{FillOptions, LineCap, LineJoin, StrokeOptions};

use crate::shape::{DrawMode, Fill, SolidOrGradient, Stroke};

/// Get an initial draw mode for a shape. If fill/stroke uses gradient fill, a
/// default white color is used in this method
pub fn shape_draw_mode(shape: &StyledShape) -> DrawMode {
    let (fill, fill_opacity) = match &shape.fill {
        AnyFill::Solid(fill) => {
            let fill_opacity = (fill.opacity.initial_value() * 255.0) as u8;
            let fill = fill.color.initial_value();
            (fill, fill_opacity)
        }
        AnyFill::Gradient(gradient) => (Rgb::new_u8(255, 255, 255), 255),
    };
    let fill_rule = match &shape.fill {
        AnyFill::Solid(fill) => &fill.fill_rule,
        AnyFill::Gradient(gradient) => &gradient.fill_rule,
    };
    DrawMode {
        fill: if fill_opacity == 0 {
            None
        } else {
            let mut fill = Fill {
                color: SolidOrGradient::Solid(Color::rgba_u8(fill.r, fill.g, fill.b, fill_opacity)),
                options: FillOptions::default(),
                opacity: 1.0,
            };
            fill.options.fill_rule = match fill_rule {
                LottieFillRule::NonZero => FillRule::NonZero,
                LottieFillRule::EvenOdd => FillRule::EvenOdd,
            };
            Some(fill)
        },
        stroke: shape.stroke.as_ref().map(|stroke| {
            let stroke_width: f32 = stroke.width().initial_value();
            let (color, stroke_opacity) = match &stroke {
                AnyStroke::Solid(stroke) => {
                    let stroke_opacity = (stroke.opacity.initial_value() * 255.0) as u8;
                    let stroke = stroke.color.initial_value();
                    (stroke, stroke_opacity)
                }
                AnyStroke::Gradient(gradient) => (Rgb::new_u8(255, 255, 255), 255),
            };

            let mut result = Stroke {
                color: SolidOrGradient::Solid(Color::rgba_u8(
                    color.r,
                    color.g,
                    color.b,
                    stroke_opacity,
                )),
                options: StrokeOptions::default().with_line_width(stroke_width),
                opacity: 1.0,
            };
            let line_cap = match stroke.line_cap() {
                LottieLineCap::Butt => LineCap::Butt,
                LottieLineCap::Round => LineCap::Round,
                LottieLineCap::Square => LineCap::Square,
            };
            result.options.start_cap = line_cap;
            result.options.end_cap = line_cap;
            result.options.line_join = match stroke.line_join() {
                LottieLineJoin::Miter => LineJoin::Miter,
                LottieLineJoin::Round => LineJoin::Round,
                LottieLineJoin::Bevel => LineJoin::Bevel,
            };
            result
        }),
    }
}
