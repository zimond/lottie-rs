use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Shape must have a sibling Transform")]
    ShapeTransformMissing,
    #[error("Malformed Lottie JSON file: {0}")]
    MalformedJSON(#[from] crate::model::Error),
    #[error(transparent)]
    FontKit(#[from] fontkit::Error),
    #[error("Font family {0} not found in `fonts` declaration")]
    FontFamilyNotFound(String),
    #[error("Font family {0} cannot be loaded")]
    FontNotLoaded(String),
    #[error("Font family {0} doesn't contain the glyph for {1}")]
    FontGlyphNotFound(String, char),
    #[error(transparent)]
    Network(#[from] ureq::Error),
    #[error("Url {0} response contains no Content-Length header")]
    NetworkMissingContentLength(String),
    #[error("Url {0} response contains invalid Content-Length header")]
    NetworkMalformedContentLength(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Url(#[from] url::ParseError),
    #[error(transparent)]
    Base64Decode(#[from] base64::DecodeError),
}
