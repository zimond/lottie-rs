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
                vec![(RenderableContent::Shape(shape_group), None, None)]
            }
            LayerContent::PreCompositionRef(_)
            | LayerContent::Empty
            | LayerContent::MediaRef(_) => vec![(RenderableContent::Group, None, None)],
            LayerContent::Text(text) => match RenderableContent::from_text(&text, model, fontdb) {
                Ok(t) => t
                    .keyframes
                    .into_iter()
                    .map(|keyframe| {
                        (
                            keyframe.start_value,
                            Some(keyframe.start_frame),
                            Some(keyframe.end_frame),
                        )
                    })
                    .collect(),
                Err(e) => {
                    log::warn!("{:?}", e);
                    vec![(RenderableContent::Group, None, None)]
                }
            },
            LayerContent::SolidColor {
                color,
                height,
                width,
            } => vec![(
                RenderableContent::Shape(ShapeGroup {
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
                }),
                None,
                None,
            )],
            LayerContent::Media(media) => {
                vec![(
                    RenderableContent::Media(Media::new(media, Some(root_path))?),
                    None,
                    None,
                )]
            }
            _ => todo!(),
        };
        let mut transform = layer.transform.unwrap_or_default();
        transform.auto_orient = layer.auto_orient;
        if let Some(end) = content.last_mut().and_then(|(_, _, end)| end.as_mut()) {
            *end = layer.end_frame;
        }
        Ok(content
            .into_iter()
            .map(|(content, start, end)| StagedLayer {
                id: Id::default(),
                name: layer.name.clone(),
                content,
                zindex: 0.0,
                target: TargetRef::Layer(0),
                parent: None,
                start_frame: start.unwrap_or(layer.start_frame),
                end_frame: end.unwrap_or(layer.end_frame),
                transform: transform.clone(),
                frame_rate: 0.0,
                transform_hierarchy: TransformHierarchy::default(),
                frame_transform: FrameTransform::new(0.0, layer.start_time),
                frame_transform_hierarchy: FrameTransformHierarchy::default(),
                is_mask: false,
                mask_hierarchy: MaskHierarchy::default(),
                blend_mode: layer.blend_mode.unwrap_or(BlendMode::Normal),
            })
            .collect())
    }
}
