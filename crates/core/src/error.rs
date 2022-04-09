use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Shape must have a sibling Transform")]
    ShapeTransformMissing,
    #[error("Malformed Lottie JSON file: {0}")]
    MalformedJASN(#[from] lottie_ast::Error),
}
