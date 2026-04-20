use alloc::string::String;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Shape must have a sibling Transform")]
    ShapeTransformMissing,
    #[error("Malformed Lottie JSON file: {0}")]
    MalformedJSON(#[from] crate::model::Error),
    #[cfg(feature = "fontkit")]
    #[error(transparent)]
    FontKit(#[from] fontkit::Error),
    #[error("Font family {0} not found in `fonts` declaration")]
    FontFamilyNotFound(String),
    #[error("Font family {0} cannot be loaded")]
    FontNotLoaded(String),
    #[error("Font family {0} doesn't contain the glyph for {1}")]
    FontGlyphNotFound(String, char),
    #[cfg(feature = "std")]
    #[error(transparent)]
    Network(#[from] ureq::Error),
    #[cfg(feature = "std")]
    #[error("Url {0} response contains no Content-Length header")]
    NetworkMissingContentLength(String),
    #[cfg(feature = "std")]
    #[error("Url {0} response contains invalid Content-Length header")]
    NetworkMalformedContentLength(String),
    #[cfg(feature = "std")]
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[cfg(feature = "std")]
    #[error(transparent)]
    Url(#[from] url::ParseError),
    #[cfg(feature = "std")]
    #[error("base64 decode error: {0:?}")]
    Base64Decode(base64::DecodeError),
    #[error("Media asset {0} was referenced but no bytes were provided")]
    MediaNotLoaded(String),
}
