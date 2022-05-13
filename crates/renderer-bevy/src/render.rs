use bevy::ecs::system::EntityCommands;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Entity, Transform};
use bevy_prototype_lyon::prelude::tess::path::path::Builder;
use bevy_prototype_lyon::prelude::*;
use bevy_tweening::lens::{TransformPositionLens, TransformScaleLens};
use bevy_tweening::{Animator, Tracks};
use lottie_core::*;

use lottie_core::prelude::*;
use lottie_core::Transform as LottieTransform;

use crate::lens::{PathLens, StrokeWidthLens};
use crate::tween::TweenProducer;
use crate::*;

pub trait LayerRenderer {
    fn spawn(&self, frame: u32, commands: &mut Commands) -> Entity;
    fn spawn_transform(
        &self,
        frame: u32,
        transform: &LottieTransform,
        commands: &mut EntityCommands,
    );
    fn spawn_stroke(&self, frame: u32, stroke: &Stroke, commands: &mut EntityCommands);
    fn spawn_shape(
        &self,
        frame: u32,
        shape: StyledShape,
        commands: &mut Commands,
    ) -> Option<Entity>;
}

impl LayerRenderer for StagedLayer {
    fn spawn(&self, frame: u32, commands: &mut Commands) -> Entity {
        let mut c = commands.spawn();
        let (transform, anchor) = utils::initial_transform_and_anchor(&self.transform);

        log::trace!(
            "spawn layer {:?}: start {}, end {}, transform: {:?}, anchor: {:?}",
            c.id(),
            self.start_frame,
            self.end_frame,
            transform,
            anchor
        );
        match &self.content {
            RenderableContent::Shape(shapes) => {
                for shape in shapes.shapes() {
                    if let Some(entity) = self.spawn_shape(frame, shape, c.commands()) {
                        log::trace!("layer {:?} get a child {:?}", c.id(), entity);
                        c.add_child(entity);
                    }
                }
            }
            RenderableContent::Group => {}
            _ => todo!(),
        }
        let local = Mat4::from_translation(anchor)
            * Mat4::from_scale(transform.scale)
            * Mat4::from_rotation_z(transform.rotation.to_axis_angle().1)
            * Mat4::from_translation(-anchor);
        let local = Transform::from_matrix(local);
        c.insert_bundle(TransformBundle {
            local,
            global: Default::default(),
        });
        let id = c.id();

        c.insert(LayerAnimationInfo {
            start_frame: self.start_frame,
            end_frame: self.end_frame,
        });
        id
    }

    fn spawn_transform(
        &self,
        frame: u32,
        transform: &LottieTransform,
        commands: &mut EntityCommands,
    ) {
        let mut tweens = vec![];
        let frame_rate = self.frame_rate;
        if transform.position.is_animated() {
            tweens.push(
                transform
                    .position
                    .keyframes
                    .tween(frame_rate, |start, end| TransformPositionLens {
                        start: Vec3::new(start.x, start.y, 0.0),
                        end: Vec3::new(end.x, end.y, 0.0),
                    }),
            );
        }
        if transform.scale.is_animated() {
            tweens.push(transform.scale.keyframes.tween(frame_rate, |start, end| {
                TransformScaleLens {
                    start: Vec3::new(start.x, start.y, 0.0) / 100.0,
                    end: Vec3::new(end.x, end.y, 0.0) / 100.0,
                }
            }));
        }
        if !tweens.is_empty() {
            let tracks = Tracks::new(tweens);
            let mut animator = Animator::new(tracks);
            let progress =
                (frame - self.start_frame) as f32 / (self.end_frame - self.start_frame) as f32;
            animator.set_progress(progress);
            commands.insert(animator);
        }
    }

    fn spawn_shape(
        &self,
        frame: u32,
        shape: StyledShape,
        commands: &mut Commands,
    ) -> Option<Entity> {
        if shape.shape.hidden {
            return None;
        }
        let draw_mode = utils::shape_draw_mode(&shape);
        // TODO: handle anchor
        let (initial_transform, _) = utils::initial_transform_and_anchor(&shape.transform);
        // let bbox = shape.shape.shape.bbox(0);
        // let center = bbox.center();
        // initial_transform.translation += Vec3::new(center.x, center.y, 0.0);

        let entity = match &shape.shape.shape {
            Shape::Ellipse(ellipse) => {
                let Ellipse { size, position, .. } = ellipse;
                let initial_size = size.initial_value() / 2.0;
                let initial_pos = position.initial_value();
                let transform = Transform::from_matrix(
                    initial_transform.compute_matrix()
                        * Mat4::from_translation(Vec3::new(initial_pos.x, initial_pos.y, 0.0)),
                );
                let ellipse_shape = shapes::Ellipse {
                    radii: Vec2::new(initial_size.x, initial_size.y),
                    center: Vec2::new(0.0, 0.0),
                };

                let mut c = commands.spawn();
                c.insert_bundle(GeometryBuilder::build_as(
                    &ellipse_shape,
                    draw_mode,
                    transform,
                ));
                self.spawn_transform(frame, &shape.transform, &mut c);
                if let Some(stroke) = shape.stroke.as_ref() {
                    self.spawn_stroke(frame, stroke, &mut c);
                }
                c.insert(LottieShapeComp(shape));
                c.id()
            }
            Shape::PolyStar(star) => {
                let mut builder = Builder::new();
                star.to_path(frame, &mut builder);
                let path_shape = Path(builder.build());
                let mut c = commands.spawn();
                c.insert_bundle(GeometryBuilder::build_as(
                    &path_shape,
                    draw_mode,
                    initial_transform,
                ));
                self.spawn_transform(frame, &shape.transform, &mut c);
                if let Some(stroke) = shape.stroke.as_ref() {
                    self.spawn_stroke(frame, stroke, &mut c);
                }
                c.id()
            }
            Shape::Rectangle(rect) => {
                let mut builder = Builder::new();
                rect.to_path(frame, &mut builder);
                let path_shape = Path(builder.build());
                let mut c = commands.spawn();
                c.insert_bundle(GeometryBuilder::build_as(
                    &path_shape,
                    draw_mode,
                    initial_transform,
                ));
                self.spawn_transform(frame, &shape.transform, &mut c);
                if let Some(stroke) = shape.stroke.as_ref() {
                    self.spawn_stroke(frame, stroke, &mut c);
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
                c.insert_bundle(GeometryBuilder::build_as(
                    &path_shape,
                    draw_mode,
                    initial_transform,
                ));
                self.spawn_transform(frame, &shape.transform, &mut c);
                if let Some(stroke) = shape.stroke.as_ref() {
                    self.spawn_stroke(frame, stroke, &mut c);
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

    fn spawn_stroke(&self, frame: u32, stroke: &Stroke, commands: &mut EntityCommands) {
        let mut tweens = vec![];
        let frame_rate = self.frame_rate;
        if stroke.width.is_animated() {
            tweens.push(
                stroke
                    .width
                    .keyframes
                    .tween(frame_rate, |start, end| StrokeWidthLens { start, end }),
            );
        }
        if !tweens.is_empty() {
            let tracks = Tracks::new(tweens);
            let mut animator = Animator::new(tracks);
            let progress =
                (frame - self.start_frame) as f32 / (self.end_frame - self.start_frame) as f32;
            animator.set_progress(progress);
            commands.insert(animator);
        }
    }
}
