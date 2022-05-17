use lottie_ast::Animated;

#[derive(Default, Debug, Clone)]
pub struct OpacityHierarchy {
    pub(crate) stack: Vec<Animated<f32>>,
}
