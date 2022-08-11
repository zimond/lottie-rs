use lottie_model::{Animated, Transform};

use crate::AnimatedExt;

#[derive(Default, Debug, Clone)]
pub struct TransformHierarchy {
    pub(crate) stack: Vec<Transform>,
}

pub struct OpacityHierarchy {
    stack: Vec<Animated<f32>>,
}

impl<'a> From<&'a TransformHierarchy> for OpacityHierarchy {
    fn from(t: &'a TransformHierarchy) -> Self {
        OpacityHierarchy {
            stack: t
                .stack
                .iter()
                .map(|transform| transform.opacity.clone())
                .collect(),
        }
    }
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
