#![feature(type_alias_impl_trait, let_chains)]

pub use animated::*;
pub use error::Error;
pub use lottie_model::*;
pub use renderer::*;
use timeline::Timeline;

mod animated;
mod error;
mod layer;
mod renderer;
mod timeline;

pub mod prelude {
    pub use crate::layer::opacity::OpacityHierarchy;
    pub use crate::layer::shape::{PathExt, ShapeIterator, StyledShape};
    pub use crate::layer::staged::{RenderableContent, StagedLayer};
    pub use crate::timeline::{Id, TimelineAction};
}

#[derive(Clone)]
pub struct Lottie {
    pub model: Model,
    pub scale: f32,
    timeline: Timeline,
    id_counter: u32,
}

impl Lottie {
    pub fn from_reader<R: std::io::Read>(r: R) -> Result<Self, lottie_model::Error> {
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
