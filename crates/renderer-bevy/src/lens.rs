use bevy::math::Vec2;
use bevy_prototype_lyon::prelude::{DrawMode, Path, PathBuilder};
use bevy_tweening::Lens;
use lottie_core::Bezier;

pub struct PathLens {
    pub(crate) start: Vec<Bezier>,
    pub(crate) end: Vec<Bezier>,
}

impl Lens<Path> for PathLens {
    fn lerp(&mut self, target: &mut Path, ratio: f32) {
        let mut builder = PathBuilder::new();
        for (start, end) in self.start.iter().zip(self.end.iter()) {
            let mut result = start.clone();
            for index in 0..result.verticies.len() {
                result.verticies[index] += (end.verticies[index] - start.verticies[index]) * ratio;
                result.in_tangent[index] +=
                    (end.in_tangent[index] - start.in_tangent[index]) * ratio;
                result.out_tangent[index] +=
                    (end.out_tangent[index] - start.out_tangent[index]) * ratio;
            }
            builder.move_to(Vec2::new(result.verticies[0].x, result.verticies[0].y));
            for index in 0..(result.verticies.len() - 1) {
                let c1 = result.verticies[index] + result.out_tangent[index];
                let c2 = result.verticies[index + 1] + result.in_tangent[index + 1];
                let to = result.verticies[index + 1];
                builder.cubic_bezier_to(
                    Vec2::new(c1.x, c1.y),
                    Vec2::new(c2.x, c2.y),
                    Vec2::new(to.x, to.y),
                );
            }
            if result.closed {
                builder.close();
            }
        }
        *target = builder.build();
    }
}

pub struct StrokeWidthLens {
    pub(crate) start: f32,
    pub(crate) end: f32,
}

impl Lens<DrawMode> for StrokeWidthLens {
    fn lerp(&mut self, target: &mut DrawMode, ratio: f32) {
        let w = self.start + (self.end - self.start) * ratio;
        match target {
            DrawMode::Stroke(s) => s.options.line_width = w,
            DrawMode::Outlined { outline_mode, .. } => outline_mode.options.line_width = w,
            _ => {}
        }
    }
}
