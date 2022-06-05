#![feature(type_alias_impl_trait, let_chains)]

use std::io::Read;

pub use animated::*;
pub use error::Error;
use fontkit::FontKit;
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

pub struct Lottie {
    pub model: Model,
    pub scale: f32,
    fontkit: FontKit,
    timeline: Timeline,
}

impl Lottie {
    pub fn from_reader_with_fontkit<R: Read>(r: R, fontkit: FontKit) -> Result<Self, Error> {
        let mut model = Model::from_reader(r)?;
        let timeline = Timeline::new(&model, &fontkit)?;
        Ok(Lottie {
            model,
            timeline,
            scale: 1.0,
            fontkit,
        })
    }

    #[cfg(not(all(target_os = "unknown", target_arch = "wasm32")))]
    pub fn from_reader<R: Read>(r: R) -> Result<Self, Error> {
        let mut fontkit = FontKit::new();
        let path = dirs::font_dir().unwrap();
        fontkit.search_fonts_from_path(path)?;
        Ok(Lottie::from_reader_with_fontkit(r, fontkit)?)
    }

    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }
}
