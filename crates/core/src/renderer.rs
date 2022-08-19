use crate::Lottie;

pub struct WindowConfig {
    pub show_controls: bool,
    pub show_debug: bool,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Target {
    Default,
    Mask,
}

pub struct HeadlessConfig {
    pub target: Target,
}

pub enum Config {
    Window(WindowConfig),
    Headless(HeadlessConfig),
}

/// The fundamental trait that every renderer need to implement
pub trait Renderer {
    /// Load a [Lottie] into this renderer
    fn load_lottie(&mut self, lottie: Lottie, config: Config);
    /// Render the lottie file, possibly mutating self
    fn render(&mut self);
}
