use crate::model::*;

use crate::font::FontDB;
use crate::prelude::{Id, MaskHierarchy};
use crate::Error;

use super::frame::{FrameTransform, FrameTransformHierarchy};
use super::hierarchy::TransformHierarchy;
use super::media::Media;

#[derive(Debug, Clone)]
pub enum RenderableContent {
    Media(Media),
    Shape(ShapeGroup),
    Group,
}

impl RenderableContent {
    pub(crate) fn into_stage_layer(self, layer: &Layer) -> StagedLayer {
        let mut transform = layer.transform.clone().unwrap_or_default();
        transform.auto_orient = layer.auto_orient;
        StagedLayer {
            id: Id::default(),
            name: layer.name.clone(),
            content: self,
            zindex: 0.0,
            target: TargetRef::Layer(0),
            parent: None,
            start_frame: layer.start_frame,
            end_frame: layer.end_frame,
            transform: transform.clone(),
            frame_rate: 0.0,
            transform_hierarchy: TransformHierarchy::default(),
            frame_transform: FrameTransform::new(0.0, layer.start_time),
            frame_transform_hierarchy: FrameTransformHierarchy::default(),
            is_mask: false,
            matte_mode: layer.matte_mode,
            mask_hierarchy: MaskHierarchy::default(),
            blend_mode: layer.blend_mode.unwrap_or(BlendMode::Normal),
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum TargetRef {
    Layer(u32),
    Asset(String),
}

pub(crate) enum ContentInfo {
    Simple(RenderableContent),
    ContentWithMasks {
        content: RenderableContent,
        masks: Vec<(RenderableContent, MatteMode)>,
    },
    TextKeyframes(Vec<TextKeyframe>),
}

pub(crate) struct TextKeyframe {
    pub content: RenderableContent,
    pub start_frame: f32,
    pub end_frame: f32,
}

/// A wrapper for [Layer], ready to be rendered
#[derive(Debug, Clone)]
pub struct StagedLayer {
    /// Unique Id across the model
    pub id: Id,
    pub name: Option<String>,
    /// Content that could be rendered, including media, shapes and groups
    pub content: RenderableContent,
    pub target: TargetRef,
    pub start_frame: f32,
    pub end_frame: f32,
    pub frame_rate: f32,
    pub parent: Option<Id>,
    pub zindex: f32,
    pub transform: Transform,
    pub transform_hierarchy: TransformHierarchy,
    pub frame_transform: FrameTransform,
    pub frame_transform_hierarchy: FrameTransformHierarchy,
    /// Mask info of this layer
    pub is_mask: bool,
    pub matte_mode: Option<MatteMode>,
    pub mask_hierarchy: MaskHierarchy,
    pub blend_mode: BlendMode,
}

impl ContentInfo {
    pub fn from_layer(
        layer: Layer,
        model: &Model,
        fontdb: &FontDB,
        root_path: &str,
    ) -> Result<ContentInfo, Error> {
        let content = match layer.content.clone() {
            LayerContent::Shape(shape_group) => {
                let mut result = vec![];
                let content = RenderableContent::Shape(shape_group);
                if layer.has_mask {
                    for mask in &layer.masks_properties {
                        let mut opacity = mask.opacity.clone();
                        for keyframe in &mut opacity.keyframes {
                            keyframe.start_value /= 100.0;
                            keyframe.end_value /= 100.0;
                        }
                        let content = RenderableContent::Shape(ShapeGroup {
                            shapes: vec![
                                ShapeLayer {
                                    name: Some(mask.name.clone()),
                                    hidden: false,
                                    shape: Shape::Path {
                                        d: mask.points.clone(),
                                        text_range: None,
                                    },
                                },
                                ShapeLayer {
                                    name: None,
                                    hidden: false,
                                    shape: Shape::Fill(Fill {
                                        opacity,
                                        color: Animated {
                                            animated: false,
                                            keyframes: vec![KeyFrame::from_value(Rgb::new_u8(
                                                0, 0, 0,
                                            ))],
                                        },
                                        fill_rule: FillRule::EvenOdd,
                                    }),
                                },
                                ShapeLayer {
                                    name: None,
                                    hidden: false,
                                    shape: Shape::Transform(Transform::default()),
                                },
                            ],
                        });
                        let matte_mode = match mask.mode {
                            MaskMode::Add => MatteMode::Alpha,
                            MaskMode::Subtract => MatteMode::InvertedAlpha,
                            MaskMode::None => MatteMode::Normal,
                            _ => unimplemented!(),
                        };
                        result.push((content, matte_mode));
                    }
                    ContentInfo::ContentWithMasks {
                        content,
                        masks: result,
                    }
                } else {
                    ContentInfo::Simple(content)
                }
            }
            LayerContent::PreCompositionRef(_)
            | LayerContent::Empty
            | LayerContent::MediaRef(_) => ContentInfo::Simple(RenderableContent::Group.into()),
            LayerContent::Text(text) => match RenderableContent::from_text(&text, model, fontdb) {
                Ok(t) => ContentInfo::TextKeyframes(
                    t.keyframes
                        .into_iter()
                        .map(|keyframe| TextKeyframe {
                            content: keyframe.start_value,
                            start_frame: keyframe.start_frame,
                            end_frame: keyframe.end_frame,
                        })
                        .collect(),
                ),
                Err(e) => {
                    log::warn!("{:?}", e);
                    ContentInfo::Simple(RenderableContent::Group)
                }
            },
            LayerContent::SolidColor {
                color,
                height,
                width,
            } => ContentInfo::Simple(RenderableContent::Shape(ShapeGroup {
                shapes: vec![
                    ShapeLayer {
                        name: None,
                        hidden: false,
                        shape: Shape::Rectangle(Rectangle {
                            direction: ShapeDirection::Clockwise,
                            position: Animated::from_value(Vector2D::new(width, height) / 2.0),
                            size: Animated::from_value(Vector2D::new(width, height)),
                            radius: Animated::from_value(0.0),
                        }),
                    },
                    ShapeLayer {
                        name: None,
                        hidden: false,
                        shape: Shape::Fill(color.into()),
                    },
                ],
            })),
            LayerContent::Media(media) => ContentInfo::Simple(RenderableContent::Media(
                Media::new(media, Some(root_path))?,
            )),
            _ => todo!(),
        };
        Ok(content)
    }
}
