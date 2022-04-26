use bevy::{
    math::{Vec2, Vec3},
    prelude::{Color, Entity},
};
use bevy_prototype_lyon::prelude::*;
use bevy_tweening::{
    lens::{TransformPositionLens, TransformScaleLens},
    Animator, AnimatorState, Tracks,
};
use lottie_core::*;

use bevy::prelude::Transform;

use crate::*;

#[derive(Component)]
pub struct LottieComp {
    data: Lottie,
    scale: f32,
}

impl LottieComp {
    pub fn new(lottie: Lottie, scale: f32) -> Self {
        LottieComp {
            data: lottie,
            scale,
        }
    }
}

pub trait LayerContainer {
    fn layers(&self) -> std::slice::Iter<Layer>;
    fn frame_rate(&self) -> u32;
    fn query_container_by_id(&self, id: &str) -> Option<PrecompositionContainer>;
    fn spawn_layers(&self, commands: &mut Commands) {
        for layer in self.layers() {
            let mut c = commands.spawn();
            match &layer.content {
                LayerContent::Shape(shapes) => {
                    for shape in shapes.shapes() {
                        if let Some(entity) = self.spawn_shape(
                            layer.start_frame,
                            layer.end_frame,
                            shape,
                            c.commands(),
                        ) {
                            c.add_child(entity);
                        }
                    }
                }
                LayerContent::Precomposition(pre) => {
                    if let Some(asset) = self.query_container_by_id(&pre.ref_id) {
                        asset.spawn_layers(c.commands());
                    }
                }
                _ => {}
            }
            c.insert_bundle(TransformBundle::default());
            c.insert(LottieLayerAnimationInfo {
                start_frame: layer.start_frame,
                end_frame: layer.end_frame,
            });
            c.insert(Visibility { is_visible: false });
        }
    }

    fn spawn_shape(
        &self,
        start_frame: u32,
        end_frame: u32,
        shape: StyledShape,
        commands: &mut Commands,
    ) -> Option<Entity> {
        if shape.shape.hidden {
            return None;
        }
        let frame_rate = self.frame_rate();
        let entity = match &shape.shape.shape {
            Shape::Ellipse(ellipse) => {
                let Ellipse { position, size } = ellipse;
                let initial_size = size.initial_value() / 2.0;
                let initial_pos = position.initial_value();
                let ellipse_shape = shapes::Ellipse {
                    radii: Vec2::new(initial_size.x, initial_size.y),
                    center: Vec2::new(0.0, 0.0),
                };
                let fill = shape.fill.color.initial_value();
                let fill_opacity = (shape.fill.opacity.initial_value() * 255.0) as u8;
                let mut c = commands.spawn();
                c.insert_bundle(GeometryBuilder::build_as(
                    &ellipse_shape,
                    DrawMode::Outlined {
                        fill_mode: FillMode::color(Color::rgba_u8(
                            fill.r,
                            fill.g,
                            fill.b,
                            fill_opacity,
                        )),
                        outline_mode: StrokeMode::new(Color::BLACK, 0.0),
                    },
                    Transform::from_translation(Vec3::new(initial_pos.x, initial_pos.y, 0.0)),
                ));
                let mut tweens = vec![];
                if shape.transform.position.is_animated() {
                    tweens.push(shape.transform.position.keyframes.tween(
                        start_frame,
                        end_frame,
                        frame_rate,
                        |start, end| TransformPositionLens {
                            start: Vec3::new(start.x, start.y, 0.0),
                            end: Vec3::new(end.x, end.y, 0.0),
                        },
                    ));
                }
                if shape.transform.scale.is_animated() {
                    tweens.push(shape.transform.scale.keyframes.tween(
                        start_frame,
                        end_frame,
                        frame_rate,
                        |start, end| TransformScaleLens {
                            start: Vec3::new(start.x, start.y, 0.0) / 100.0,
                            end: Vec3::new(end.x, end.y, 0.0) / 100.0,
                        },
                    ));
                }
                if !tweens.is_empty() {
                    let tracks = Tracks::new(tweens);
                    let animator = Animator::new(tracks).with_state(AnimatorState::Paused);
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
                    Transform::default(), //from_translation(Vec3::new(initial_pos.x, initial_pos.y, 0.0)),
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

impl LayerContainer for LottieComp {
    fn layers(&self) -> std::slice::Iter<Layer> {
        self.data.model.layers.iter()
    }

    fn query_container_by_id(&self, id: &str) -> Option<PrecompositionContainer> {
        let asset = self.data.model.assets.iter().find(|asset| asset.id == id)?;
        Some(PrecompositionContainer { asset, comp: &self })
    }

    fn frame_rate(&self) -> u32 {
        self.data.model.frame_rate
    }
}

pub struct PrecompositionContainer<'a> {
    asset: &'a Precomposition,
    comp: &'a LottieComp,
}

impl<'a> LayerContainer for PrecompositionContainer<'a> {
    fn layers(&self) -> std::slice::Iter<Layer> {
        self.asset.layers.iter()
    }

    fn query_container_by_id(&self, id: &str) -> Option<PrecompositionContainer> {
        self.comp.query_container_by_id(id)
    }

    fn frame_rate(&self) -> u32 {
        self.asset
            .frame_rate
            .unwrap_or_else(|| self.comp.frame_rate())
    }
}
