use lottie_model::*;

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
    pub fn new(
        layer: Layer,
        model: &Model,
        fontdb: &FontDB,
        root_path: &str,
    ) -> Result<Self, Error> {
        let content = match layer.content {
            LayerContent::Shape(shape_group) => RenderableContent::Shape(shape_group),
            LayerContent::PreCompositionRef(_)
            | LayerContent::Empty
            | LayerContent::MediaRef(_) => RenderableContent::Group,
            LayerContent::Text(text) => match RenderableContent::from_text(&text, model, fontdb) {
                Ok(t) => t,
                Err(e) => {
                    log::warn!("{:?}", e);
                    RenderableContent::Group
                }
            },
            LayerContent::SolidColor {
                color,
                height,
                width,
            } => RenderableContent::Shape(ShapeGroup {
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
            LayerContent::Media(media) => {
                RenderableContent::Media(Media::new(media, Some(root_path))?)
            }
            _ => todo!(),
        };
        let mut transform = layer.transform.unwrap_or_default();
        transform.auto_orient = layer.auto_orient;
        Ok(StagedLayer {
            id: Id::default(),
            name: layer.name.clone(),
            content,
            zindex: 0.0,
            target: TargetRef::Layer(0),
            parent: None,
            start_frame: layer.start_frame,
            end_frame: layer.end_frame,
            transform,
            frame_rate: 0.0,
            transform_hierarchy: TransformHierarchy::default(),
            frame_transform: FrameTransform::new(0.0, layer.start_time),
            frame_transform_hierarchy: FrameTransformHierarchy::default(),
            is_mask: false,
            mask_hierarchy: MaskHierarchy::default(),
            blend_mode: layer.blend_mode.unwrap_or(BlendMode::Normal),
        })
    }
}
