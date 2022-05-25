use lottie_model::Animated;

use crate::AnimatedExt;

#[derive(Default, Debug, Clone)]
pub struct OpacityHierarchy {
    pub(crate) stack: Vec<Animated<f32>>,
}

impl AnimatedExt for OpacityHierarchy {
    type Target = f32;

    fn initial_value(&self) -> Self::Target {
        self.value(0.0)
    }

    fn value(&self, frame: f32) -> Self::Target {
        self.stack
            .iter()
            .fold(1.0, |current, item| current * item.value(frame) / 100.0)
    }

    fn is_animated(&self) -> bool {
        self.stack.iter().any(|item| item.is_animated())
    }
}
