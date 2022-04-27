#![feature(type_alias_impl_trait)]

pub use animated::*;
pub use error::Error;
use layer::precomposition::PrecompositionContainer;
use layer::staged::{StagedLayer, StagedLayerExt};
pub use lottie_ast::*;
pub use renderer::*;

mod animated;
mod error;
mod layer;
pub mod prelude;
mod renderer;

#[derive(Clone)]
pub struct Lottie {
    pub model: Model,
    pub scale: f32,
    id_counter: u32,
}

impl Lottie {
    pub fn from_reader<R: std::io::Read>(r: R) -> Result<Self, lottie_ast::Error> {
        let mut model = Model::from_reader(r)?;
        // assign ids to shapes
        let mut id_counter = 1;
        for (index, layer) in model.layers.iter_mut().enumerate() {
            layer.id = index as u32;
            if let LayerContent::Shape(ShapeGroup { shapes }) = &mut layer.content {
                for layer in shapes {
                    assign_id(layer, &mut id_counter);
                }
            }
        }
        Ok(Lottie {
            model,
            id_counter,
            scale: 1.0,
        })
    }

    /// Get an iterator which outputs every actual rendered layer in this
    /// Lottie. More concisely, this method flattens precomposition layers
    /// into normal layers.
    pub fn flatten_layers(&self) -> impl Iterator<Item = StagedLayer<'_>> {
        self.layers()
            .filter_map(move |layer: StagedLayer<'_>| {
                Some(match &layer.layer.content {
                    LayerContent::Precomposition(pre) => {
                        let asset = self
                            .model
                            .assets
                            .iter()
                            .find(|asset| asset.id == pre.ref_id)?;
                        let pre = PrecompositionContainer {
                            asset,
                            layer: &layer.layer,
                            comp: &self,
                            ref_item: pre,
                        };
                        pre.layers().collect::<Vec<_>>()
                    }
                    _ => vec![layer],
                })
            })
            .flat_map(|s| s)
    }

    // pub fn query_asset_by_id(&self, id: &str) -> Option<PrecompositionContainer>
    // {     let asset = self.model.assets.iter().find(|asset| asset.id == id)?;
    //     Some(PrecompositionContainer { asset, comp: &self })
    // }
}

fn assign_id(layer: &mut ShapeLayer, id_counter: &mut u32) {
    layer.id = *id_counter;
    *id_counter += 1;
    if let Shape::Group { shapes } = &mut layer.shape {
        for shape in shapes {
            assign_id(shape, id_counter);
        }
    }
}
