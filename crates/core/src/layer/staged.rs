use lottie_model::{Layer, LayerContent, ShapeGroup, Transform};

use crate::prelude::Id;

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
    pub start_frame: u32,
    pub end_frame: u32,
    pub frame_rate: u32,
    pub parent: Option<Id>,
    pub transform: Transform,
    pub opacity: OpacityHierarchy,
}

impl StagedLayer {
    pub fn new(layer: Layer) -> Self {
        let start_frame = layer.spawn_frame();
        let end_frame = layer.despawn_frame();
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
            target: TargetRef::Layer(0),
            parent: None,
            start_frame,
            end_frame,
            transform,
            frame_rate: 0,
            opacity: OpacityHierarchy::default(),
        }
    }
}
