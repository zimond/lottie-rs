mod asset;
mod lens;
mod render;
mod tween;
mod utils;

use asset::PrecompositionAsset;
use bevy::app::PluginGroupBuilder;
use bevy::ecs::schedule::IntoSystemDescriptor;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy::winit::WinitPlugin;
// use bevy_diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_prototype_lyon::prelude::*;
use bevy_tweening::{component_animator_system, TweeningPlugin};
use lottie_core::prelude::{Id as TimelineItemId, StyledShape, TimelineAction};
use lottie_core::*;
use render::*;

use bevy::prelude::Transform;

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
    current_time: f32,
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
        plugin_group_builder.disable::<LogPlugin>();
        plugin_group_builder.finish(&mut app);
        app.insert_resource(Msaa { samples: 4 })
            .add_plugin(TweeningPlugin)
            // .add_plugin(FrameTimeDiagnosticsPlugin)
            // .add_plugin(LogDiagnosticsPlugin::default())
            .add_plugin(ShapePlugin)
            .add_system(component_animator_system::<Path>)
            .add_system(component_animator_system::<DrawMode>)
            .add_system(animate_system);
        BevyRenderer { app }
    }

    pub fn add_plugin(&mut self, plugin: impl Plugin) {
        self.app.add_plugin(plugin);
    }

    pub fn add_system<Params>(&mut self, system: impl IntoSystemDescriptor<Params>) {
        self.app.add_system(system);
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
        current_time: 0.0,
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
    comp: Query<(Entity, &LottieComp)>,
    mut info: ResMut<LottieAnimationInfo>,
    time: Res<Time>,
) {
    let current_time = time.time_since_startup().as_secs_f32();
    let total_time = (info.end_frame - info.start_frame) as f32 / info.frame_rate as f32;
    let current_time = current_time - (current_time / total_time).floor() * total_time;
    if current_time < info.current_time {
        info.current_time = 0.0;
        info.entities.clear();
        log::trace!("destroy all entities");
        for (entity, _) in comp.iter() {
            commands.entity(entity).despawn_descendants();
        }
    }
    let prev_frame = (info.current_time * info.frame_rate as f32).ceil() as u32;
    let current_frame = (current_time * info.frame_rate as f32).ceil() as u32;

    let (root_entity, comp) = comp.get_single().unwrap();
    let mut unresolved: HashMap<TimelineItemId, Vec<Entity>> = HashMap::new();
    for frame in prev_frame..current_frame {
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
                        let entity = layer.spawn(frame, &mut commands);
                        info.entities.insert(layer.id, entity);
                        if let Some(parent_id) = layer.parent {
                            if let Some(parent_entity) = info.entities.get(&parent_id) {
                                log::trace!("adding {:?} -> {:?}", entity, parent_entity);
                                commands.entity(*parent_entity).add_child(entity);
                            } else {
                                unresolved.entry(parent_id).or_default().push(entity);
                            }
                        } else {
                            log::trace!("adding {:?} -> {:?}", entity, root_entity);
                            commands.entity(root_entity).add_child(entity);
                        }
                        if let Some(entities) = unresolved.remove(&layer.id) {
                            let mut current = commands.entity(entity);
                            for entity in entities {
                                current.add_child(entity);
                            }
                        }
                    }
                }
                _ => {} // Skip destory event as we are destroying directly from bevy
            }
        }
    }

    // Destory ended layers
    for (entity, layer_info) in query.iter() {
        if layer_info.end_frame < current_frame {
            commands.entity(entity).despawn_recursive();
        }
    }

    info.current_time = current_time;
}
