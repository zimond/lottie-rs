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

use std::borrow::Cow;
use std::io::Write;
use std::time::Duration;

use bevy::app::{
    AppExit, Plugin, PluginGroupBuilder, ScheduleRunnerPlugin, ScheduleRunnerSettings,
};
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::ecs::schedule::IntoSystemDescriptor;
use bevy::ecs::system::Resource;
use bevy::prelude::*;
use bevy::render::camera::{RenderTarget, Viewport};
use bevy::render::render_resource::TextureFormat;
use bevy::render::renderer::RenderDevice;
use bevy::render::view::{RenderLayers, VisibilityPlugin};
use bevy::utils::HashMap;
use bevy::window::{ModifiesWindows, WindowSettings};
use bevy::winit::WinitPlugin;
// use bevy_diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_tweening::{component_animator_system, Animator, AnimatorState, TweeningPlugin};
use frame_capture::{ImageCopier, ImageCopyPlugin, ImageToSave};
// use gradient::GradientManager;
// use frame_capture::{
//     CaptureCamera, Frame, FrameCapture, FrameCaptureEvent,
// FrameCapturePlugin, TargetBuffer, };
use lottie_core::prelude::{Id as TimelineItemId, StyledShape};
use lottie_core::*;
use material::LottieMaterial;
use plugin::LottiePlugin;
use render::*;

use bevy::prelude::Transform;
use bevy::render::texture::{BevyDefault, Image};
use shape::{DrawMode, Path};
use webp_animation::Encoder;

pub use bevy;
use wgpu::{Extent3d, TextureDescriptor, TextureDimension, TextureUsages};

#[derive(Component)]
pub struct LottieComp {
    lottie: Lottie,
}

pub struct Capturing(bool);

#[derive(Component)]
struct LottieShapeComp(StyledShape);

#[derive(Component)]
struct LayerId(TimelineItemId);

fn modifies_windows() {}

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

struct WebpEncoder {
    encoder: Encoder,
    width: u32,
    height: u32,
}

impl WebpEncoder {
    pub fn new() -> Self {
        WebpEncoder {
            encoder: Encoder::new((1, 1)).unwrap(),
            width: 1,
            height: 1,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.encoder = Encoder::new((width, height)).unwrap();
        self.width = width;
        self.height = height;
    }

    pub fn finish(&mut self) -> Encoder {
        self.width = 0;
        self.height = 0;
        std::mem::replace(&mut self.encoder, Encoder::new((1, 1)).unwrap())
    }

    pub fn finished(&self) -> bool {
        self.width == 0
    }
}

pub struct BevyRenderer {
    app: App,
}

impl BevyRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        let mut app = App::new();
        app.insert_resource(WindowDescriptor {
            width: width as f32,
            height: height as f32,
            resizable: false,
            ..default()
        });

        BevyRenderer { app }
    }

    pub fn add_plugin(&mut self, plugin: impl Plugin) {
        self.app.add_plugin(plugin);
    }

    pub fn add_system<Params>(&mut self, system: impl IntoSystemDescriptor<Params>) {
        self.app.add_system(system);
    }

    pub fn insert_resource<R: Resource>(&mut self, resource: R) {
        self.app.insert_resource(resource);
    }
}

impl Renderer for BevyRenderer {
    fn load_lottie(&mut self, lottie: Lottie, config: Config) {
        let capture = match &config {
            Config::Window(window_conf) => {
                #[cfg(feature = "bevy_egui")]
                if window_conf.show_controls {
                    self.app
                        .add_plugin(bevy_egui::EguiPlugin)
                        .add_system(system::controls_system);
                }
                false
            }
            Config::Headless(_) => {
                self.app.insert_resource(WindowSettings {
                    add_primary_window: false,
                    exit_on_all_closed: false,
                    close_when_requested: true,
                });
                true
            }
        };
        let mut plugin_group_builder = PluginGroupBuilder::default();
        DefaultPlugins.build(&mut plugin_group_builder);
        // Defaulty disable GUI window
        plugin_group_builder.disable::<WinitPlugin>();
        // Disable gamepad support
        plugin_group_builder.disable::<GilrsPlugin>();
        plugin_group_builder.finish(&mut self.app);

        self.app
            .insert_resource(Msaa { samples: 4 })
            .insert_resource(Capturing(false))
            .add_plugin(TweeningPlugin)
            .add_plugin(VisibilityPlugin)
            // .add_plugin(FrameTimeDiagnosticsPlugin)
            // .add_plugin(LogDiagnosticsPlugin::default())
            .add_plugin(LottiePlugin)
            .add_system(component_animator_system::<Path>)
            .add_system(component_animator_system::<DrawMode>)
            .add_system(animate_system)
            .insert_resource(Some(lottie))
            .add_startup_system(setup_system)
            .insert_resource(config);
        if capture {
            let encoder = WebpEncoder::new();
            self.app
                .add_plugin(ImageCopyPlugin)
                .insert_resource(ClearColor(Color::rgb(1.0, 1.0, 1.0)))
                .insert_non_send_resource(encoder)
                .add_system_to_stage(
                    CoreStage::PostUpdate,
                    // Bevy hard-coded this so use an empty function to prevent warnings
                    modifies_windows.label(ModifiesWindows),
                )
                .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
                    1.0 / 60.0, //Don't run faster than 60fps
                )))
                .insert_resource(Capturing(true))
                .add_plugin(ScheduleRunnerPlugin)
                .add_system_to_stage(CoreStage::PostUpdate, save_img);
        } else {
            self.app.add_plugin(WinitPlugin);
        }
    }

    fn render(&mut self) {
        self.app.run()
    }
}

fn setup_system(
    mut commands: Commands,
    mut lottie: ResMut<Option<Lottie>>,
    mut image_assets: ResMut<Assets<Image>>,
    mut audio_assets: ResMut<Assets<AudioSource>>,
    mut material_assets: ResMut<Assets<LottieMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    window: Res<Windows>,
    config: Res<Config>,
    capturing: Res<Capturing>,
    render_device: Res<RenderDevice>,
) {
    let scale = window
        .get_primary()
        .map(|p| p.scale_factor() as f32)
        .unwrap_or(1.0);
    let mut lottie = lottie.take().unwrap();
    commands.remove_resource::<Lottie>();
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
        .filter(|layer| layer.matte_mode.is_some())
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
            priority: -1,
            ..default()
        },
        transform: Transform::from_scale(Vec3::new(1.0, -1.0, 1.0)).with_translation(Vec3::new(
            mask_size.width as f32 / 2.0,
            mask_size.height as f32 / 2.0,
            0.0,
        )),
        ..default()
    };
    commands
        .spawn_bundle(mask_camera)
        .insert(RenderLayers::layer(1));

    if capturing.0 {
        let target = if let Config::Headless(headless) = &*config {
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
        commands.spawn().insert(ImageCopier::new(
            render_target_image_handle,
            cpu_image_handle.clone(),
            size,
            &render_device,
        ));
        commands
            .spawn()
            .insert(ImageToSave(cpu_image_handle.clone()));
    }

    commands.spawn_bundle(camera);

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
        .spawn()
        .insert_bundle(VisibilityBundle::default())
        .insert_bundle(TransformBundle::default())
        .id();
    let mut unresolved: HashMap<TimelineItemId, Vec<Entity>> = HashMap::new();
    let mut mask_index = 0_u32;
    for layer in lottie.timeline().items() {
        let entity = BevyStagedLayer {
            layer,
            meshes: &mut meshes,
            image_assets: &mut image_assets,
            audio_assets: &mut audio_assets,
            material_assets: &mut material_assets,
            mask_handle: mask_texture_handle.clone(),
            mask_index: &mut mask_index,
            mask_count,
            model_size: Vec2::new(lottie.model.width as f32, lottie.model.height as f32),
            scale,
        }
        .spawn(&mut commands)
        .unwrap();
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

    let comp = LottieComp { lottie };
    commands.entity(root_entity).insert(comp);
}

fn animate_system(
    mut visibility_query: Query<(
        Entity,
        &mut Visibility,
        &ComputedVisibility,
        Option<&Handle<AudioSource>>,
        &FrameTracker,
    )>,
    mut transform_animation: Query<(&mut Animator<Transform>, &FrameTracker)>,
    mut path_animation: Query<(&mut Animator<Path>, &FrameTracker)>,
    mut draw_mode_animation: Query<(&mut Animator<DrawMode>, &FrameTracker)>,
    mut info: ResMut<LottieAnimationInfo>,
    capturing: Res<Capturing>,
    audio: Res<Audio>,
    time: Res<Time>,
) {
    let capturing = capturing.0;
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

    for (_, mut visibility, computed_visibility, audio_handle, tracker) in
        visibility_query.iter_mut()
    {
        let visible = tracker.value(current_frame).is_some();
        if let Some(handle) = audio_handle {
            if !computed_visibility.is_visible() && visible {
                audio.play(handle.clone());
            }
        }
        visibility.is_visible = visible;
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
    info: Res<LottieAnimationInfo>,
    image_to_save: Query<&ImageToSave>,
    mut images: ResMut<Assets<Image>>,
    mut encoder: NonSendMut<WebpEncoder>,
    mut exit: EventWriter<AppExit>,
    // mut event_writer: EventWriter<FrameCaptureEvent>,
) {
    // Capture has 1 frame latency
    let delta = 1.0 / info.frame_rate;
    let mut timestamp = info.current_time - delta;
    if info.finished_once {
        if !encoder.finished() {
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
    if timestamp >= end_time && !encoder.finished() {
        let encoder = encoder.finish();
        let data = encoder.finalize((end_time * 1000.0) as i32).unwrap();
        let mut f = std::fs::File::create("result.webp").unwrap();
        f.write_all(&data).unwrap();
        drop(f);
        exit.send(AppExit);
        return;
    }
    for capture in image_to_save.iter() {
        let image = images.get_mut(capture).unwrap();
        let (width, height) = (image.size().x as u32, image.size().y as u32);
        if encoder.width != width || encoder.height != height {
            encoder.resize(width, height);
        }
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
        encoder
            .encoder
            .add_frame(&data, (timestamp * 1000.0) as i32)
            .unwrap();
    }
}
