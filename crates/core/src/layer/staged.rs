use lottie_model::*;

use crate::font::FontDB;
use crate::prelude::Id;
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

#[derive(Debug, Clone)]
pub enum StagedLayerMask {
    None,
    IsMask,
    HasMask(Vec<StagedLayerMaskInfo>),
}

impl StagedLayerMask {
    pub fn is_mask(&self) -> bool {
        match self {
            StagedLayerMask::IsMask => true,
            _ => false,
        }
    }

    pub fn is_none(&self) -> bool {
        match self {
            StagedLayerMask::None => true,
            _ => false,
        }
    }

    pub fn masks(&self) -> Option<&[StagedLayerMaskInfo]> {
        match self {
            StagedLayerMask::HasMask(info) => Some(info.as_slice()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StagedLayerMaskInfo {
    pub mode: MatteMode,
    pub id: Id,
}

/// A wrapper for [Layer], ready to be rendered
#[derive(Debug, Clone)]
pub struct StagedLayer {
    pub id: Id,
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
    pub mask: StagedLayerMask,
}

impl StagedLayer {
    pub fn new(layer: Layer, model: &Model, fontdb: &FontDB) -> Result<Self, Error> {
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
            LayerContent::Media(media) => RenderableContent::Media(Media::new(media, None)?),
            _ => todo!(),
        };
        let mut transform = layer.transform.unwrap_or_default();
        transform.auto_orient = layer.auto_orient;
        Ok(StagedLayer {
            id: Id::default(),
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
            mask: StagedLayerMask::None,
        })
    }
}
