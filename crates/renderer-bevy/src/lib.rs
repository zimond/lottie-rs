#![feature(let_chains)]

// mod frame_capture;
// mod gradient;
mod lens;
mod material;
mod plugin;
mod render;
mod shape;
mod system;
mod tween;
mod utils;

use std::io::Write;

use bevy::app::{Plugin, PluginGroupBuilder, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::ecs::schedule::IntoSystemDescriptor;
use bevy::ecs::system::Resource;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::TextureFormat;
use bevy::render::renderer::RenderDevice;
use bevy::render::view::{RenderLayers, VisibilityPlugin};
use bevy::utils::HashMap;
use bevy::winit::WinitPlugin;
// use bevy_diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_tweening::{component_animator_system, Animator, AnimatorState, TweeningPlugin};
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

pub struct BevyRenderer {
    app: App,
}

impl BevyRenderer {
    pub fn new(width: u32, height: u32, capture: bool) -> Self {
        let mut app = App::new();
        app.insert_resource(WindowDescriptor {
            width: width as f32,
            height: height as f32,
            resizable: false,
            ..default()
        });
        let mut plugin_group_builder = PluginGroupBuilder::default();
        DefaultPlugins.build(&mut plugin_group_builder);
        // Defaulty disable GUI window
        plugin_group_builder.disable::<WinitPlugin>();
        // Disable gamepad support
        plugin_group_builder.disable::<GilrsPlugin>();
        plugin_group_builder.finish(&mut app);
        app.insert_resource(Msaa { samples: 4 })
            .insert_resource(Capturing(false))
            .add_plugin(TweeningPlugin)
            .add_plugin(VisibilityPlugin)
            // .add_event::<FrameCaptureEvent>()
            // .add_plugin(FrameTimeDiagnosticsPlugin)
            // .add_plugin(LogDiagnosticsPlugin::default())
            .add_plugin(LottiePlugin)
            .add_system(component_animator_system::<Path>)
            .add_system(component_animator_system::<DrawMode>)
            .add_system(animate_system);

        if capture {
            let encoder = Encoder::new((width, height)).unwrap();
            app //.add_plugin(CameraTypePlugin::<CaptureCamera>::default())
                // .add_plugin(FrameCapturePlugin)
                .insert_resource(ClearColor(Color::rgb(1.0, 1.0, 1.0)))
                .insert_non_send_resource(encoder)
                .insert_resource(ScheduleRunnerSettings {
                    run_mode: bevy::app::RunMode::Loop { wait: None },
                })
                .insert_resource(Capturing(true))
                // .insert_resource(Frame {
                //     width,
                //     height,
                //     data: vec![0; (width * height * 4) as usize],
                // })
                .add_plugin(ScheduleRunnerPlugin);
            // .add_system(save_img.after(animate_system));
        }
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
    fn load_lottie(&mut self, lottie: Lottie) {
        self.app
            .insert_resource(Some(lottie))
            .add_startup_system(setup_system);
    }

    fn render(&mut self, config: Config) {
        match config {
            Config::Window(window_conf) => {
                self.app.add_plugin(WinitPlugin);
                #[cfg(feature = "bevy_egui")]
                if window_conf.show_controls {
                    self.app
                        .add_plugin(bevy_egui::EguiPlugin)
                        .add_system(system::controls_system);
                }
            }
            Config::Headless(_) => todo!(),
        }
        self.app.run()
    }
}

fn setup_system(
    mut commands: Commands,
    mut lottie: ResMut<Option<Lottie>>,
    mut image_assets: ResMut<Assets<Image>>,
    mut audio_assets: ResMut<Assets<AudioSource>>,
    mut material_assets: ResMut<Assets<LottieMaterial>>,
    // mut gradient_assets: ResMut<Assets<GradientMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    window: Res<Windows>,
    capturing: Res<Capturing>,
    asset_server: Res<AssetServer>,
    render_device: Res<RenderDevice>,
) {
    asset_server.watch_for_changes().unwrap();
    let scale = window.primary().scale_factor() as f32;
    let mut lottie = lottie.take().unwrap();
    commands.remove_resource::<Lottie>();
    let mut camera = Camera2dBundle::default();
    let transform = Transform::from_scale(Vec3::new(1.0, -1.0, 1.0)).with_translation(Vec3::new(
        lottie.model.width as f32 / 2.0,
        lottie.model.height as f32 / 2.0,
        0.0,
    ));
    camera.transform = transform;
    // Create the mask texture
    let size = Extent3d {
        width: lottie.model.width,
        height: lottie.model.height,
        depth_or_array_layers: 1,
    };
    let mut mask = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("mask_texture"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::bevy_default(),
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
        },
        ..default()
    };
    mask.resize(size);
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
        transform,
        ..default()
    };
    commands
        .spawn_bundle(mask_camera)
        .insert(RenderLayers::layer(1));

    // Create the gradient texture
    // let mut gradient_manager = GradientManager::new(&lottie, &mut image_assets,
    // scale);

    let mut cmd = commands.spawn_bundle(camera);

    if capturing.0 {
        // cmd.with_children(|c| {
        //     let capture = FrameCapture::new_cpu_buffer(
        //         lottie.model.width,
        //         lottie.model.height,
        //         true,
        //         TextureFormat::Rgba8UnormSrgb,
        //         &mut image_assets,
        //         &render_device,
        //     );
        //     let t_camera = Camera2dBundle::new_2d();
        //     let render_target =
        // RenderTarget::Image(capture.gpu_image.clone());
        //     let bundle = Camera2dBundle::<CaptureCamera> {
        //         camera: Camera {
        //             target: render_target,
        //             ..default()
        //         },
        //         orthographic_projection: t_camera.orthographic_projection,
        //         visible_entities: t_camera.visible_entities,
        //         frustum: t_camera.frustum,
        //         transform: Transform::identity(),
        //         global_transform: t_camera.global_transform,
        //         marker: CaptureCamera,
        //     };
        //     c.spawn_bundle(bundle).insert(capture);
        // });
    }

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
    for layer in lottie.timeline().items() {
        let entity = BevyStagedLayer {
            layer,
            meshes: &mut meshes,
            image_assets: &mut image_assets,
            audio_assets: &mut audio_assets,
            material_assets: &mut material_assets,
            // gradient_assets: &mut gradient_assets,
            // gradient: &mut gradient_manager,
            mask_handle: mask_texture_handle.clone(),
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
        if let Some(handle) = audio_handle && !computed_visibility.is_visible() && visible {
            audio.play(handle.clone());
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

// fn save_img(
//     info: Res<LottieAnimationInfo>,
//     captures: Query<&FrameCapture>,
//     render_device: Res<RenderDevice>,
//     mut frame: ResMut<Frame>,
//     mut encoder: NonSendMut<Encoder>,
//     mut exit: EventWriter<AppExit>,
//     // mut event_writer: EventWriter<FrameCaptureEvent>,
// ) {
//     if info.finished_once {
//         let encoder = std::mem::replace(
//             encoder.as_mut(),
//             Encoder::new((info.width as u32, info.height as u32)).unwrap(),
//         );
//         let data = encoder
//             .finalize(((info.end_frame / info.frame_rate) * 1000.0) as i32)
//             .unwrap();
//         let mut f = std::fs::File::create("result.webp").unwrap();
//         f.write_all(&data).unwrap();
//         drop(f);
//         exit.send(AppExit);
//     }
//     for capture in captures.iter() {
//         if let Some(target_buffer) = &capture.target_buffer {
//             match target_buffer {
//                 TargetBuffer::CPUBuffer(target_buffer) => {
//                     target_buffer.get(&render_device, |buf| {
//                         frame.load_buffer(&buf);
//                         encoder
//                             .add_frame(&frame.data, (info.current_time *
// 1000.0) as i32)                             .unwrap();
//                     });
//                 }
//                 _ => continue,
//             }
//         }
//     }
// }
