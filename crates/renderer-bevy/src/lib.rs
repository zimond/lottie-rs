mod render;

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use bevy::winit::WinitPlugin;
use bevy_diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_prototype_lyon::prelude::*;
use bevy_tweening::{
    Animator, AnimatorState, Delay, EaseMethod, Lens, Sequence, Tween, TweeningPlugin, TweeningType,
};
use flo_curves::bezier::{curve_intersects_line, Curve};
use flo_curves::{BezierCurveFactory, Coord2};
use lottie_core::prelude::StyledShape;
use lottie_core::*;
use render::*;
use std::sync::Arc;
use std::time::Duration;

use bevy::prelude::Transform;

#[derive(Component, Deref, DerefMut)]
pub struct LottieComp(Lottie);

#[derive(Component)]
struct LottieShapeComp(StyledShape);

#[derive(Component)]
struct LayerAnimationInfo {
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

impl<T> TweenProducer<Transform, T> for Vec<KeyFrame<Vector2D>>
where
    T: Lens<Transform> + Send + Sync + 'static,
{
    type Key = Vector2D;
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
    let mut lottie = lottie.clone();
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

    lottie.scale = scale;
    let comp = LottieComp(lottie);
    for layer in comp.flatten_layers() {
        layer.spawn(&mut commands);
    }
    commands
        .spawn()
        .insert(comp)
        .insert_bundle(TransformBundle::default());
}

fn lottie_animate_system(
    query: Query<(&LayerAnimationInfo, &Children)>,
    mut animated_shapes: Query<(&mut Visibility, &mut Animator<Transform>)>,
    mut info: ResMut<LottieAnimationInfo>,
    time: Res<Time>,
) {
    let t = time.delta_seconds() * (info.frame_rate as f32);
    info.current_frame = info.current_frame + t.round() as u32;

    for (i, children) in query.iter() {
        for child in children.iter() {
            if let Ok((mut visibility, mut animator)) = animated_shapes.get_mut(*child) {
                if info.current_frame < i.start_frame || info.current_frame >= i.end_frame {
                    visibility.is_visible = false;
                    animator.stop();
                } else {
                    visibility.is_visible = true;
                    if animator.state == AnimatorState::Paused {
                        animator.state = AnimatorState::Playing;
                        animator.set_progress(
                            info.current_frame as f32 / (i.end_frame - i.start_frame) as f32,
                        );
                    }
                }
            }
        }
    }
    info.current_frame %= info.end_frame;
}
