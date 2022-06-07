use fontkit::FontKit;
use lottie_model::*;

use crate::prelude::Id;
use crate::Error;

use super::frame::{FrameTransform, FrameTransformHierarchy};
use super::opacity::OpacityHierarchy;

#[derive(Debug, Clone)]
pub enum RenderableContent {
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
    pub id: Id,
    pub content: RenderableContent,
    pub target: TargetRef,
    pub start_frame: f32,
    pub end_frame: f32,
    pub frame_rate: f32,
    pub parent: Option<Id>,
    pub transform: Transform,
    pub zindex: f32,
    pub opacity: OpacityHierarchy,
    pub frame_transform: FrameTransform,
    pub frame_transform_hierarchy: FrameTransformHierarchy,
}

impl StagedLayer {
    pub fn new(layer: Layer, model: &Model, fontkit: &FontKit) -> Result<Self, Error> {
        let content = match layer.content {
            LayerContent::Shape(shape_group) => RenderableContent::Shape(shape_group),
            LayerContent::Precomposition(_) | LayerContent::Empty => RenderableContent::Group,
            LayerContent::Text(text) => RenderableContent::from_text(&text, model, fontkit)?,
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
            opacity: OpacityHierarchy::default(),
            frame_transform: FrameTransform::new(0.0, layer.start_time),
            frame_transform_hierarchy: FrameTransformHierarchy::default(),
        })
    }
}
