use lottie_ast::{Layer, PreCompositionRef, Precomposition};

use crate::Lottie;

use super::staged::{StagedLayer, StagedLayerExt};

/// A wrapper representing a precomposition
pub struct PrecompositionContainer<'a> {
    pub(crate) asset: &'a Precomposition,
    pub(crate) comp: &'a Lottie,
    pub(crate) ref_item: &'a PreCompositionRef,
    pub(crate) layer: &'a Layer,
}

impl<'a> StagedLayerExt<'a> for PrecompositionContainer<'a> {
    type Iter = impl Iterator<Item = StagedLayer<'a>>;

    fn layers(self) -> Self::Iter {
        self.asset.layers.iter().map(|layer| {
            let mut staged = StagedLayer::new(
                layer,
                self.asset
                    .frame_rate
                    .unwrap_or_else(|| self.comp.model.frame_rate),
            );
            staged.set_start_frame(self.layer.start_frame + layer.start_frame);
            staged.set_end_frame(self.layer.end_frame + layer.end_frame);
            staged
        })
    }
}
