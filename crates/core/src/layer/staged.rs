use lottie_ast::{Layer, LayerContent};

use crate::Lottie;

/// A wrapper for [Layer], ready to be rendered
pub struct StagedLayer<'a> {
    pub(crate) layer: &'a Layer,
    start_frame: u32,
    end_frame: u32,
    frame_rate: u32,
}

impl<'a> StagedLayer<'a> {
    pub fn new(layer: &'a Layer, frame_rate: u32) -> Self {
        StagedLayer {
            layer,
            start_frame: layer.start_frame,
            end_frame: layer.end_frame,
            frame_rate,
        }
    }

    pub fn content(&self) -> &LayerContent {
        &self.layer.content
    }

    pub fn start_frame(&self) -> u32 {
        self.start_frame
    }

    pub fn end_frame(&self) -> u32 {
        self.end_frame
    }

    pub fn frame_rate(&self) -> u32 {
        self.frame_rate
    }

    pub fn set_start_frame(&mut self, start: u32) {
        self.start_frame = start;
    }

    pub fn set_end_frame(&mut self, end: u32) {
        self.end_frame = end;
    }
}

pub trait StagedLayerExt<'a> {
    type Iter: Iterator<Item = StagedLayer<'a>>;
    /// Get all layers in this layer container, wrapped into [StagedLayer] with
    /// start/end frames adjusted
    fn layers(self) -> Self::Iter;
}

impl<'a> StagedLayerExt<'a> for &'a Lottie {
    type Iter = impl Iterator<Item = StagedLayer<'a>>;

    fn layers(self) -> Self::Iter {
        self.model
            .layers
            .iter()
            .map(|layer| StagedLayer::new(layer, self.model.frame_rate))
    }
}
