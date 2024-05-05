use bevy::prelude::{Transform, Vec2};
use bevy_tweening_captured::{Lens, Targetable};
use lottie_core::prelude::{
    Animated, Bezier, OpacityHierarchy, PathFactory, TextBased, TextRangeInfo, TextRangeSelector,
    Transform as LottieTransform, TransformHierarchy, TrimInfo,
};
use lyon::algorithms::measure::PathMeasurements;
use lyon::algorithms::measure::SampleType::Normalized;
use lyon::geom::euclid::approxeq::ApproxEq;
use lyon::path::Path as LyonPath;

use crate::shape::{DrawMode, Path};

pub struct PathLens {
    pub(crate) start: Vec<Bezier>,
    pub(crate) end: Vec<Bezier>,
    pub(crate) trims: Vec<TrimInfo>,
    pub(crate) start_frame: f32,
    pub(crate) end_frame: f32,
}

impl Lens<Path> for PathLens {
    fn lerp(&mut self, target: &mut dyn Targetable<Path>, ratio: f32) {
        let frame = (self.end_frame - self.start_frame) * ratio + self.start_frame;
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
        let path = beziers.path(0.0);
        let mut start = 0.0f32;
        let mut end = 1.0f32;
        for trim in &self.trims {
            let offset = norm(trim.trim.offset.value(frame) / 360.0);
            let mut s = norm(trim.trim.start.value(frame) / 100.0 + offset);
            let mut e = norm(trim.trim.end.value(frame) / 100.0 + offset);
            if s < 0.0 {
                s += 1.0;
            }
            if e < 0.0 {
                e += 1.0;
            }
            let (s, e) = (s.min(e), s.max(e));
            start = start.max(s);
            end = end.min(e);
        }
        if start.approx_eq(&0.0) && end.approx_eq(&1.0) {
            target.0 = path;
        } else {
            let measures = PathMeasurements::from_path(&path, 1e-3);
            let mut sampler = measures.create_sampler(&path, Normalized);
            let mut builder = LyonPath::builder();
            sampler.split_range(start..end, &mut builder);
            let path = builder.build();
            target.0 = path;
        }
    }
}

fn norm(value: f32) -> f32 {
    if value > 0.0 && value.fract() == 0.0 {
        1.0
    } else {
        value.fract()
    }
}

pub struct StrokeWidthLens {
    pub(crate) start: f32,
    pub(crate) end: f32,
}

impl Lens<DrawMode> for StrokeWidthLens {
    fn lerp(&mut self, target: &mut dyn Targetable<DrawMode>, ratio: f32) {
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
    pub(crate) transform_hierarchy: TransformHierarchy,
    pub(crate) text_range: Option<TextRangeInfo>,
}

impl Lens<Transform> for TransformLens {
    fn lerp(&mut self, target: &mut dyn Targetable<Transform>, ratio: f32) {
        let frame = self.frames * ratio;
        let value = self.data.value(frame);
        *target.target_mut() = Transform::from_matrix(value);
        target.translation.z = self.zindex;
        target.translation.x += self.mask_offset.x / self.transform_hierarchy.scale_x(frame);

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
    fn lerp(&mut self, target: &mut dyn Targetable<DrawMode>, ratio: f32) {
        let frame = self.frames as f32 * ratio;
        let value = self.opacity.value(frame);
        let fill_opacity = self.fill_opacity.value(frame) / 100.0;

        if let Some(fill) = target.fill.as_mut() {
            fill.opacity = value * fill_opacity;
        }
        if let Some(stroke) = target.stroke.as_mut() {
            if let Some(stroke_opacity) =
                self.stroke_opacity.as_ref().map(|s| s.value(frame) / 100.0)
            {
                stroke.opacity = value * stroke_opacity;
            }
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

pub struct PathFactoryLens {
    pub(crate) start_frame: f32,
    pub(crate) end_frame: f32,
    pub(crate) factory: Box<dyn PathFactory + Send + Sync>,
}

impl Lens<Path> for PathFactoryLens {
    fn lerp(&mut self, target: &mut dyn Targetable<Path>, ratio: f32) {
        let frame = (self.end_frame - self.start_frame) * ratio + self.start_frame;
        target.0 = self.factory.path(frame);
    }
}
