mod asset;
mod lens;
mod render;
mod tween;
mod utils;

use asset::PrecompositionAsset;
use bevy::app::PluginGroupBuilder;
use bevy::ecs::schedule::IntoSystemDescriptor;
use bevy::prelude::*;
use bevy::render::view::VisibilityPlugin;
use bevy::utils::HashMap;
use bevy::winit::WinitPlugin;
use bevy_diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_prototype_lyon::prelude::*;
use bevy_tweening::{component_animator_system, Animator, AnimatorState, TweeningPlugin};
use lottie_core::prelude::{Id as TimelineItemId, StyledShape};
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

pub struct LottieAnimationInfo {
    start_frame: f32,
    end_frame: f32,
    frame_rate: f32,
    current_time: f32,
    paused: bool,
    entities: HashMap<TimelineItemId, Entity>,
}

impl LottieAnimationInfo {
    pub fn progress(&self) -> f32 {
        self.current_time / (self.end_frame - self.start_frame) * self.frame_rate
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    pub fn pause(&mut self, pause: bool) {
        self.paused = pause;
    }

    pub fn current_time(&self) -> f32 {
        self.current_time
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
            // .insert_resource(ClearColor(Color::rgb(1.0, 1.0, 1.0)))
            .add_plugin(TweeningPlugin)
            .add_plugin(VisibilityPlugin)
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
    commands.spawn_bundle(camera);

    lottie.scale = scale;
    let mut info = LottieAnimationInfo {
        start_frame: lottie.model.start_frame,
        end_frame: lottie.model.end_frame,
        frame_rate: lottie.model.frame_rate,
        current_time: 0.0,
        paused: false,
        entities: HashMap::new(),
    };

    let root_entity = commands
        .spawn()
        .insert_bundle(TransformBundle::default())
        .id();
    let mut unresolved: HashMap<TimelineItemId, Vec<Entity>> = HashMap::new();
    for layer in lottie.timeline().items() {
        let entity = layer.spawn(&mut commands);
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

    commands.insert_resource(info);

    let comp = LottieComp {
        lottie,
        asset_handles: HashMap::new(),
    };
    commands.entity(root_entity).insert(comp);
}

fn animate_system(
    mut visibility_query: Query<(Entity, &mut Visibility, &FrameTracker)>,
    mut transform_animation: Query<(&mut Animator<Transform>, &FrameTracker)>,
    mut path_animation: Query<(&mut Animator<Path>, &FrameTracker)>,
    mut draw_mode_animation: Query<(&mut Animator<DrawMode>, &FrameTracker)>,
    mut info: ResMut<LottieAnimationInfo>,
    time: Res<Time>,
) {
    if info.paused {
        for (mut a, _) in transform_animation.iter_mut() {
            a.state = AnimatorState::Paused;
        }
        return;
    }
    let current_time = info.current_time + time.delta_seconds();
    let total_time = info.end_frame / info.frame_rate;
    let current_time = current_time - (current_time / total_time).floor() * total_time;
    if current_time < info.current_time {
        info.current_time = 0.0;
    }
    let current_frame = current_time * info.frame_rate;

    for (mut a, tracker) in transform_animation.iter_mut() {
        if let Some(frame) = tracker.value(current_frame) {
            a.state = AnimatorState::Playing;
            let secs = frame / tracker.frame_rate();
            let total = a.tweenable().unwrap().duration().as_secs_f32();
            a.set_progress(secs / total);
        } else {
            a.state = AnimatorState::Paused
        }
    }

    for (mut a, tracker) in path_animation.iter_mut() {
        if let Some(frame) = tracker.value(current_frame) {
            a.state = AnimatorState::Playing;
            let secs = frame / tracker.frame_rate();
            let total = a.tweenable().unwrap().duration().as_secs_f32();
            a.set_progress(secs / total);
        } else {
            a.state = AnimatorState::Paused
        }
    }

    for (mut a, tracker) in draw_mode_animation.iter_mut() {
        if let Some(frame) = tracker.value(current_frame) {
            a.state = AnimatorState::Playing;
            let secs = frame / tracker.frame_rate();
            let total = a.tweenable().unwrap().duration().as_secs_f32();
            a.set_progress(secs / total);
        } else {
            a.state = AnimatorState::Paused
        }
    }

    // let mut hidden = VecDeque::new();
    for (entity, mut visibility, tracker) in visibility_query.iter_mut() {
        let visible = tracker.value(current_frame).is_some();
        visibility.is_visible = visible;
        // if !visible {
        //     hidden.push_back(entity);
        // }
    }

    // while let Some(entity) = hidden.pop_front() {
    //     for child in hierarchy
    //         .get(entity)
    //         .into_iter()
    //         .map(|children| children.iter())
    //         .flatten()
    //     {
    //         if let Ok((_, mut visibility, _)) = visibility_query.get_mut(*child)
    // {             visibility.is_visible = false;
    //         }
    //         hidden.push_back(*child);
    //     }
    // }

    // let (root_entity, comp) = comp.get_single().unwrap();
    // for item in comp.lottie.timeline().events_in(prev_frame, current_frame) {
    //     match item {
    //         TimelineAction::Spawn(id) => if let Some(layer) =
    // comp.lottie.timeline().item(*id) {},         _ => {} // Skip destory
    // event as we are destroying directly from bevy     }
    // }

    // // Destory ended layers
    // for (entity, layer_info) in query.iter() {
    //     let current_frame = if let Some(remapping) =
    // layer_info.time_remapping.as_ref() {         let current_time =
    // remapping.value(current_frame);         current_time * info.frame_rate
    //     } else {
    //         current_frame
    //     };
    //     if layer_info.end_frame < current_frame {
    //         commands.entity(entity).despawn_recursive();
    //     }
    // }

    info.current_time = current_time;
}
