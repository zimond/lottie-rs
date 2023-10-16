use crate::model::Animated;

#[derive(Debug, Clone)]
pub struct FrameTransform {
    pub time_remapping: Option<Animated<f32>>,
    pub frame_rate: f32,
    /// Maps to `Layer::start_time`
    pub frame_offset: f32,
}

impl FrameTransform {
    pub fn new(frame_rate: f32, frame_offset: f32) -> Self {
        FrameTransform {
            time_remapping: None,
            frame_rate,
            frame_offset,
        }
    }

    /// Remap a given global frame to local frame number
    pub fn transform(&self, frame: f32) -> f32 {
        if let Some(animated) = self.time_remapping.as_ref() {
            let f = animated.value(frame - self.frame_offset);
            f * self.frame_rate
        } else {
            frame - self.frame_offset
        }
    }
}

#[derive(Clone, Debug)]
pub struct FrameInfo {
    pub start_frame: f32,
    pub end_frame: f32,
    pub frame_transform: FrameTransform,
}

#[derive(Clone, Debug, Default)]
pub struct FrameTransformHierarchy {
    pub(crate) stack: Vec<FrameInfo>,
}

impl FrameTransformHierarchy {
    pub fn value(&self, mut frame: f32) -> Option<f32> {
        for item in &self.stack {
            if frame < item.start_frame || frame > item.end_frame {
                return None;
            }
            frame = item.frame_transform.transform(frame);
        }
        Some(frame)
    }

    pub fn frame_rate(&self) -> f32 {
        self.stack.last().unwrap().frame_transform.frame_rate
    }
}
