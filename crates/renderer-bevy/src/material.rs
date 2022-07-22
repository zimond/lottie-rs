use bevy::math::Vec2;
use bevy::prelude::{Handle, Image};
use bevy::reflect::TypeUuid;
use bevy::render::mesh::MeshVertexBufferLayout;
use bevy::render::render_resource::{
    AsBindGroup, BindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
    VertexBufferLayout,
};
use bevy::sprite::{Material2d, Material2dKey};
use wgpu::*;

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "e66b6c0e-bcac-4128-bdc6-9a3cace5c2fc"]
pub struct MaskMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub mask: Option<Handle<Image>>,
    #[uniform(2)]
    pub size: Vec2,
    #[uniform(3)]
    pub texture_index: Vec2,
}

impl Material2d for MaskMaterial {
    fn vertex_shader() -> ShaderRef {
        "shader.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shader.wgsl".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _: &MeshVertexBufferLayout,
        _: Material2dKey<Self>,
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
        Ok(())
    }
}

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "777b2e54-4580-4cc7-8e86-07e1d7ad6ba3"]
pub struct GradientMaterial {
    #[uniform(0)]
    pub start: Vec2,
    #[uniform(1)]
    pub end: Vec2,
}

impl Material2d for GradientMaterial {
    fn vertex_shader() -> ShaderRef {
        "gradient.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "gradient.wgsl".into()
    }
}
