use lottie_model::Layer;

pub mod frame;
pub mod opacity;
pub mod shape;
pub mod staged;

pub trait LayerExt {
    fn adjust_frame_rate(&mut self, frame_rate: u32);
    fn spawn_frame(&self) -> f32;
    fn despawn_frame(&self) -> f32;
}

impl LayerExt for Layer {
    fn adjust_frame_rate(&mut self, frame_rate: u32) {
        todo!()
    }

    fn spawn_frame(&self) -> f32 {
        self.start_frame + self.start_time
    }

    fn despawn_frame(&self) -> f32 {
        self.end_frame + self.start_time
    }
}
