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
    pub use crate::layer::frame::*;
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
}

impl Lottie {
    pub fn from_reader<R: std::io::Read>(r: R) -> Result<Self, lottie_model::Error> {
        let mut model = Model::from_reader(r)?;
        let timeline = Timeline::new(&model);
        Ok(Lottie {
            model,
            timeline,
            scale: 1.0,
        })
    }

    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }
}
