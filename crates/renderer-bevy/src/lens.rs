use bevy::math::Vec2;
use bevy::prelude::Transform;
use bevy_prototype_lyon::prelude::{DrawMode, Path, PathBuilder};
use bevy_tweening::Lens;
use lottie_core::prelude::OpacityHierarchy;
use lottie_core::{Animated, AnimatedExt, Bezier, Transform as LottieTransform};

pub struct PathLens {
    pub(crate) start: Vec<Bezier>,
    pub(crate) end: Vec<Bezier>,
}

impl Lens<Path> for PathLens {
    fn lerp(&mut self, target: &mut Path, ratio: f32) {
        let mut builder = PathBuilder::new();
        for (start, end) in self.start.iter().zip(self.end.iter()) {
            let mut result = start.clone();
            for index in 0..result.verticies.len() {
                result.verticies[index] += (end.verticies[index] - start.verticies[index]) * ratio;
                result.in_tangent[index] +=
                    (end.in_tangent[index] - start.in_tangent[index]) * ratio;
                result.out_tangent[index] +=
                    (end.out_tangent[index] - start.out_tangent[index]) * ratio;
            }
            builder.move_to(Vec2::new(result.verticies[0].x, result.verticies[0].y));
            for index in 0..(result.verticies.len() - 1) {
                let c1 = result.verticies[index] + result.out_tangent[index];
                let c2 = result.verticies[index + 1] + result.in_tangent[index + 1];
                let to = result.verticies[index + 1];
                builder.cubic_bezier_to(
                    Vec2::new(c1.x, c1.y),
                    Vec2::new(c2.x, c2.y),
                    Vec2::new(to.x, to.y),
                );
            }
            if result.closed {
                builder.close();
            }
        }
        *target = builder.build();
    }
}

pub struct StrokeWidthLens {
    pub(crate) start: f32,
    pub(crate) end: f32,
}

impl Lens<DrawMode> for StrokeWidthLens {
    fn lerp(&mut self, target: &mut DrawMode, ratio: f32) {
        let w = self.start + (self.end - self.start) * ratio;
        match target {
            DrawMode::Stroke(s) => s.options.line_width = w,
            DrawMode::Outlined { outline_mode, .. } => outline_mode.options.line_width = w,
            _ => {}
        }
    }
}

/// Lerp [LottieTransform] as a whole
pub struct TransformLens {
    pub(crate) data: LottieTransform,
    pub(crate) frames: u32,
}

impl Lens<Transform> for TransformLens {
    fn lerp(&mut self, target: &mut Transform, ratio: f32) {
        let frame = (self.frames as f32 * ratio).round() as u32;
        let value = self.data.value(frame);
        *target = Transform::from_matrix(value)
    }
}

pub struct OpacityLens {
    pub(crate) opacity: OpacityHierarchy,
    pub(crate) frames: u32,
    pub(crate) fill_opacity: Animated<f32>,
    pub(crate) stroke_opacity: Option<Animated<f32>>,
}

impl Lens<DrawMode> for OpacityLens {
    fn lerp(&mut self, target: &mut DrawMode, ratio: f32) {
        let frame = (self.frames as f32 * ratio).round() as u32;
        let value = self.opacity.value(frame);
        let fill_opacity = self.fill_opacity.value(frame) / 100.0;
        let stroke_opacity = self
            .stroke_opacity
            .as_ref()
            .map(|o| o.value(frame) / 100.0)
            .unwrap_or(1.0);
        match target {
            DrawMode::Fill(fill) => fill.color.set_a(value * fill_opacity),
            DrawMode::Stroke(stroke) => stroke.color.set_a(value * stroke_opacity),
            DrawMode::Outlined {
                fill_mode,
                outline_mode,
            } => {
                fill_mode.color.set_a(value * fill_opacity);
                outline_mode.color.set_a(value * stroke_opacity)
            }
        };
    }
}
