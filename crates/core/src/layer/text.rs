use fontkit::PathSegment;
use lottie_model::*;

use crate::font::FontDB;
use crate::prelude::RenderableContent;
use crate::Error;

struct GlyphData {
    c: char,
    beziers: Vec<Bezier>,
    offset_x: f32,
}

impl RenderableContent {
    pub fn from_text(
        text: &TextAnimationData,
        model: &Model,
        fontdb: &FontDB,
    ) -> Result<Self, Error> {
        let mut glyph_layers = vec![];
        for keyframe in &text.document.keyframes {
            let doc = &keyframe.start_value;
            let font = model
                .font(&doc.font_name)
                .ok_or_else(|| Error::FontFamilyNotFound(doc.font_name.clone()))?;
            let font = fontdb
                .font(font)
                .ok_or_else(|| Error::FontNotLoaded(doc.font_name.clone()))?;
            font.load()?;

            // parse fill/opacity data
            let rgb = Rgb::new_u8(doc.fill_color.r, doc.fill_color.g, doc.fill_color.b);
            let fill = KeyFrame {
                start_value: rgb,
                end_value: rgb,
                start_frame: keyframe.start_frame,
                end_frame: keyframe.end_frame,
                easing_out: keyframe.easing_out.clone(),
                easing_in: keyframe.easing_in.clone(),
            };

            let opacity = doc.fill_color.a as f32 / 255.0 * 100.0;
            let fill_opacity = KeyFrame {
                start_value: opacity,
                end_value: opacity,
                start_frame: keyframe.start_frame,
                end_frame: keyframe.end_frame,
                easing_out: keyframe.easing_out.clone(),
                easing_in: keyframe.easing_in.clone(),
            };
            let fill_layer = ShapeLayer {
                name: None,
                hidden: false,
                shape: Shape::Fill(Fill {
                    opacity: Animated {
                        animated: false,
                        keyframes: vec![fill_opacity],
                    },
                    color: Animated {
                        animated: false,
                        keyframes: vec![fill],
                    },
                    fill_rule: FillRule::NonZero,
                }),
            };

            // parse font data
            let metrics = font.measure(&doc.value)?;
            let units = font.units_per_em() as f32;
            let factor = doc.size / units;
            let mut all_beziers = vec![];
            let mut adv = 0.0;
            for (c, metric) in doc.value.chars().zip(metrics.positions()) {
                let (glyph, _) = font
                    .outline(c)
                    .ok_or_else(|| Error::FontGlyphNotFound(doc.font_name.clone(), c))?;
                let mut bezier = Bezier::default();
                let mut beziers = vec![];
                let mut last_pt = Vector2D::new(0.0, 0.0);
                let length =
                    metric.metrics.advanced_x as f32 * factor + metric.kerning as f32 * factor;
                for segment in glyph.path.iter() {
                    match segment {
                        PathSegment::MoveTo { x, y } => {
                            if !bezier.verticies.is_empty() {
                                let mut old = std::mem::replace(&mut bezier, Bezier::default());
                                old.out_tangent.push(Vector2D::new(0.0, 0.0));
                                beziers.push(old);
                            }
                            bezier.in_tangent.push(Vector2D::new(0.0, 0.0));
                            last_pt = Vector2D::new(*x as f32, -*y as f32) * factor;
                            bezier.verticies.push(last_pt);
                        }
                        PathSegment::LineTo { x, y } => {
                            let pt = Vector2D::new(*x as f32, -*y as f32) * factor;
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
                            let pt1 = Vector2D::new(*x1 as f32, -*y1 as f32) * factor;
                            let pt2 = Vector2D::new(*x2 as f32, -*y2 as f32) * factor;
                            let pt = Vector2D::new(*x as f32, -*y as f32) * factor;

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
                all_beziers.push(GlyphData {
                    c,
                    beziers,
                    offset_x: adv,
                });

                adv += length;
            }

            let glyphs = all_beziers
                .into_iter()
                .map(|data| {
                    let GlyphData {
                        c,
                        beziers,
                        offset_x,
                    } = data;

                    let mut transform = Transform::default();
                    transform.position = Some(Animated {
                        animated: false,
                        keyframes: vec![KeyFrame::from_value(Vector2D::new(offset_x, 0.0))],
                    });
                    ShapeLayer {
                        name: Some(format!("{}", c)),
                        hidden: false,
                        shape: Shape::Group {
                            shapes: vec![
                                ShapeLayer {
                                    name: None,
                                    hidden: false,
                                    shape: Shape::Path {
                                        d: Animated {
                                            animated: false,
                                            keyframes: vec![KeyFrame {
                                                start_value: beziers.clone(),
                                                end_value: beziers,
                                                start_frame: keyframe.start_frame,
                                                end_frame: keyframe.end_frame,
                                                easing_out: keyframe.easing_out.clone(),
                                                easing_in: keyframe.easing_in.clone(),
                                            }],
                                        },
                                    },
                                },
                                fill_layer.clone(),
                                ShapeLayer {
                                    name: None,
                                    hidden: false,
                                    shape: Shape::Transform(transform),
                                },
                            ],
                        },
                    }
                })
                .collect::<Vec<_>>();
            let glyphs = ShapeLayer {
                name: None,
                hidden: false,
                shape: Shape::Group { shapes: glyphs },
            };

            let start_shift_x = match doc.justify {
                TextJustify::Left => 0.0,
                TextJustify::Center => -adv / 2.0,
                TextJustify::Right => -adv,
                _ => 0.0, // TODO: support other TextJustify options
            };
            let start_shift_y = -doc.baseline_shift;

            let transform_position = KeyFrame {
                start_value: Vector2D::new(start_shift_x, start_shift_y),
                end_value: Vector2D::new(start_shift_x, start_shift_y),
                start_frame: keyframe.start_frame,
                end_frame: keyframe.end_frame,
                easing_out: keyframe.easing_out.clone(),
                easing_in: keyframe.easing_in.clone(),
            };

            let mut transform = Transform::default();
            transform.position = Some(Animated {
                animated: false,
                keyframes: vec![transform_position],
            });
            glyph_layers.push(ShapeLayer {
                name: None,
                hidden: false,
                shape: Shape::Group {
                    shapes: vec![
                        glyphs,
                        ShapeLayer {
                            name: None,
                            hidden: false,
                            shape: Shape::Transform(transform),
                        },
                    ],
                },
            });
        }

        Ok(RenderableContent::Shape(ShapeGroup {
            shapes: glyph_layers,
        }))
    }
}
