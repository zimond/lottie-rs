use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Color, Entity};
use bevy_prototype_lyon::prelude::*;
use bevy_tweening::lens::{TransformPositionLens, TransformScaleLens};
use bevy_tweening::{Animator, Tracks};
use lottie_core::*;

use bevy::prelude::Transform;
use lottie_core::prelude::*;

use crate::shape::StyledShapeExt;
use crate::*;

pub trait LayerRenderer {
    fn spawn(&self, frame: u32, commands: &mut Commands) -> Entity;
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
        match &self.content {
            RenderableContent::Shape(shapes) => {
                for shape in shapes.shapes() {
                    if let Some(entity) = self.spawn_shape(frame, shape, c.commands()) {
                        c.add_child(entity);
                    }
                }
            }
            RenderableContent::Group => {}
            _ => todo!(),
        }

        c.insert_bundle(TransformBundle::default());
        c.insert(LayerAnimationInfo {
            start_frame: self.start_frame,
            end_frame: self.end_frame,
        });
        c.insert(Visibility { is_visible: false });
        c.id()
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
        let frame_rate = self.frame_rate;
        let draw_mode = shape.draw_mode();
        let initial_transform = shape.initial_transform();
        // let bbox = shape.shape.shape.bbox(0);
        // let center = bbox.center();
        // initial_transform.translation += Vec3::new(center.x, center.y, 0.0);

        let entity =
            match &shape.shape.shape {
                Shape::Ellipse(ellipse) => {
                    let Ellipse { size, .. } = ellipse;
                    let initial_size = size.initial_value() / 2.0;
                    let ellipse_shape = shapes::Ellipse {
                        radii: Vec2::new(initial_size.x, initial_size.y),
                        center: Vec2::new(0.0, 0.0),
                    };

                    let mut c = commands.spawn();
                    c.insert_bundle(GeometryBuilder::build_as(
                        &ellipse_shape,
                        draw_mode,
                        initial_transform,
                    ));
                    let mut tweens = vec![];
                    if shape.transform.position.is_animated() {
                        tweens.push(shape.transform.position.keyframes.tween(
                            frame_rate,
                            |start, end| TransformPositionLens {
                                start: Vec3::new(start.x, start.y, 0.0),
                                end: Vec3::new(end.x, end.y, 0.0),
                            },
                        ));
                    }
                    if shape.transform.scale.is_animated() {
                        tweens.push(shape.transform.scale.keyframes.tween(
                            frame_rate,
                            |start, end| TransformScaleLens {
                                start: Vec3::new(start.x, start.y, 0.0) / 100.0,
                                end: Vec3::new(end.x, end.y, 0.0) / 100.0,
                            },
                        ));
                    }
                    if !tweens.is_empty() {
                        let tracks = Tracks::new(tweens);
                        let mut animator = Animator::new(tracks);
                        let progress = (frame - self.start_frame) as f32
                            / (self.end_frame - self.start_frame) as f32;
                        animator.set_progress(progress);
                        c.insert(animator);
                    }
                    c.insert(LottieShapeComp(shape));
                    c.id()
                }
                Shape::Path { d } => {
                    let mut beziers = d.initial_value();
                    let mut bbox = beziers.bbox(d.keyframes[0].start_frame.unwrap());
                    let stroke_width: f32 = shape
                        .strokes
                        .iter()
                        .map(|stroke| stroke.width.initial_value())
                        .sum();
                    beziers.move_origin(-bbox.min_x() + stroke_width, -bbox.min_y() + stroke_width);
                    bbox.origin.x -= stroke_width;
                    bbox.origin.y -= stroke_width;
                    bbox.size.width += 2.0 * stroke_width;
                    bbox.size.height += 2.0 * stroke_width;
                    let path_shape = shapes::SvgPathShape {
                        svg_doc_size_in_px: Vec2::new(bbox.width(), bbox.height()),
                        svg_path_string: beziers.to_svg_d(),
                    };
                    let fill = shape.fill.color.initial_value();
                    let fill_opacity = (shape.fill.opacity.initial_value() * 255.0) as u8;
                    let mut c = commands.spawn();
                    c.insert_bundle(GeometryBuilder::build_as(
                        &path_shape,
                        DrawMode::Outlined {
                            fill_mode: FillMode::color(Color::rgba_u8(
                                fill.r,
                                fill.g,
                                fill.b,
                                fill_opacity,
                            )),
                            outline_mode: StrokeMode::new(Color::BLACK, 10.0),
                        },
                        Transform::default(), /* from_translation(Vec3::new(initial_pos.x,
                                               * initial_pos.y, 0.0)), */
                    ));
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
}
