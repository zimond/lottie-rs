mod bezier;
mod render;
mod tween;
mod utils;

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::utils::HashMap;
use bevy::winit::WinitPlugin;
// use bevy_diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_prototype_lyon::prelude::*;
use bevy_tweening::{component_animator_system, TweeningPlugin};
use lottie_core::prelude::{Id as TimelineItemId, StyledShape, TimelineAction};
use lottie_core::*;
use render::*;
use std::cmp::min;

use bevy::prelude::Transform;

#[derive(TypeUuid)]
#[uuid = "760e41e4-94c0-44e7-bbc8-f00ea42d2420"]
pub struct PrecompositionAsset {
    data: Precomposition,
}

#[derive(Component)]
pub struct LottieComp {
    lottie: Lottie,
    asset_handles: HashMap<String, Handle<PrecompositionAsset>>,
}

#[derive(Component)]
struct LottieShapeComp(StyledShape);

#[derive(Component)]
struct LayerId(TimelineItemId);

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
    entities: HashMap<TimelineItemId, Entity>,
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
            .add_system(component_animator_system::<Path>)
            .add_system(animate_system);
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
        entities: HashMap::new(),
    });
    commands.spawn_bundle(camera);

    lottie.scale = scale;
    let comp = LottieComp {
        lottie,
        asset_handles: HashMap::new(),
    };
    commands
        .spawn()
        .insert(comp)
        .insert_bundle(TransformBundle::default());
}

fn animate_system(
    mut commands: Commands,
    query: Query<(Entity, &LayerAnimationInfo)>,
    comp: Query<&LottieComp>,
    mut info: ResMut<LottieAnimationInfo>,
    time: Res<Time>,
) {
    let frame_window = (time.delta_seconds() * (info.frame_rate as f32)).round() as u32;
    let frame_window = min(info.end_frame - info.current_frame, frame_window);
    let comp = comp.get_single().unwrap();
    for delta in 0..frame_window {
        let frame = info.current_frame + delta;
        let items = comp
            .lottie
            .timeline()
            .events_at(frame)
            .into_iter()
            .flatten();
        for item in items {
            match item {
                TimelineAction::Spawn(id) => {
                    if let Some(layer) = comp.lottie.timeline().item(*id) {
                        let entity = layer.spawn(info.current_frame + frame_window, &mut commands);
                        if let Some(parent_entity) =
                            layer.parent.and_then(|id| info.entities.get(&id))
                        {
                            commands.entity(*parent_entity).add_child(entity);
                        }
                    }
                }
                _ => {} // Skip destory event as we are destroying directly from bevy
            }
        }
    }

    info.current_frame = info.current_frame + frame_window;

    // Destory ended layers
    for (entity, layer_info) in query.iter() {
        if layer_info.end_frame <= info.current_frame {
            commands.entity(entity).despawn_recursive();
        }
    }

    info.current_frame %= info.end_frame;
}
