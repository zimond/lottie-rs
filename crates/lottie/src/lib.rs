#![no_std]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

pub use tiny_skia_path;

use alloc::vec::Vec;
#[cfg(feature = "std")]
use alloc::string::ToString;
use alloc::string::String;

use crate::model::Model;
pub use error::Error;
use font::FontDB;
pub use lerp::*;
pub use renderer::*;
use timeline::Timeline;

mod error;
mod font;
mod layer;
mod lerp;
mod model;
mod renderer;
mod timeline;

#[cfg(feature = "std")]
use std::io::Read;

#[cfg(feature = "std")]
use crate::model::Asset;

pub type MediaAssets = hashbrown::HashMap<String, Vec<u8>>;

pub mod prelude {
    pub use crate::layer::frame::*;
    pub use crate::layer::hierarchy::*;
    pub use crate::layer::shape::{
        AnyFill, AnyStroke, PathFactory, StyledShape, StyledShapeIterator, TrimInfo,
    };
    pub use crate::layer::staged::{RenderableContent, StagedLayer};
    pub use crate::model::*;
    pub use crate::timeline::{Id, TimelineAction};
    pub use crate::{Lottie, MediaAssets};
}

pub struct Lottie {
    pub model: Model,
    pub scale: f32,
    fontdb: FontDB,
    timeline: Timeline,
}

impl Lottie {
    /// Initiate a new `Lottie` by providing a parsed `Model`.
    ///
    /// Any media bytes must be provided separately as a map keyed by the
    /// asset id. In no_std builds this is the only loading path; in std builds
    /// convenience wrappers can populate the map from files, URLs, or embedded
    /// base64 payloads.
    pub fn new(model: Model, media_assets: Option<&MediaAssets>) -> Result<Self, Error> {
        let mut fontdb = FontDB::new();
        fontdb.load_fonts_from_model(&model)?;

        let timeline = Timeline::new(&model, &fontdb, media_assets)?;
        Ok(Lottie {
            model,
            timeline,
            fontdb,
            scale: 1.0,
        })
    }

    #[cfg(feature = "std")]
    pub fn from_reader<R: Read>(r: R, root_path: &str) -> Result<Self, Error> {
        let model = Model::from_reader(r)?;
        let media_assets = resolve_media_assets(&model, root_path)?;
        Ok(Lottie::new(model, Some(&media_assets))?)
    }

    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }

    pub fn fontdb(&self) -> &FontDB {
        &self.fontdb
    }
}

#[cfg(feature = "std")]
fn resolve_media_assets(model: &Model, root_path: &str) -> Result<MediaAssets, Error> {
    use base64::engine::general_purpose;
    use base64::Engine as _;
    use std::path::PathBuf;
    use url::Url;

    let mut assets = MediaAssets::new();
    for asset in &model.assets {
        let media = match asset {
            Asset::Media(media) => media,
            _ => continue,
        };

        let content = if media.embedded {
            let mut content = media.filename.as_str();
            if let Some(pos) = content.find(',') {
                content = &content[pos + 1..];
            }
                general_purpose::STANDARD
                    .decode(content)
                    .map_err(Error::Base64Decode)?
        } else {
            let local_path = PathBuf::from(root_path).join(&media.filename);
            if local_path.exists() {
                std::fs::read(local_path)?
            } else {
                let url = match Url::parse(&media.pwd) {
                    Ok(url) => url.join(&media.filename),
                    Err(_) => Url::parse(&media.filename),
                }?;
                if url.scheme() == "data" {
                    let content = url.path().splitn(2, ',').nth(1).unwrap_or("");
                    general_purpose::STANDARD
                        .decode(content)
                        .map_err(Error::Base64Decode)?
                } else {
                    let url = url.as_str();
                    let response = ureq::get(url).call()?;
                    let len: usize = response
                        .header("Content-Length")
                        .ok_or_else(|| Error::NetworkMissingContentLength(url.to_string()))?
                        .parse()
                        .map_err(|_| Error::NetworkMalformedContentLength(url.to_string()))?;
                    let mut bytes: Vec<u8> = Vec::with_capacity(len);
                    response
                        .into_reader()
                        .take(len as u64)
                        .read_to_end(&mut bytes)?;
                    bytes
                }
            }
        };
        assets.insert(asset.id().to_string(), content);
    }
    Ok(assets)
}
