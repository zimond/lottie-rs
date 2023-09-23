use bevy::math::Vec2;
use bevy::prelude::{Color, Handle, Image, UVec4, Vec4};
use bevy::reflect::{TypePath, TypeUuid};
use bevy::render::mesh::MeshVertexBufferLayout;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderRef, ShaderType, SpecializedMeshPipelineError,
    VertexBufferLayout,
};
use bevy::sprite::{Material2d, Material2dKey};
use lottie_core::GradientColor;
use wgpu::*;

#[derive(AsBindGroup, TypeUuid, Clone, TypePath)]
#[uuid = "e66b6c0e-bcac-4128-bdc6-9a3cace5c2fc"]
// #[uniform(3, GradientDataUniform)]
#[bind_group_data(GradientDataKey)]
pub struct LottieMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub mask: Option<Handle<Image>>,
    // width, height, scale
    #[uniform(2)]
    pub size: Vec4,
    /// Mask index, mask count, matte mode
    #[uniform(3)]
    pub mask_info: MaskDataUniform,
    #[uniform(4)]
    pub gradient: GradientDataUniform,
}

impl Material2d for LottieMaterial {
    fn vertex_shader() -> ShaderRef {
        "shader.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shader.wgsl".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _: &MeshVertexBufferLayout,
        key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // Customize how to store the meshes' vertex attributes in the vertex buffer
        // Our meshes only have position and color
        let formats = vec![
            // Position
            VertexFormat::Float32x2,
            // Normal
            VertexFormat::Float32x3,
            // UV
            VertexFormat::Float32x2,
            // Color
            VertexFormat::Uint32,
        ];

        let vertex_layout =
            VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, formats);
        descriptor.vertex.buffers = vec![vertex_layout];

        if key.bind_group_data.use_gradient {
            let fragment = descriptor.fragment.as_mut().unwrap();
            fragment.shader_defs.push("USE_GRADIENT".into());
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct GradientInfo {
    pub start_pos: Vec2,
    pub end_pos: Vec2,
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct GradientDataKey {
    use_gradient: bool,
}

#[derive(Clone, Default, ShaderType)]
pub struct GradientDataUniform {
    pub start: Vec2,
    pub end: Vec2,
    // #[size(runtime)]
    // TODO: change this to a Vec (which compiles to a storage buffer) when bevy supports it
    // tracking: https://github.com/bevyengine/bevy/issues/5499
    pub use_gradient: u32,
    pub stops: [GradientDataStop; 2],
}

#[derive(Clone, Default, ShaderType)]
pub struct MaskDataUniform {
    // #[size(runtime)]
    // TODO: change this to a Vec (which compiles to a storage buffer) when bevy supports it
    // tracking: https://github.com/bevyengine/bevy/issues/5499
    pub masks: [UVec4; 4],
    pub mask_count: u32,
    pub mask_total_count: u32,
}

#[derive(Clone, Default, ShaderType)]
pub struct GradientDataStop {
    pub offset: f32,
    pub color: Color,
}

impl<'a> From<&'a GradientColor> for GradientDataStop {
    fn from(stop: &'a GradientColor) -> Self {
        GradientDataStop {
            offset: stop.offset,
            color: Color::Rgba {
                red: stop.color.r as f32 / 255.0,
                green: stop.color.g as f32 / 255.0,
                blue: stop.color.b as f32 / 255.0,
                alpha: stop.color.a as f32 / 255.0,
            },
        }
    }
}

impl From<&LottieMaterial> for GradientDataKey {
    fn from(material: &LottieMaterial) -> Self {
        Self {
            use_gradient: material.gradient.stops.is_empty(),
        }
    }
}
