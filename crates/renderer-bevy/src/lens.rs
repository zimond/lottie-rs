use bevy::prelude::Transform;
use bevy_tweening::Lens;
use lottie_core::prelude::{OpacityHierarchy, PathExt};
use lottie_core::{Animated, AnimatedExt, Bezier, Transform as LottieTransform};
use lyon::path::path::Builder;

use crate::shape::{DrawMode, Path};

pub struct PathLens {
    pub(crate) start: Vec<Bezier>,
    pub(crate) end: Vec<Bezier>,
}

impl Lens<Path> for PathLens {
    fn lerp(&mut self, target: &mut Path, ratio: f32) {
        let mut builder = Builder::new();
        let beziers = self
            .start
            .iter()
            .zip(self.end.iter())
            .map(|(start, end)| {
                let mut result = start.clone();
                for index in 0..result.verticies.len() {
                    result.verticies[index] +=
                        (end.verticies[index] - start.verticies[index]) * ratio;
                    result.in_tangent[index] +=
                        (end.in_tangent[index] - start.in_tangent[index]) * ratio;
                    result.out_tangent[index] +=
                        (end.out_tangent[index] - start.out_tangent[index]) * ratio;
                }
                result
            })
            .collect::<Vec<_>>();
        beziers.to_path(0.0, &mut builder);
        *target = Path(builder.build());
    }
}

pub struct StrokeWidthLens {
    pub(crate) start: f32,
    pub(crate) end: f32,
}

impl Lens<DrawMode> for StrokeWidthLens {
    fn lerp(&mut self, target: &mut DrawMode, ratio: f32) {
        let w = self.start + (self.end - self.start) * ratio;
        if let Some(stroke) = target.stroke.as_mut() {
            stroke.options.line_width = w;
        }
    }
}

/// Lerp [LottieTransform] as a whole
pub struct TransformLens {
    pub(crate) data: LottieTransform,
    pub(crate) frames: f32,
}

impl Lens<Transform> for TransformLens {
    fn lerp(&mut self, target: &mut Transform, ratio: f32) {
        let frame = self.frames * ratio;
        let value = self.data.value(frame);
        let zindex = target.translation.z;
        *target = Transform::from_matrix(value);
        target.translation.z = zindex;
    }
}

pub struct OpacityLens {
    pub(crate) opacity: OpacityHierarchy,
    pub(crate) frames: f32,
    pub(crate) fill_opacity: Animated<f32>,
    pub(crate) stroke_opacity: Option<Animated<f32>>,
}

impl Lens<DrawMode> for OpacityLens {
    fn lerp(&mut self, target: &mut DrawMode, ratio: f32) {
        let frame = self.frames as f32 * ratio;
        let value = self.opacity.value(frame);
        let fill_opacity = self.fill_opacity.value(frame) / 100.0;
        let stroke_opacity = self
            .stroke_opacity
            .as_ref()
            .map(|o| o.value(frame) / 100.0)
            .unwrap_or(1.0);

        if let Some(fill) = target.fill.as_mut() {
            fill.color.set_a(value * fill_opacity);
        }
        if let Some(stroke) = target.stroke.as_mut() {
            stroke.color.set_a(value * stroke_opacity);
        }
    }
}
