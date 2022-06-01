use lottie_model::{Animated, Layer, LayerContent, ShapeGroup, Transform};

use crate::prelude::Id;
use crate::AnimatedExt;

use super::frame::{FrameTransform, FrameTransformHierarchy};
use super::opacity::OpacityHierarchy;
use super::LayerExt;

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
    pub fn new(layer: Layer) -> Self {
        let content = match layer.content {
            LayerContent::Shape(shape_group) => RenderableContent::Shape(shape_group),
            LayerContent::Precomposition(_) | LayerContent::Empty => RenderableContent::Group,
            _ => todo!(),
        };
        let mut transform = layer.transform.unwrap_or_default();
        transform.auto_orient = layer.auto_orient;
        StagedLayer {
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
        }
    }
}
