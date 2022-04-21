use bevy::app::PluginGroupBuilder;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::window::WindowPlugin;
use bevy::winit::WinitPlugin;
use bevy_diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_prototype_lyon::prelude::*;
use bevy_tweening::lens::{TransformPositionLens, TransformScaleLens};
use bevy_tweening::{
    Animator, AnimatorState, Delay, EaseMethod, Lens, Sequence, Tracks, Tween, Tweenable,
    TweeningPlugin, TweeningType,
};
use flo_curves::bezier::{curve_intersects_line, Curve};
use flo_curves::{BezierCurveFactory, Coord2};
use lottie_core::{Renderer, *};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::time::Duration;

use bevy::prelude::Transform;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct LayerKey(u32);

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct ShapeKey(u32);

#[derive(Component)]
struct LottieComp {
    data: Lottie,
    scale: f32,
    current_frame: u32,
    entities: HashMap<LayerKey, HashMap<ShapeKey, Entity>>,
}

impl LottieComp {
    fn spawn_layers(&mut self, entity: Entity, commands: &mut Commands) {
        let current = self.current_frame;
        for layer in &self.data.model.layers {
            if current >= layer.end_frame || current < layer.start_frame {
                if let Some(shapes) = self.entities.remove(&LayerKey(layer.id)) {
                    for (_, entity) in shapes {
                        commands.entity(entity).despawn();
                    }
                    continue;
                }
            } else if current == layer.start_frame {
                let mut entity_commands = commands.entity(entity);
                match &layer.content {
                    LayerContent::Shape(shapes) => {
                        for shape in shapes.shapes() {
                            let shape_id = ShapeKey(shape.shape.id);
                            if shape.shape.hidden
                                || self
                                    .entities
                                    .entry(LayerKey(layer.id))
                                    .or_default()
                                    .contains_key(&shape_id)
                            {
                                continue;
                            }
                            let id = self.spawn_shape(
                                layer.start_frame,
                                layer.end_frame,
                                shape,
                                &mut entity_commands,
                            );
                            self.entities
                                .entry(LayerKey(layer.id))
                                .or_default()
                                .insert(shape_id, id);
                        }
                    }
                    LayerContent::Precomposition(pre) => {}
                    _ => {}
                }
            }
        }

        self.current_frame = (current + 1) % self.data.model.end_frame;
    }

    fn spawn_shape(
        &self,
        start_frame: u32,
        end_frame: u32,
        shape: StyledShape,
        commands: &mut EntityCommands,
    ) -> Entity {
        let frame_rate = self.data.model.frame_rate;
        match &shape.shape.shape {
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
        }
    }
}

#[derive(Bundle)]
struct LottieBundle {
    transform: Transform,
    global_transform: GlobalTransform,
    comp: LottieComp,
}

#[derive(Component)]
struct LottieShapeComp(StyledShape);

#[derive(Component)]
struct LottieLayerAnimationInfo {
    start_frame: u32,
    end_frame: u32,
}

struct LottieAnimationInfo {
    start_frame: u32,
    end_frame: u32,
    frame_rate: u32,
    current_frame: u32,
}

trait TweenProducer<T, L>
where
    L: Lens<T> + Send + Sync + 'static,
{
    type Key;
    fn tween(
        &self,
        start_frame: u32,
        end_frame: u32,
        frame_rate: u32,
        producer: fn(start: Self::Key, end: Self::Key) -> L,
    ) -> Sequence<Transform>;
}

impl<T> TweenProducer<Transform, T> for Vec<KeyFrame<Vector2D<f32>>>
where
    T: Lens<Transform> + Send + Sync + 'static,
{
    type Key = Vector2D<f32>;
    fn tween(
        &self,
        start_frame: u32,
        end_frame: u32,
        frame_rate: u32,
        producer: fn(start: Self::Key, end: Self::Key) -> T,
    ) -> Sequence<Transform> {
        let mut tween: Option<Sequence<Transform>> = None;
        for p in self.windows(2) {
            let p0 = &p[0];
            let p1 = &p[1];
            let start = p0.value;
            let end = p1.value;
            let ease_out = p0.easing_out.clone().unwrap();
            let ease_in = p0.easing_in.clone().unwrap();
            let frames = p1.start_frame.unwrap() - p0.start_frame.unwrap();
            let secs = frames as f32 / frame_rate as f32;
            let curve = Curve::from_points(
                Coord2(0.0, 0.0),
                (
                    Coord2(ease_out.x[0] as f64, ease_out.y[0] as f64),
                    Coord2(ease_in.x[0] as f64, ease_in.y[0] as f64),
                ),
                Coord2(1.0, 1.0),
            );
            let t = Tween::new(
                EaseMethod::CustomFunction(Arc::new(move |x| {
                    let intersection = curve_intersects_line(
                        &curve,
                        &(Coord2(x as f64, 0.0), Coord2(x as f64, 1.0)),
                    );
                    if intersection.is_empty() {
                        x
                    } else {
                        intersection[0].2 .1 as f32
                    }
                })),
                TweeningType::Once,
                Duration::from_secs_f32(secs),
                producer(start, end),
            );
            let t = if self[0].start_frame.unwrap() > start_frame && tween.is_none() {
                Delay::new(Duration::from_secs_f32(
                    (self[0].start_frame.unwrap() - start_frame) as f32 / (frame_rate as f32),
                ))
                .then(t)
            } else {
                Sequence::from_single(t)
            };
            tween = Some(match tween {
                Some(seq) => seq.then(t),
                None => Sequence::from_single(t),
            });
        }
        let mut seq = tween.unwrap();
        if self[self.len() - 1].start_frame.unwrap() < end_frame {
            seq = seq.then(Delay::new(Duration::from_secs_f32(
                (end_frame - self[self.len() - 1].start_frame.unwrap()) as f32
                    / (frame_rate as f32),
            )));
        }
        seq
    }
}

pub struct BevyRenderer {
    app: App,
}

impl BevyRenderer {
    pub fn new() -> Self {
        let mut app = App::new();
        let mut plugin_group_builder = PluginGroupBuilder::default();
        DefaultPlugins.build(&mut plugin_group_builder);
        // Defaulty disable GUI window
        plugin_group_builder.disable::<WinitPlugin>();
        // Disable gamepad support
        plugin_group_builder.disable::<GilrsPlugin>();
        plugin_group_builder.finish(&mut app);
        app.insert_resource(Msaa { samples: 4 })
            .add_plugin(TweeningPlugin)
            // .add_plugin(FrameTimeDiagnosticsPlugin)
            // .add_plugin(LogDiagnosticsPlugin::default())
            .add_plugin(ShapePlugin)
            .add_system(lottie_spawn_system)
            .add_system(lottie_animate_system);
        BevyRenderer { app }
    }

    pub fn add_plugin(&mut self, plugin: impl Plugin) {
        self.app.add_plugin(plugin);
    }
}

impl Renderer for BevyRenderer {
    fn load_lottie(&mut self, lottie: Lottie) {
        self.app
            .insert_resource(lottie)
            .add_startup_system(setup_system);
    }

    fn render(&mut self) {
        self.app.run()
    }
}

fn setup_system(mut commands: Commands, mut windows: ResMut<Windows>, lottie: Res<Lottie>) {
    let window = windows.get_primary_mut().unwrap();
    let scale = window.scale_factor() as f32;
    let lottie = lottie.clone();
    commands.remove_resource::<Lottie>();
    window.set_title(
        lottie
            .model
            .name
            .clone()
            .unwrap_or_else(|| String::from("Lottie Animation")),
    );
    window.set_resolution(lottie.model.width as f32, lottie.model.height as f32);
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform =
        Transform::from_scale(Vec3::new(1.0, -1.0, 1.0)).with_translation(Vec3::new(
            lottie.model.width as f32 / 2.0,
            lottie.model.height as f32 / 2.0,
            0.0,
        ));
    commands.insert_resource(LottieAnimationInfo {
        start_frame: lottie.model.start_frame,
        end_frame: lottie.model.end_frame,
        frame_rate: lottie.model.frame_rate,
        current_frame: 0,
    });
    commands.spawn_bundle(camera);
    commands.spawn_bundle(LottieBundle {
        comp: LottieComp {
            data: lottie,
            current_frame: 0,
            scale,
            entities: HashMap::new(),
        },
        global_transform: GlobalTransform::default(),
        transform: Transform::default(),
    });
}

fn lottie_spawn_system(mut query: Query<(Entity, &mut LottieComp)>, mut commands: Commands) {
    for (entity, mut comp) in query.iter_mut() {
        comp.spawn_layers(entity, &mut commands);
    }
}

fn lottie_animate_system(
    mut query: Query<(&mut Animator<Transform>, &LottieLayerAnimationInfo)>,
    mut info: ResMut<LottieAnimationInfo>,
    time: Res<Time>,
) {
    let t = time.delta_seconds() * (info.frame_rate as f32);
    info.current_frame = info.current_frame + t.round() as u32;
    for (mut animator, layer_info) in query.iter_mut() {
        if info.current_frame >= layer_info.start_frame && animator.state == AnimatorState::Paused {
            animator.state = AnimatorState::Playing;
            animator.set_progress(0.0);
        } else if info.current_frame >= layer_info.end_frame {
            animator.stop();
        }
    }
    info.current_frame %= info.end_frame;
}
