use crate::model::{Animated, MatteMode, Transform};

use crate::prelude::Id;

#[derive(Default, Debug, Clone)]
pub struct TransformHierarchy {
    // FIXME: from parenting, which breaks opacity hierarchy
    pub(crate) stack: Vec<Transform>,
}

impl TransformHierarchy {
    pub fn scale_x(&self, frame: f32) -> f32 {
        self.stack
            .iter()
            .map(|t| t.value(frame).x_axis.x)
            .fold(1.0, |current, i| current * i)
    }
}

pub struct OpacityHierarchy {
    stack: Vec<Animated<f32>>,
}

impl<'a> From<&'a TransformHierarchy> for OpacityHierarchy {
    fn from(t: &'a TransformHierarchy) -> Self {
        OpacityHierarchy {
            stack: t.stack.iter().map(|t| t.opacity.clone()).collect(),
        }
    }
}

impl OpacityHierarchy {
    pub fn initial_value(&self) -> f32 {
        self.value(0.0)
    }

    pub fn value(&self, frame: f32) -> f32 {
        self.stack
            .iter()
            .fold(1.0, |current, item| current * item.value(frame) / 100.0)
    }

    pub fn is_animated(&self) -> bool {
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
