use fontkit::{FontKit, PathSegment};
use lottie_model::*;

use crate::font::FontLoader;
use crate::prelude::RenderableContent;
use crate::Error;

impl RenderableContent {
    pub fn from_text(
        text: &TextAnimationData,
        model: &Model,
        fontkit: &FontKit,
    ) -> Result<Self, Error> {
        let mut path_frames = vec![];
        let mut fill_frames = vec![];
        let mut fill_opacity_frames = vec![];
        for keyframe in &text.data.keyframes {
            let doc = &keyframe.start_value;
            let font = model
                .font(&doc.font_family)
                .ok_or_else(|| Error::FontFamilyNotFound(doc.font_family.clone()))?;
            let font = fontkit
                .fetch_font(font)
                .ok_or_else(|| Error::FontNotLoaded(doc.font_family.clone()))?;
            font.load()?;
            let metrics = font.measure(&doc.value, None)?;
            let units = font.units_per_em() as f32;
            let factor = doc.size / units;
            let mut beziers = vec![];
            for (c, metric) in doc.value.chars().zip(metrics.positions()) {
                let (glyph, _) = font
                    .outline(c)
                    .ok_or_else(|| Error::FontGlyphNotFound(doc.font_family.clone(), c))?;
                let mut bezier = Bezier::default();
                let mut last_pt = Vector2D::new(0.0, 0.0);
                let offset = Vector2D::new(metric.x_a as f32 * factor, 0.0);
                for segment in glyph.path.iter() {
                    match segment {
                        PathSegment::MoveTo { x, y } => {
                            if !bezier.verticies.is_empty() {
                                let mut old = std::mem::replace(&mut bezier, Bezier::default());
                                old.out_tangent.push(Vector2D::new(0.0, 0.0));
                                beziers.push(old);
                            }
                            bezier.in_tangent.push(Vector2D::new(0.0, 0.0));
                            last_pt = Vector2D::new(*x as f32, -*y as f32) * factor + offset;
                            bezier.verticies.push(last_pt);
                        }
                        PathSegment::LineTo { x, y } => {
                            let pt = Vector2D::new(*x as f32, -*y as f32) * factor + offset;
                            bezier.out_tangent.push(Vector2D::new(0.0, 0.0));
                            bezier.in_tangent.push(Vector2D::new(0.0, 0.0));
                            bezier.verticies.push(pt);
                            last_pt = pt;
                        }
                        PathSegment::CurveTo {
                            x1,
                            y1,
                            x2,
                            y2,
                            x,
                            y,
                        } => {
                            let pt1 = Vector2D::new(*x1 as f32, -*y1 as f32) * factor + offset;
                            let pt2 = Vector2D::new(*x2 as f32, -*y2 as f32) * factor + offset;
                            let pt = Vector2D::new(*x as f32, -*y as f32) * factor + offset;

                            bezier.out_tangent.push(pt1 - last_pt);
                            bezier.in_tangent.push(pt2 - pt);
                            bezier.verticies.push(pt);
                            last_pt = pt;
                        }
                        PathSegment::ClosePath => {
                            bezier.closed = true;
                        }
                    }
                }
                if !bezier.verticies.is_empty() {
                    bezier.out_tangent.push(Vector2D::new(0.0, 0.0));
                    beziers.push(bezier);
                }
            }

            path_frames.push(KeyFrame {
                start_value: beziers.clone(),
                end_value: beziers.clone(),
                start_frame: keyframe.start_frame,
                end_frame: keyframe.end_frame,
                easing_out: None,
                easing_in: None,
            });

            let rgb = Rgb::new_u8(doc.fill_color.r, doc.fill_color.g, doc.fill_color.b);
            fill_frames.push(KeyFrame {
                start_value: rgb,
                end_value: rgb,
                start_frame: keyframe.start_frame,
                end_frame: keyframe.end_frame,
                easing_out: None,
                easing_in: None,
            });
            let opacity = doc.fill_color.a as f32 / 255.0 * 100.0;
            fill_opacity_frames.push(KeyFrame {
                start_value: opacity,
                end_value: opacity,
                start_frame: keyframe.start_frame,
                end_frame: keyframe.end_frame,
                easing_out: None,
                easing_in: None,
            })
        }
        Ok(RenderableContent::Shape(ShapeGroup {
            shapes: vec![
                ShapeLayer {
                    name: None,
                    hidden: false,
                    shape: Shape::Path {
                        d: Animated {
                            animated: true,
                            keyframes: path_frames,
                        },
                    },
                },
                ShapeLayer {
                    name: None,
                    hidden: false,
                    shape: Shape::Fill(Fill {
                        opacity: Animated {
                            animated: true,
                            keyframes: fill_opacity_frames,
                        },
                        color: Animated {
                            animated: true,
                            keyframes: fill_frames,
                        },
                        fill_rule: FillRule::NonZero,
                    }),
                },
                ShapeLayer {
                    name: None,
                    hidden: false,
                    shape: Shape::Transform(Transform::default()),
                },
            ],
        }))
    }
}
