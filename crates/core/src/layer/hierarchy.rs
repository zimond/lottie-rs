use lottie_model::{Animated, MatteMode, Transform};

use crate::prelude::Id;
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

#[derive(Debug, Clone, Copy)]
pub struct StagedLayerMask {
    pub mode: MatteMode,
    pub id: Id,
}

#[derive(Debug, Clone, Default)]
pub struct MaskHierarchy {
    pub(crate) stack: Vec<StagedLayerMask>,
}

impl MaskHierarchy {
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn masks(&self) -> &[StagedLayerMask] {
        &self.stack
    }
}
