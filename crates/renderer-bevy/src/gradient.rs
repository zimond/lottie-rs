use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::shape::Quad;
use bevy::prelude::*;
use bevy::render::camera::{RenderTarget, Viewport};
use bevy::render::texture::BevyDefault;
use bevy::render::view::RenderLayers;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use lottie_core::{AnimatedExt, Gradient, Lottie};
use wgpu::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

use crate::material::GradientMaterial;

pub struct GradientManager {
    handle: Handle<Image>,
    transform: Transform,
    size: UVec2,
    index: u32,
}

impl GradientManager {
    pub fn new(lottie: &Lottie, assets: &mut Assets<Image>) -> Self {
        let gradient_count = lottie.timeline().gradient_count() as u32;
        let size = Extent3d {
            width: std::cmp::max(lottie.model.width * gradient_count, 1),
            height: lottie.model.height,
            depth_or_array_layers: 1,
        };
        let mut gradient_image = Image {
            texture_descriptor: TextureDescriptor {
                label: Some("gradient_texture"),
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
        gradient_image.resize(size);
        let gradient_texture_handle = assets.add(gradient_image);
        let transform =
            Transform::from_scale(Vec3::new(1.0, -1.0, 1.0)).with_translation(Vec3::new(
                lottie.model.width as f32 / 2.0,
                lottie.model.height as f32 / 2.0,
                0.0,
            ));
        GradientManager {
            handle: gradient_texture_handle,
            transform,
            size: UVec2::new(lottie.model.width, lottie.model.height),
            index: 0,
        }
    }

    pub fn register(
        &mut self,
        gradient: &Gradient,
        mesh_assets: &mut Assets<Mesh>,
        gradient_assets: &mut Assets<GradientMaterial>,
        commands: &mut Commands,
    ) -> u32 {
        // Create mesh for gradient
        let mesh = Mesh::from(Quad::new(self.size.as_vec2()));
        let start = gradient.start.initial_value();
        let end = gradient.end.initial_value();
        let material = GradientMaterial {
            start: Vec2::new(start.x, start.y),
            end: Vec2::new(end.x, end.y),
        };
        let material = gradient_assets.add(material);
        let mesh = Mesh2dHandle(mesh_assets.add(mesh));
        commands.spawn_bundle(MaterialMesh2dBundle {
            mesh,
            material,
            ..default()
        });
        // Spawn a new camera for the texture viewport
        let camera = Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::NONE),
            },
            camera: Camera {
                target: RenderTarget::Image(self.handle.clone()),
                priority: -1,
                viewport: Some(Viewport {
                    physical_position: UVec2::new(self.index as u32, 0) * self.size,
                    physical_size: self.size,
                    depth: 0.0..1.0,
                }),
                ..default()
            },
            transform: self.transform,
            ..default()
        };
        commands.spawn_bundle(camera).insert(RenderLayers::layer(2));
        let index = self.index;
        self.index += 1;
        index
    }
}
