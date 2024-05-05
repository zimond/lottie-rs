use crate::model::*;
use fontkit::{Area, Line, PathSegment, Span};

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
    ) -> Result<Animated<RenderableContent>, Error> {
        let keyframes = text
            .document
            .keyframes
            .iter()
            .map(|keyframe| {
                let parser = TextDocumentParser::new(keyframe, &text.ranges, &model, fontdb)?;
                let shape = parser.shape_layer()?;
                let content = RenderableContent::Shape(ShapeGroup {
                    shapes: vec![shape],
                });
                Ok(keyframe.alter_value(content.clone(), content))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(Animated {
            animated: true,
            keyframes,
        })
    }
}

#[derive(Clone)]
struct Styles {
    fill: Rgb,
    fill_opacity: f32,
}

struct TextDocumentParser<'a> {
    model: &'a Model,
    fontdb: &'a FontDB,
    area: Area<Styles>,
    lottie_font: &'a Font,
    keyframe: &'a KeyFrame<TextDocument>,
    text_ranges: &'a Vec<TextRange>,
}

impl<'a> TextDocumentParser<'a> {
    fn new(
        keyframe: &'a KeyFrame<TextDocument>,
        text_ranges: &'a Vec<TextRange>,
        model: &'a Model,
        fontdb: &'a FontDB,
    ) -> Result<Self, Error> {
        let doc = &keyframe.start_value;
        let lottie_font = model
            .font(&doc.font_name)
            .ok_or_else(|| Error::FontFamilyNotFound(doc.font_name.clone()))?;
        let font = fontdb
            .font(lottie_font)
            .ok_or_else(|| Error::FontNotLoaded(doc.font_name.clone()))?;
        font.load()?;

        // parse fill/opacity data
        let rgb = Rgb::new_u8(doc.fill_color.r, doc.fill_color.g, doc.fill_color.b);

        let opacity = doc.fill_color.a as f32 / 255.0 * 100.0;
        let styles = Styles {
            fill: rgb,
            fill_opacity: opacity,
        };
        // parse font data
        let mut area = Area::new();
        for line in doc.value.split('\r') {
            let metrics = font.measure(&line)?;
            let span = Span {
                font_key: font.key(),
                letter_spacing: 0.0,
                line_height: None,
                size: doc.size,
                broke_from_prev: false,
                metrics,
                swallow_leading_space: false,
                additional: styles.clone(),
            };
            let line = Line::new(span);
            area.lines.push(line);
        }

        Ok(TextDocumentParser {
            model,
            fontdb,
            area,
            lottie_font,
            keyframe,
            text_ranges,
        })
    }

    fn shape_layer(&self) -> Result<ShapeLayer, Error> {
        let font = self
            .fontdb
            .font(self.lottie_font)
            .ok_or_else(|| Error::FontNotLoaded(self.lottie_font.name.clone()))?;
        font.load()?;
        let units = font.units_per_em() as f32;
        let doc = &self.keyframe.start_value;

        let mut result = vec![];
        let align_factor = match doc.justify {
            TextJustify::Left => 0.0,
            TextJustify::Center => -0.5,
            TextJustify::Right => -1.0,
            _ => 0.0, // TODO: support other TextJustify options
        };
        let start_shift_y = -doc.baseline_shift;
        let mut line_y = 0.0;
        let value = self
            .area
            .lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .flat_map(|span| span.metrics.positions().iter().map(|p| p.metrics.c))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        for (line_index, line) in self.area.lines.iter().enumerate() {
            let mut char_index = 0;
            let mut adv = line.width() * align_factor;
            for span in &line.spans {
                let factor = span.size / units;
                let mut all_beziers = vec![];

                // styles
                let fill = self
                    .keyframe
                    .alter_value(span.additional.fill, span.additional.fill);
                let fill_opacity = self
                    .keyframe
                    .alter_value(span.additional.fill_opacity, span.additional.fill_opacity);
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
                for c in span.metrics.positions() {
                    let (glyph, _) = font.outline(c.metrics.c).ok_or_else(|| {
                        Error::FontGlyphNotFound(self.lottie_font.name.clone(), c.metrics.c)
                    })?;
                    let mut bezier = Bezier::default();
                    let mut beziers = vec![];
                    let mut last_pt = Vector2D::new(0.0, 0.0);
                    let length = c.metrics.advanced_x as f32 * factor + c.kerning as f32 * factor;
                    let segments = glyph.path.finish();
                    let segments = segments
                        .as_ref()
                        .map(|p| p.segments())
                        .into_iter()
                        .flatten();
                    for segment in segments {
                        match segment {
                            PathSegment::MoveTo(p) => {
                                if !bezier.verticies.is_empty() {
                                    let mut old = std::mem::replace(&mut bezier, Bezier::default());
                                    old.out_tangent.push(Vector2D::new(0.0, 0.0));
                                    beziers.push(old);
                                }
                                bezier.in_tangent.push(Vector2D::new(0.0, 0.0));
                                last_pt = Vector2D::new(p.x, -p.y) * factor;
                                bezier.verticies.push(last_pt);
                            }
                            PathSegment::LineTo(p) => {
                                let pt = Vector2D::new(p.x, -p.y) * factor;
                                bezier.out_tangent.push(Vector2D::new(0.0, 0.0));
                                bezier.in_tangent.push(Vector2D::new(0.0, 0.0));
                                bezier.verticies.push(pt);
                                last_pt = pt;
                            }
                            PathSegment::CubicTo(p1, p2, p) => {
                                let pt1 = Vector2D::new(p1.x, -p1.y) * factor;
                                let pt2 = Vector2D::new(p2.x, -p2.y) * factor;
                                let pt = Vector2D::new(p.x, -p.y) * factor;

                                bezier.out_tangent.push(pt1 - last_pt);
                                bezier.in_tangent.push(pt2 - pt);
                                bezier.verticies.push(pt);
                                last_pt = pt;
                            }
                            PathSegment::QuadTo(p1, p) => {
                                let pt1 = Vector2D::new(p1.x, -p1.y) * factor;
                                let pt = Vector2D::new(p.x, -p.y) * factor;

                                bezier.out_tangent.push(pt1 - last_pt);
                                bezier.in_tangent.push(pt1 - pt);
                                bezier.verticies.push(pt);
                                last_pt = pt;
                            }
                            PathSegment::Close => {
                                bezier.closed = true;
                            }
                        }
                    }
                    if !bezier.verticies.is_empty() {
                        bezier.out_tangent.push(Vector2D::new(0.0, 0.0));
                        beziers.push(bezier);
                    }
                    all_beziers.push(GlyphData {
                        c: c.metrics.c,
                        beziers,
                        offset_x: adv,
                    });

                    adv += length;
                }

                let mut glyphs = all_beziers
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
                        let text_range = if self.text_ranges.is_empty() {
                            None
                        } else {
                            Some(TextRangeInfo {
                                value: value.clone(),
                                index: (line_index, char_index),
                                ranges: self.text_ranges.clone(),
                            })
                        };
                        char_index += 1;
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
                                                keyframes: vec![self
                                                    .keyframe
                                                    .alter_value(beziers.clone(), beziers)],
                                            },
                                            text_range,
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

                let shift = Vector2D::new(0.0, start_shift_y + line_y);
                let transform_position = self.keyframe.alter_value(shift, shift);
                let mut transform = Transform::default();
                transform.position = Some(Animated {
                    animated: false,
                    keyframes: vec![transform_position],
                });

                glyphs.push(ShapeLayer {
                    name: None,
                    hidden: false,
                    shape: Shape::Transform(transform),
                });
                let line_values = line
                    .spans()
                    .iter()
                    .map(|span| span.metrics.value().to_string())
                    .collect::<Vec<_>>();
                let line_value = line_values.join("");
                result.push(ShapeLayer {
                    name: Some(line_value),
                    hidden: false,
                    shape: Shape::Group { shapes: glyphs },
                });
            }
            line_y += line.height();
        }
        Ok(ShapeLayer {
            name: None,
            hidden: false,
            shape: Shape::Group { shapes: result },
        })
    }
}
