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

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum TargetRef {
    Layer(u32),
    Asset(String),
}

struct ContentInfo {
    content: RenderableContent,
    start_frame: Option<f32>,
    end_frame: Option<f32>,
    matte_mode: Option<MatteMode>,
}

impl From<RenderableContent> for ContentInfo {
    fn from(content: RenderableContent) -> Self {
        ContentInfo {
            content,
            start_frame: None,
            end_frame: None,
            matte_mode: None,
        }
    }
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

impl StagedLayer {
    pub fn from_layer(
        layer: Layer,
        model: &Model,
        fontdb: &FontDB,
        root_path: &str,
    ) -> Result<Vec<Self>, Error> {
        let mut content = match layer.content {
            LayerContent::Shape(shape_group) => {
                let mut result = vec![];
                let mut matte_mode = layer.matte_mode;
                if layer.has_mask {
                    for mask in &layer.masks_properties {
                        let d = Animated {
                            animated: mask.points.animated,
                            keyframes: mask
                                .points
                                .keyframes
                                .iter()
                                .map(|b| {
                                    b.alter_value(
                                        vec![b.start_value.clone()],
                                        vec![b.end_value.clone()],
                                    )
                                })
                                .collect(),
                        };
                        let mut opacity = mask.opacity.clone();
                        for keyframe in &mut opacity.keyframes {
                            keyframe.start_value /= 100.0;
                            keyframe.end_value /= 100.0;
                        }
                        result.push(ContentInfo {
                            content: RenderableContent::Shape(ShapeGroup {
                                shapes: vec![
                                    ShapeLayer {
                                        name: Some(mask.name.clone()),
                                        hidden: false,
                                        shape: Shape::Path {
                                            d,
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
                            }),
                            start_frame: None,
                            end_frame: None,
                            matte_mode,
                        });
                        matte_mode = Some(match mask.mode {
                            MaskMode::Add => MatteMode::Alpha,
                            MaskMode::Subtract => MatteMode::InvertedAlpha,
                            MaskMode::None => MatteMode::Normal,
                            _ => unimplemented!(),
                        })
                    }
                }
                let mut layer_info = ContentInfo::from(RenderableContent::Shape(shape_group));
                layer_info.matte_mode = matte_mode;
                result.push(layer_info);
                result
            }
            LayerContent::PreCompositionRef(_)
            | LayerContent::Empty
            | LayerContent::MediaRef(_) => vec![RenderableContent::Group.into()],
            LayerContent::Text(text) => match RenderableContent::from_text(&text, model, fontdb) {
                Ok(t) => t
                    .keyframes
                    .into_iter()
                    .map(|keyframe| ContentInfo {
                        content: keyframe.start_value,
                        start_frame: Some(keyframe.start_frame),
                        end_frame: Some(keyframe.end_frame),
                        matte_mode: None,
                    })
                    .collect(),
                Err(e) => {
                    log::warn!("{:?}", e);
                    vec![RenderableContent::Group.into()]
                }
            },
            LayerContent::SolidColor {
                color,
                height,
                width,
            } => vec![
                (RenderableContent::Shape(ShapeGroup {
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
                })
                .into()),
            ],
            LayerContent::Media(media) => {
                vec![RenderableContent::Media(Media::new(media, Some(root_path))?).into()]
            }
            _ => todo!(),
        };
        let mut transform = layer.transform.unwrap_or_default();
        transform.auto_orient = layer.auto_orient;
        if let Some(end) = content.last_mut().and_then(|info| info.end_frame.as_mut()) {
            *end = layer.end_frame;
        }
        Ok(content
            .into_iter()
            .map(|info| StagedLayer {
                id: Id::default(),
                name: layer.name.clone(),
                content: info.content,
                zindex: 0.0,
                target: TargetRef::Layer(0),
                parent: None,
                start_frame: info.start_frame.unwrap_or(layer.start_frame),
                end_frame: info.end_frame.unwrap_or(layer.end_frame),
                transform: transform.clone(),
                frame_rate: 0.0,
                transform_hierarchy: TransformHierarchy::default(),
                frame_transform: FrameTransform::new(0.0, layer.start_time),
                frame_transform_hierarchy: FrameTransformHierarchy::default(),
                is_mask: false,
                matte_mode: info.matte_mode,
                mask_hierarchy: MaskHierarchy::default(),
                blend_mode: layer.blend_mode.unwrap_or(BlendMode::Normal),
            })
            .collect())
    }
}
