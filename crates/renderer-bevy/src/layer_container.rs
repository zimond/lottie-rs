use std::collections::HashMap;

use bevy::{
    ecs::system::EntityCommands,
    math::{Vec2, Vec3},
    prelude::{Color, Entity},
};
use bevy_prototype_lyon::prelude::*;
use bevy_tweening::{
    lens::{TransformPositionLens, TransformScaleLens},
    Animator, AnimatorState, Tracks,
};
use dashmap::DashMap;
use lottie_core::*;

use bevy::prelude::Transform;

use crate::*;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub struct LayerKey(u32);

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ShapeKey(u32);

#[derive(Component)]
pub struct LottieComp {
    data: Lottie,
    scale: f32,
    current_frame: u32,
    entities: DashMap<LayerKey, DashMap<ShapeKey, Entity>>,
}

impl LottieComp {
    pub fn new(lottie: Lottie, scale: f32) -> Self {
        LottieComp {
            data: lottie,
            scale,
            current_frame: 0,
            entities: DashMap::new(),
        }
    }
}

pub trait LayerContainer {
    fn contains(&self, layer: &LayerKey, shape: &ShapeKey) -> bool;
    fn insert(&self, layer: LayerKey, shape: ShapeKey, entity: Entity);
    fn layers(&self) -> std::slice::Iter<Layer>;
    fn remove(&self, layer: &LayerKey) -> Option<DashMap<ShapeKey, Entity>>;
    fn current_frame(&self) -> u32;
    fn frame_rate(&self) -> u32;
    fn query_container_by_id(&self, id: &str) -> Option<PrecompositionContainer>;
    fn spawn_layers(&self, commands: &mut EntityCommands) {
        let current = self.current_frame();
        for layer in self.layers() {
            let layer_key = LayerKey(layer.id);
            if current >= layer.end_frame || current < layer.start_frame {
                if let Some(shapes) = self.remove(&layer_key) {
                    for (_, entity) in shapes {
                        commands.commands().entity(entity).despawn();
                    }
                    continue;
                }
            } else if current == layer.start_frame {
                match &layer.content {
                    LayerContent::Shape(shapes) => {
                        for shape in shapes.shapes() {
                            let key = LayerKey(layer.id);
                            let shape_key = ShapeKey(shape.shape.id);
                            if let Some(entity) = self.spawn_shape(
                                key,
                                layer.start_frame,
                                layer.end_frame,
                                shape,
                                commands,
                            ) {
                                self.insert(layer_key, shape_key, entity);
                            }
                        }
                    }
                    LayerContent::Precomposition(pre) => {
                        if let Some(asset) = self.query_container_by_id(&pre.ref_id) {
                            asset.spawn_layers(commands);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn spawn_shape(
        &self,
        layer_key: LayerKey,
        start_frame: u32,
        end_frame: u32,
        shape: StyledShape,
        commands: &mut EntityCommands,
    ) -> Option<Entity> {
        let shape_key = ShapeKey(shape.shape.id);
        if shape.shape.hidden || self.contains(&layer_key, &shape_key) {
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
                let fill = shape.fill.color.initial_color();
                let fill_opacity = (shape.fill.opacity.initial_value() * 255.0) as u8;
                let c = commands.insert_bundle(GeometryBuilder::build_as(
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
                c.insert(LottieLayerAnimationInfo {
                    start_frame,
                    end_frame,
                });
                c.id()
            }
            Shape::Group { .. } => {
                unreachable!()
            }
            _ => {
                todo!()
            }
        };
        Some(entity)
    }
}

impl LayerContainer for LottieComp {
    fn contains(&self, layer: &LayerKey, shape: &ShapeKey) -> bool {
        self.entities
            .get(layer)
            .map(|shapes| shapes.contains_key(shape))
            .unwrap_or_default()
    }

    fn insert(&self, layer: LayerKey, shape: ShapeKey, entity: Entity) {
        self.entities
            .entry(layer)
            .or_default()
            .insert(shape, entity);
    }

    fn layers(&self) -> std::slice::Iter<Layer> {
        self.data.model.layers.iter()
    }

    fn remove(&self, layer: &LayerKey) -> Option<DashMap<ShapeKey, Entity>> {
        Some(self.entities.remove(layer)?.1)
    }

    fn current_frame(&self) -> u32 {
        self.current_frame
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
    fn contains(&self, layer: &LayerKey, shape: &ShapeKey) -> bool {
        self.comp.contains(layer, shape)
    }

    fn insert(&self, layer: LayerKey, shape: ShapeKey, entity: Entity) {
        self.comp.insert(layer, shape, entity)
    }

    fn layers(&self) -> std::slice::Iter<Layer> {
        self.asset.layers.iter()
    }

    fn remove(&self, layer: &LayerKey) -> Option<DashMap<ShapeKey, Entity>> {
        self.comp.remove(layer)
    }

    fn current_frame(&self) -> u32 {
        self.comp.current_frame
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
