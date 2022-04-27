use lottie_ast::{Animated, KeyFrame};

pub trait AnimatedExt {
    type Target;
    fn initial_value(&self) -> Self::Target;
    fn value(&self, frame: u32) -> Self::Target;
    fn is_animated(&self) -> bool;
}

impl<T> AnimatedExt for Animated<T>
where
    T: Clone,
{
    type Target = T;

    fn initial_value(&self) -> Self::Target {
        self.keyframes[0].value.clone()
    }

    fn value(&self, mut frame: u32) -> Self::Target {
        if !self.is_animated() {
            return self.initial_value();
        }
        let len = self.keyframes.len() - 1;
        frame = std::cmp::max(self.keyframes[0].start_frame.unwrap_or(0), frame);
        frame = std::cmp::min(self.keyframes[len].start_frame.unwrap_or(0), frame);
        if let Some(window) = self.keyframes.windows(2).find(|window| {
            frame >= window[0].start_frame.unwrap() && frame < window[1].start_frame.unwrap()
        }) {
            window[0].value.clone()
        } else {
            self.keyframes[len].value.clone()
        }
    }

    fn is_animated(&self) -> bool {
        self.animated
    }
}

pub trait KeyFrameExt {
    fn alter_value<U>(&self, value: U) -> KeyFrame<U>;
}

impl<T> KeyFrameExt for KeyFrame<T> {
    fn alter_value<U>(&self, value: U) -> KeyFrame<U> {
        KeyFrame {
            value,
            start_frame: self.start_frame.clone(),
            easing_out: self.easing_out.clone(),
            easing_in: self.easing_in.clone(),
        }
    }
}
