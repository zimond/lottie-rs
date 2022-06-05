use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Shape must have a sibling Transform")]
    ShapeTransformMissing,
    #[error("Malformed Lottie JSON file: {0}")]
    MalformedJSON(#[from] lottie_model::Error),
    #[error(transparent)]
    FontKit(#[from] fontkit::Error),
    #[error("Font family {0} used is not found")]
    FontFamilyNotFound(String),
}
