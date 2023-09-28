use bevy::prelude::{Transform, Vec2};
use bevy_tweening::Lens;
use lottie_core::prelude::{OpacityHierarchy, PathExt};
use lottie_core::{
    Animated, AnimatedExt, Bezier, TextBased, TextRangeInfo, TextRangeSelector,
    Transform as LottieTransform,
};
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
    pub(crate) zindex: f32,
    pub(crate) mask_offset: Vec2,
    pub(crate) text_range: Option<TextRangeInfo>,
}

impl Lens<Transform> for TransformLens {
    fn lerp(&mut self, target: &mut Transform, ratio: f32) {
        let frame = self.frames * ratio;
        let value = self.data.value(frame);
        *target = Transform::from_matrix(value);
        target.translation.z = self.zindex;
        target.translation.x += self.mask_offset.x;

        if let Some(info) = self.text_range.as_ref() {
            for range in &info.ranges {
                let appliable = lerp_index_in_text_range(
                    &range.selector,
                    frame,
                    &info.value,
                    info.index.0,
                    info.index.1,
                );
                if !appliable {
                    continue;
                }
                // TODO: support more selector attributes
                let styles = match range.style.as_ref() {
                    Some(s) => s,
                    None => continue,
                };
                let letter_spacing = styles
                    .letter_spacing
                    .as_ref()
                    .map(|l| l.value(frame))
                    .unwrap_or(0.0);
                target.translation.x += info.index.1 as f32 * letter_spacing;
            }
        }
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
        let opacity = frame * fill_opacity;

        if let Some(fill) = target.fill.as_mut() {
            fill.opacity = opacity;
        }
        if let Some(stroke) = target.stroke.as_mut() {
            stroke.opacity = opacity;
        }
    }
}

fn lerp_index_in_text_range(
    selector: &TextRangeSelector,
    frame: f32,
    value: &Vec<Vec<char>>,
    line: usize,
    c: usize,
) -> bool {
    let start = selector
        .start
        .as_ref()
        .map(|start| start.value(frame))
        .unwrap_or(0.0)
        .round() as usize;
    let end = selector
        .end
        .as_ref()
        .map(|end| end.value(frame))
        .unwrap_or(std::f32::MAX)
        .round() as usize;
    match selector.range_units {
        TextBased::Characters => {
            let index = (0..line).map(|i| value[i].len()).sum::<usize>() + c;
            return index >= start && index < end;
        }
        _ => unimplemented!(),
    }
}
