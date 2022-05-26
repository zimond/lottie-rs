use std::time::Duration;

use bevy::math::Vec2;
use bevy::prelude::{Entity, Transform};
use bevy_prototype_lyon::prelude::tess::path::path::Builder;
use bevy_prototype_lyon::prelude::*;
use bevy_tweening::{Animator, EaseMethod, Sequence, Tracks, Tween, TweeningType};
use lottie_core::*;

use lottie_core::prelude::*;
use lottie_core::Transform as LottieTransform;

use crate::lens::{OpacityLens, PathLens, StrokeWidthLens, TransformLens};
use crate::tween::TweenProducer;
use crate::*;

pub trait LayerRenderer {
    fn spawn(&self, frame: f32, commands: &mut Commands) -> Entity;
    fn spawn_shape(
        &self,
        frame: f32,
        zindex: f32,
        shape: StyledShape,
        commands: &mut Commands,
    ) -> Option<Entity>;
    fn transform_animator(&self, transform: &LottieTransform) -> Option<Animator<Transform>>;
    fn draw_mode_animator(&self, shape: &StyledShape) -> Option<Animator<DrawMode>>;
    fn sync_animator<T: Component>(&self, animator: &mut Animator<T>, frame: f32);
}

impl LayerRenderer for StagedLayer {
    fn spawn(&self, frame: f32, commands: &mut Commands) -> Entity {
        let mut c = commands.spawn();
        let mut initial_transform = Transform::from_matrix(self.transform.value(0.0));
        initial_transform.translation.z = self.zindex as f32 * -1.0;

        log::trace!(
            "spawn layer {:?}: start {}, end {}, transform: {:?}",
            c.id(),
            self.start_frame,
            self.end_frame,
            initial_transform
        );
        match &self.content {
            RenderableContent::Shape(shapes) => {
                let shapes = shapes.shapes();
                let count = shapes.shape_count() as f32;
                for (zindex, shape) in shapes.enumerate() {
                    if let Some(entity) = self.spawn_shape(
                        frame,
                        zindex as f32 / count + self.zindex as f32,
                        shape,
                        c.commands(),
                    ) {
                        log::trace!("layer {:?} get a child {:?}", c.id(), entity);
                        c.add_child(entity);
                    }
                }
            }
            RenderableContent::Group => {}
            _ => todo!(),
        }
        c.insert_bundle(TransformBundle {
            local: initial_transform,
            global: Default::default(),
        });
        if let Some(animator) = self.transform_animator(&self.transform) {
            c.insert(animator);
        }
        let id = c.id();

        c.insert(LayerAnimationInfo {
            start_frame: self.start_frame,
            end_frame: self.end_frame,
        });
        id
    }

    fn spawn_shape(
        &self,
        frame: f32,
        zindex: f32,
        shape: StyledShape,
        commands: &mut Commands,
    ) -> Option<Entity> {
        if shape.shape.hidden {
            return None;
        }
        let mut draw_mode = utils::shape_draw_mode(&shape);
        let global_opacity = self.opacity.initial_value();
        if global_opacity < 1.0 {
            match &mut draw_mode {
                DrawMode::Fill(f) => f.color.set_a(f.color.a() * global_opacity),
                DrawMode::Stroke(s) => s.color.set_a(s.color.a() * global_opacity),
                DrawMode::Outlined {
                    fill_mode,
                    outline_mode,
                } => {
                    fill_mode.color.set_a(fill_mode.color.a() * global_opacity);
                    outline_mode
                        .color
                        .set_a(outline_mode.color.a() * global_opacity)
                }
            };
        }
        let mut transform = Transform::from_matrix(shape.transform.value(0.0));
        transform.translation.z = -1.0 * zindex;

        let entity = match &shape.shape.shape {
            Shape::Ellipse(ellipse) => {
                let Ellipse { size, position, .. } = ellipse;
                let initial_size = size.initial_value() / 2.0;
                let initial_pos = position.initial_value();
                let ellipse_shape = shapes::Ellipse {
                    radii: Vec2::new(initial_size.x, initial_size.y),
                    center: Vec2::new(initial_pos.x, initial_pos.y),
                };

                let mut c = commands.spawn();
                c.insert_bundle(GeometryBuilder::build_as(
                    &ellipse_shape,
                    draw_mode,
                    transform,
                ));
                if let Some(mut animator) = self.transform_animator(&shape.transform) {
                    self.sync_animator(&mut animator, frame);
                    c.insert(animator);
                }
                if let Some(mut animator) = self.draw_mode_animator(&shape) {
                    self.sync_animator(&mut animator, frame);
                    c.insert(animator);
                }
                c.insert(LottieShapeComp(shape));
                c.id()
            }
            Shape::PolyStar(star) => {
                let mut builder = Builder::new();
                star.to_path(frame, &mut builder);
                let path_shape = Path(builder.build());
                let mut c = commands.spawn();
                c.insert_bundle(GeometryBuilder::build_as(&path_shape, draw_mode, transform));
                if let Some(mut animator) = self.transform_animator(&shape.transform) {
                    self.sync_animator(&mut animator, frame);
                    c.insert(animator);
                }
                if let Some(mut animator) = self.draw_mode_animator(&shape) {
                    self.sync_animator(&mut animator, frame);
                    c.insert(animator);
                }
                c.id()
            }
            Shape::Rectangle(rect) => {
                let mut builder = Builder::new();
                rect.to_path(frame, &mut builder);
                let path_shape = Path(builder.build());
                let mut c = commands.spawn();
                c.insert_bundle(GeometryBuilder::build_as(&path_shape, draw_mode, transform));
                if let Some(mut animator) = self.transform_animator(&shape.transform) {
                    self.sync_animator(&mut animator, frame);
                    c.insert(animator);
                }
                if let Some(mut animator) = self.draw_mode_animator(&shape) {
                    self.sync_animator(&mut animator, frame);
                    c.insert(animator);
                }
                c.id()
            }
            Shape::Path { d } => {
                let mut beziers = d.initial_value();
                // Since we already globally changed the axis system, here bevy_lyon_prototype's
                // y-axis logic is redundant. So we inverse it again to make the
                // result correct
                beziers.inverse_y_orientation();

                let mut builder = Builder::new();
                beziers.to_path(frame, &mut builder);
                let path_shape = Path(builder.build());
                let mut c = commands.spawn();
                c.insert_bundle(TransformBundle::default());
                c.insert_bundle(GeometryBuilder::build_as(&path_shape, draw_mode, transform));
                if let Some(mut animator) = self.transform_animator(&shape.transform) {
                    self.sync_animator(&mut animator, frame);
                    c.insert(animator);
                }
                if let Some(mut animator) = self.draw_mode_animator(&shape) {
                    self.sync_animator(&mut animator, frame);
                    c.insert(animator);
                }

                // Add bezier tween
                if d.animated {
                    let tween = d
                        .keyframes
                        .tween(self.frame_rate, |start, end| PathLens { start, end });
                    let mut animator = Animator::new(tween);
                    let progress = (frame - self.start_frame) as f32
                        / (self.end_frame - self.start_frame) as f32;
                    animator.set_progress(progress);
                    c.insert(animator);
                }
                c.id()
            }
            Shape::Group { .. } => {
                unreachable!()
            }
            _ => {
                println!("{:?}", shape.shape.shape);
                todo!()
            }
        };
        Some(entity)
    }

    fn transform_animator(&self, transform: &LottieTransform) -> Option<Animator<Transform>> {
        let mut tweens = vec![];
        let frame_rate = self.frame_rate;
        if transform.is_animated() {
            tweens.push(transform.tween(frame_rate, |data, _| TransformLens { data, frames: 0.0 }));
        }
        if !tweens.is_empty() {
            let tracks = Tracks::new(tweens);
            Some(Animator::new(tracks))
        } else {
            None
        }
    }

    fn draw_mode_animator(&self, shape: &StyledShape) -> Option<Animator<DrawMode>> {
        let mut tweens = vec![];
        let frame_rate = self.frame_rate;
        if let Some(stroke) = shape.stroke.as_ref() {
            if stroke.width.is_animated() {
                tweens.push(
                    stroke
                        .width
                        .keyframes
                        .tween(frame_rate, |start, end| StrokeWidthLens { start, end }),
                );
            }
        }

        if self.opacity.is_animated() {
            let opacity_lens = OpacityLens {
                opacity: self.opacity.clone(),
                frames: self.end_frame - self.start_frame,
                fill_opacity: shape.fill.opacity.clone(),
                stroke_opacity: shape.stroke.as_ref().map(|s| s.opacity.clone()),
            };
            let secs = opacity_lens.frames as f32 / self.frame_rate as f32;
            let tween = Tween::new(
                EaseMethod::Linear,
                TweeningType::Once,
                Duration::from_secs_f32(secs),
                opacity_lens,
            );
            tweens.push(Sequence::from_single(tween));
        }

        if !tweens.is_empty() {
            let tracks = Tracks::new(tweens);
            Some(Animator::new(tracks))
        } else {
            None
        }
    }

    fn sync_animator<T: Component>(&self, animator: &mut Animator<T>, frame: f32) {
        let progress = (frame - self.start_frame) / (self.end_frame - self.start_frame);
        animator.set_progress(progress);
    }
}
