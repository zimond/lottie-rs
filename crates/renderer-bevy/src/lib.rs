use std::borrow::Cow;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use bevy::app::{AppExit, Plugin, ScheduleRunnerPlugin};
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::ecs::system::Resource;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::TextureFormat;
use bevy::render::renderer::RenderDevice;
use bevy::render::view::RenderLayers;
use bevy::utils::HashMap;
use bevy::window::{ExitCondition, PrimaryWindow};
use bevy::winit::WinitPlugin;
// use bevy_diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_tweening::{component_animator_system, Animator, AnimatorState, TweeningPlugin};
// use gradient::GradientManager;
// use frame_capture::{
//     CaptureCamera, Frame, FrameCapture, FrameCaptureEvent,
// FrameCapturePlugin, TargetBuffer, };
pub use bevy;
use bevy::prelude::Transform;
use bevy::render::texture::{BevyDefault, Image};
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use lottie_core::prelude::{Id as TimelineItemId, StyledShape};
use lottie_core::*;
use shape::{DrawMode, Path};
use wgpu::{Extent3d, TextureDescriptor, TextureDimension, TextureUsages};

mod frame_capture;
// mod gradient;
mod lens;
mod material;
mod plugin;
mod render;
mod shape;
mod system;
mod tween;
mod utils;

use frame_capture::{ImageCopier, ImageCopyPlugin, ImageToSave};
use material::LottieMaterial;
use ordered_float::OrderedFloat;
use plugin::LottiePlugin;
use render::*;

#[derive(Component)]
pub struct LottieComp {
    lottie: Lottie,
}

#[derive(Component)]
struct LottieShapeComp(StyledShape);

#[derive(Component)]
struct LayerId(TimelineItemId);

#[derive(Resource)]
struct LottieGlobals {
    lottie: Option<Lottie>,
    capturing: bool,
    config: Config,
}

#[derive(Resource)]
pub struct LottieAnimationInfo {
    start_frame: f32,
    end_frame: f32,
    frame_rate: f32,
    current_time: f32,
    paused: bool,
    width: f32,
    height: f32,
    finished_once: bool,
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

#[derive(Resource)]
struct FrameSender {
    sender: UnboundedSender<FrameData>,
    closed: Arc<AtomicBool>,
}

impl FrameSender {
    fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
        self.sender.close_channel();
    }
}

pub struct FrameData {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub timestamp: i32,
}

pub struct BevyRenderer {
    app: App,
    frame_sender: UnboundedSender<FrameData>,
}

impl BevyRenderer {
    pub fn new() -> (Self, UnboundedReceiver<FrameData>) {
        let app = App::new();
        let (sender, receiver) = unbounded();

        (
            BevyRenderer {
                app,
                frame_sender: sender,
            },
            receiver,
        )
    }

    pub fn add_plugin(&mut self, plugin: impl Plugin) {
        self.app.add_plugins(plugin);
    }

    pub fn add_system<M>(&mut self, system: impl IntoSystemConfigs<M>) {
        self.app.add_systems(Update, system);
    }

    pub fn insert_resource<R: Resource>(&mut self, resource: R) {
        self.app.insert_resource(resource);
    }
}

impl Renderer for BevyRenderer {
    fn load_lottie(&mut self, lottie: Lottie, config: Config) {
        let width = lottie.model.width as f32;
        let height = lottie.model.height as f32;
        let capturing = if let Config::Headless(_) = &config {
            true
        } else {
            false
        };
        let default_plugins = DefaultPlugins
            .build()
            // Defaulty disable GUI window
            .disable::<WinitPlugin>()
            // Disable gamepad support
            .disable::<GilrsPlugin>()
            .set(WindowPlugin {
                primary_window: if capturing {
                    None
                } else {
                    Some(Window {
                        resolution: (width, height).into(),
                        ..default()
                    })
                },
                close_when_requested: true,
                exit_condition: if capturing {
                    ExitCondition::DontExit
                } else {
                    ExitCondition::OnAllClosed
                },
            });
        self.app
            .insert_resource(Msaa::Sample4)
            .add_plugins(default_plugins)
            .add_plugins(TweeningPlugin)
            // .add_plugin(FrameTimeDiagnosticsPlugin)
            // .add_plugin(LogDiagnosticsPlugin::default())
            .add_plugins(LottiePlugin)
            .add_systems(Update, component_animator_system::<Path>)
            .add_systems(Update, component_animator_system::<DrawMode>)
            .add_systems(Update, animate_system)
            .add_systems(Startup, setup_system);

        if let Config::Window(window_conf) = &config {
            #[cfg(feature = "bevy_egui")]
            if window_conf.show_controls {
                self.app
                    .add_plugins(bevy_egui::EguiPlugin)
                    .add_systems(Update, system::controls_system);
            }
            #[cfg(feature = "bevy-inspector-egui")]
            if window_conf.show_inspector {
                self.app
                    .add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());
            }
        }

        let frame_rate = lottie.model.frame_rate as f64;
        self.app.insert_resource(LottieGlobals {
            lottie: Some(lottie),
            capturing,
            config,
        });

        if capturing {
            self.app
                .add_plugins(ImageCopyPlugin)
                .insert_resource(ClearColor(Color::rgb(1.0, 1.0, 1.0)))
                .insert_resource(FrameSender {
                    sender: self.frame_sender.clone(),
                    closed: Arc::default(),
                })
                .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                    1.0 / frame_rate,
                )))
                .add_systems(PostUpdate, save_img);
        } else {
            self.app.add_plugins(WinitPlugin);
        }
    }

    fn render(&mut self) {
        self.app.run()
    }
}

fn setup_system(
    mut commands: Commands,
    mut lottie_globals: ResMut<LottieGlobals>,
    mut image_assets: ResMut<Assets<Image>>,
    mut audio_assets: ResMut<Assets<AudioSource>>,
    mut material_assets: ResMut<Assets<LottieMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    window: Query<&Window, With<PrimaryWindow>>,
    render_device: Res<RenderDevice>,
) {
    let scale = if let Ok(primary) = window.get_single() {
        primary.scale_factor() as f32
    } else {
        1.0
    };
    let mut lottie = lottie_globals.lottie.take().unwrap();
    let mut camera = Camera2dBundle::default();
    let transform = Transform::from_scale(Vec3::new(1.0, -1.0, 1.0)).with_translation(Vec3::new(
        lottie.model.width as f32 / 2.0,
        lottie.model.height as f32 / 2.0,
        0.0,
    ));
    camera.transform = transform;
    let mask_count = lottie
        .timeline()
        .items()
        .filter(|layer| layer.is_mask)
        .count() as u32;
    // Create the mask texture
    let mask_size = Extent3d {
        width: std::cmp::max(1, lottie.model.width * mask_count),
        height: lottie.model.height,
        depth_or_array_layers: 1,
    };
    let mut mask = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("mask_texture"),
            size: mask_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::bevy_default(),
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    mask.resize(mask_size);
    let mask_texture_handle = image_assets.add(mask);
    let mask_camera = Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::NONE),
        },
        camera: Camera {
            target: RenderTarget::Image(mask_texture_handle.clone()),
            order: -1,
            ..default()
        },
        transform: Transform::from_scale(Vec3::new(1.0, -1.0, 1.0)).with_translation(Vec3::new(
            mask_size.width as f32 / 2.0,
            mask_size.height as f32 / 2.0,
            0.0,
        )),
        ..default()
    };
    commands.spawn(mask_camera).insert(RenderLayers::layer(1));

    if lottie_globals.capturing {
        let target = if let Config::Headless(headless) = &lottie_globals.config {
            headless.target
        } else {
            Target::Default
        };
        let size = if target == Target::Mask {
            mask_size
        } else {
            Extent3d {
                width: lottie.model.width,
                height: lottie.model.height,
                depth_or_array_layers: 1,
            }
        };

        let mut cpu_image = Image {
            texture_descriptor: TextureDescriptor {
                label: Some("cpu image"),
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            },
            ..Default::default()
        };
        cpu_image.resize(size);
        let cpu_image_handle = image_assets.add(cpu_image);
        let render_target_image_handle = if target == Target::Default {
            let mut render_target_image = Image {
                texture_descriptor: TextureDescriptor {
                    label: Some("render target image"),
                    size,
                    dimension: TextureDimension::D2,
                    format: TextureFormat::Bgra8UnormSrgb,
                    mip_level_count: 1,
                    sample_count: 1,
                    usage: TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_DST
                        | TextureUsages::COPY_SRC
                        | TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                },
                ..Default::default()
            };
            render_target_image.resize(size);
            image_assets.add(render_target_image)
        } else {
            mask_texture_handle.clone()
        };

        if target == Target::Default {
            camera.camera.target = RenderTarget::Image(render_target_image_handle.clone());
        }
        commands.spawn(ImageCopier::new(
            render_target_image_handle,
            cpu_image_handle.clone(),
            size,
            &render_device,
        ));
        commands.spawn(ImageToSave(cpu_image_handle.clone()));
    }

    commands.spawn(camera);

    lottie.scale = 1.0; //scale;
    let mut info = LottieAnimationInfo {
        start_frame: lottie.model.start_frame,
        end_frame: lottie.model.end_frame,
        frame_rate: lottie.model.frame_rate,
        current_time: 0.0,
        paused: false,
        width: lottie.model.width as f32,
        height: lottie.model.height as f32,
        finished_once: false,
        entities: HashMap::new(),
    };

    let root_entity = commands
        .spawn(VisibilityBundle::default())
        .insert(TransformBundle::default())
        .id();
    let mut unresolved: HashMap<TimelineItemId, Vec<Entity>> = HashMap::new();
    let mut mask_index = 0_u32;
    let mut mask_registry = HashMap::new();
    let mut zindexes = lottie
        .timeline()
        .items()
        .map(|layer| OrderedFloat(layer.zindex))
        .collect::<Vec<_>>();
    zindexes.sort();
    // First we spawn all mask layers
    for layer in lottie.timeline().items() {
        if layer.is_mask {
            let prev_zindex = zindexes
                .iter()
                .position(|i| *i == OrderedFloat(layer.zindex))
                .and_then(|pos| {
                    if pos == 0 {
                        None
                    } else {
                        zindexes.get(pos - 1)
                    }
                })
                .cloned()
                .unwrap_or(OrderedFloat(-1.0))
                .0;
            let entity = BevyStagedLayer {
                layer,
                zindex_window: layer.zindex - prev_zindex,
                meshes: &mut meshes,
                image_assets: &mut image_assets,
                audio_assets: &mut audio_assets,
                material_assets: &mut material_assets,
                mask_handle: mask_texture_handle.clone(),
                mask_index: &mut mask_index,
                mask_registry: &mut mask_registry,
                mask_count,
                model_size: Vec2::new(lottie.model.width as f32, lottie.model.height as f32),
                scale,
            }
            .spawn(&mut commands)
            .unwrap();
            info.entities.insert(layer.id, entity);
        }
    }
    for layer in lottie.timeline().items() {
        let entity = if !layer.is_mask {
            let prev_zindex = zindexes
                .iter()
                .position(|i| *i == OrderedFloat(layer.zindex))
                .and_then(|pos| {
                    if pos == 0 {
                        None
                    } else {
                        zindexes.get(pos - 1)
                    }
                })
                .cloned()
                .unwrap_or(OrderedFloat(-1.0))
                .0;
            let entity = BevyStagedLayer {
                zindex_window: layer.zindex - prev_zindex,
                layer,
                meshes: &mut meshes,
                image_assets: &mut image_assets,
                audio_assets: &mut audio_assets,
                material_assets: &mut material_assets,
                mask_handle: mask_texture_handle.clone(),
                mask_index: &mut mask_index,
                mask_registry: &mut mask_registry,
                mask_count,
                model_size: Vec2::new(lottie.model.width as f32, lottie.model.height as f32),
                scale,
            }
            .spawn(&mut commands)
            .unwrap();
            info.entities.insert(layer.id, entity);
            entity
        } else {
            *info.entities.get(&layer.id).unwrap()
        };
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

    let comp = LottieComp { lottie };
    commands.entity(root_entity).insert(comp);
}

fn animate_system(
    mut visibility_query: Query<(
        Entity,
        &mut Visibility,
        &ComputedVisibility,
        Option<(&AudioSink, With<LottieAudio>)>,
        &FrameTracker,
    )>,
    mut transform_animation: Query<(&mut Animator<Transform>, &FrameTracker)>,
    mut path_animation: Query<(&mut Animator<Path>, &FrameTracker)>,
    mut draw_mode_animation: Query<(&mut Animator<DrawMode>, &FrameTracker)>,
    mut info: ResMut<LottieAnimationInfo>,
    lottie: Res<LottieGlobals>,
    time: Res<Time>,
) {
    let capturing = lottie.capturing;
    if info.paused {
        for (mut a, _) in transform_animation.iter_mut() {
            a.state = AnimatorState::Paused;
        }
        return;
    }
    let delta = if capturing {
        1.0 / info.frame_rate
    } else {
        time.delta_seconds()
    };
    let current_frame = info.current_time * info.frame_rate;

    for (mut a, tracker) in transform_animation.iter_mut() {
        let total = a.tweenable().duration().as_secs_f32();
        if total == 0.0 {
            a.tweenable_mut()
                .set_elapsed(Duration::from_secs_f32(total));
        } else if let Some(frame) = tracker.value(current_frame) {
            a.state = AnimatorState::Playing;
            let secs = frame / tracker.frame_rate();
            a.tweenable_mut().set_elapsed(Duration::from_secs_f32(secs));
        } else {
            a.state = AnimatorState::Paused
        }
    }

    for (mut a, tracker) in path_animation.iter_mut() {
        let total = a.tweenable().duration().as_secs_f32();
        if total == 0.0 {
            a.tweenable_mut()
                .set_elapsed(Duration::from_secs_f32(total));
        } else if let Some(frame) = tracker.value(current_frame) {
            a.state = AnimatorState::Playing;
            let secs = (frame / tracker.frame_rate()).max(0.0);
            a.tweenable_mut().set_elapsed(Duration::from_secs_f32(secs));
        } else {
            a.state = AnimatorState::Paused
        }
    }

    for (mut a, tracker) in draw_mode_animation.iter_mut() {
        if let Some(frame) = tracker.value(current_frame) {
            a.state = AnimatorState::Playing;
            let secs = frame / tracker.frame_rate();
            a.tweenable_mut().set_elapsed(Duration::from_secs_f32(secs));
        } else {
            a.state = AnimatorState::Paused
        }
    }

    for (_, mut visibility, computed_visibility, audio_sink, tracker) in visibility_query.iter_mut()
    {
        let visible = tracker.value(current_frame).is_some();
        if let Some(sink) = audio_sink {
            if !computed_visibility.is_visible() && visible {
                sink.0.play();
            } else {
                sink.0.pause();
            }
        }
        *visibility = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    let current_time = info.current_time + delta;
    let total_time = info.end_frame / info.frame_rate;
    let current_time = current_time - (current_time / total_time).floor() * total_time;
    if current_time < info.current_time {
        info.current_time = 0.0;
        info.finished_once = true;
    }

    info.current_time = current_time;
}

fn save_img(
    image_to_save: Query<&ImageToSave>,
    info: Res<LottieAnimationInfo>,
    mut images: ResMut<Assets<Image>>,
    image_sender: Res<FrameSender>,
    mut exit: EventWriter<AppExit>,
    // mut event_writer: EventWriter<FrameCaptureEvent>,
) {
    // Capture has 1 frame latency
    let delta = 1.0 / info.frame_rate;
    let mut timestamp = info.current_time - delta;
    if info.finished_once {
        if !image_sender.is_closed() {
            timestamp += info.end_frame / info.frame_rate;
        } else {
            return;
        }
    }
    // Skip first frame
    timestamp -= 2.0 * delta;
    if timestamp < 0.0 {
        return;
    }
    let end_time = info.end_frame / info.frame_rate;
    if timestamp >= end_time && !image_sender.is_closed() {
        image_sender.close();
        exit.send(AppExit);
        return;
    }
    for capture in image_to_save.iter() {
        let image = images.get_mut(capture).unwrap();
        let (width, height) = (image.size().x as u32, image.size().y as u32);
        let data = &mut image.data;
        if data.is_empty() {
            continue;
        }
        // bgra -> rgba
        for pixel in data.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }
        let unpadded_len = (width * height) as usize * 4;
        let data = if data.len() != unpadded_len {
            // Has padding
            let len = data.len();
            let mut result = Vec::with_capacity(unpadded_len);
            for chunk in data.chunks_exact_mut(len / height as usize) {
                result.extend_from_slice(&chunk[..(unpadded_len / height as usize)]);
            }
            assert_eq!(unpadded_len, result.len());
            Cow::Owned(result)
        } else {
            Cow::Borrowed(data)
        };
        image_sender
            .sender
            .unbounded_send(FrameData {
                data: data.into_owned(),
                width,
                height,
                timestamp: (timestamp * 1000.0) as i32,
            })
            .unwrap();
    }
}
