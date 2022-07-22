#![feature(type_alias_impl_trait, let_chains)]

use std::io::Read;

pub use animated::*;
pub use error::Error;
use font::FontDB;
use fontkit::FontKit;
pub use lottie_model::*;
pub use renderer::*;
use timeline::Timeline;

mod animated;
mod error;
mod font;
mod layer;
mod renderer;
mod timeline;

pub mod prelude {
    pub use crate::layer::frame::*;
    pub use crate::layer::opacity::OpacityHierarchy;
    pub use crate::layer::shape::{AnyFill, AnyStroke, PathExt, ShapeIterator, StyledShape};
    pub use crate::layer::staged::{RenderableContent, StagedLayer};
    pub use crate::timeline::{Id, TimelineAction};
}

pub struct Lottie {
    pub model: Model,
    pub scale: f32,
    fontdb: FontDB,
    timeline: Timeline,
}

impl Lottie {
    pub fn new(model: Model, fontkit: FontKit) -> Result<Self, Error> {
        let mut fontdb = FontDB::new(fontkit);
        fontdb.load_fonts_from_model(&model)?;

        let timeline = Timeline::new(&model, &fontdb)?;
        Ok(Lottie {
            model,
            timeline,
            fontdb,
            scale: 1.0,
        })
    }

    #[cfg(not(all(target_os = "unknown", target_arch = "wasm32")))]
    pub fn from_reader<R: Read>(r: R) -> Result<Self, Error> {
        let mut fontkit = FontKit::new();
        let path = dirs::font_dir().unwrap();
        fontkit.search_fonts_from_path(path)?;
        let model = Model::from_reader(r)?;
        Ok(Lottie::new(model, fontkit)?)
    }

    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }
}
