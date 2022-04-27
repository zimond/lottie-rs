use crate::Lottie;

/// The fundamental trait that every renderer need to implement
pub trait Renderer {
    /// Load a [Lottie] into this renderer
    fn load_lottie(&mut self, lottie: Lottie);
    /// Render the lottie file, possibly mutating self
    fn render(&mut self);
}
