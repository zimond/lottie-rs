use std::io::Read;

use crate::model::Model;
pub use animated::*;
pub use error::Error;
use font::FontDB;
pub use fontkit::tiny_skia_path;
use fontkit::FontKit;
pub use renderer::*;
use timeline::Timeline;

mod animated;
mod error;
mod font;
mod layer;
mod model;
mod renderer;
mod timeline;

pub mod prelude {
    pub use crate::layer::frame::*;
    pub use crate::layer::hierarchy::*;
    pub use crate::layer::shape::{
        AnyFill, AnyStroke, PathExt, StyledShape, StyledShapeIterator, TrimInfo,
    };
    pub use crate::layer::staged::{RenderableContent, StagedLayer};
    pub use crate::model::*;
    pub use crate::timeline::{Id, TimelineAction};
}

pub struct Lottie {
    pub model: Model,
    pub scale: f32,
    fontdb: FontDB,
    timeline: Timeline,
}

impl Lottie {
    /// Initiate a new `Lottie` by providing a raw `Model`, a `FontKit` for font
    /// management, and a root path.Root path will be used to resolve relative
    /// paths of media files in this lottie model
    pub fn new(model: Model, fontkit: FontKit, root_path: &str) -> Result<Self, Error> {
        let mut fontdb = FontDB::new(fontkit);
        fontdb.load_fonts_from_model(&model)?;

        let timeline = Timeline::new(&model, &fontdb, root_path)?;
        Ok(Lottie {
            model,
            timeline,
            fontdb,
            scale: 1.0,
        })
    }

    #[cfg(not(all(target_os = "unknown", target_arch = "wasm32")))]
    pub fn from_reader<R: Read>(r: R, root_path: &str) -> Result<Self, Error> {
        let mut fontkit = FontKit::new();
        let path = dirs::font_dir().unwrap();
        fontkit.search_fonts_from_path(path)?;
        #[cfg(target_os = "macos")]
        fontkit.search_fonts_from_path(std::path::PathBuf::from("/System/Library/Fonts"))?;
        let model = Model::from_reader(r)?;
        Ok(Lottie::new(model, fontkit, root_path)?)
    }

    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }

    pub fn fontdb(&self) -> &FontDB {
        &self.fontdb
    }
}
