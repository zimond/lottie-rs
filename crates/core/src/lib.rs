#![feature(type_alias_impl_trait)]

pub use animated::*;
pub use error::Error;
use layer::precomposition::PrecompositionContainer;
use layer::staged::StagedLayer;
pub use lottie_ast::*;
pub use renderer::*;
use timeline::Timeline;

mod animated;
mod error;
mod layer;
pub mod prelude;
mod renderer;
mod timeline;

#[derive(Clone)]
pub struct Lottie {
    pub model: Model,
    pub scale: f32,
    timeline: Timeline,
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
        let timeline = Timeline::new(&model);
        Ok(Lottie {
            model,
            timeline,
            id_counter,
            scale: 1.0,
        })
    }

    pub fn timeline(&self) -> &Timeline {
        &self.timeline
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
