use bevy::prelude::*;
use bevy::render::texture::BevyDefault;
use lottie_core::{Gradient, Lottie};
use wgpu::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

pub struct GradientManager {
    handle: Handle<Image>,
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
        GradientManager {
            handle: gradient_texture_handle,
        }
    }

    pub fn register(&mut self, gradient: &Gradient) -> usize {
        0
    }
}
