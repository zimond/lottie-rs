use lottie_ast::ShapeGroup;

#[derive(Debug, Clone)]
pub enum RenderableContent {
    Shape(ShapeGroup),
}

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum TargetRef {
    Layer(u32),
    Asset(String),
}

/// A wrapper for [Layer], ready to be rendered
#[derive(Debug, Clone)]
pub struct StagedLayer {
    pub content: RenderableContent,
    pub target: TargetRef,
    pub start_frame: u32,
    pub end_frame: u32,
    pub frame_rate: u32,
}
