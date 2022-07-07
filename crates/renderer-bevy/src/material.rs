use bevy::ecs::system::lifetimeless::SRes;
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::{AssetServer, Handle, Shader};
use bevy::reflect::TypeUuid;
use bevy::render::render_asset::{PrepareAssetError, RenderAsset};
use bevy::render::render_resource::{
    encase, BindGroup, BindGroupLayout, ShaderType, UniformBuffer,
};
use bevy::render::renderer::RenderDevice;
use bevy::sprite::{Material2d, Material2dPipeline};
use wgpu::util::BufferInitDescriptor;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, BufferBindingType, BufferSize, BufferUsages, ShaderStages,
};

use crate::plugin::MaskedMesh2dPipeline;

#[derive(TypeUuid, Clone)]
#[uuid = "e66b6c0e-bcac-4128-bdc6-9a3cace5c2fc"]
pub struct MaskAwareMaterial {
    pub data: f32,
}

pub struct MaskAwareMaterialGPU {
    bind_group: BindGroup,
}

impl Material2d for MaskAwareMaterial {
    fn bind_group(material: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &material.bind_group
    }

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(f32::min_size().get()),
                },
                count: None,
            }],
        })
    }
}

impl RenderAsset for MaskAwareMaterial {
    type ExtractedAsset = MaskAwareMaterial;

    type PreparedAsset = MaskAwareMaterialGPU;

    type Param = (SRes<RenderDevice>, SRes<MaskedMesh2dPipeline>);

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (render_device, pipeline): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let mut buffer = encase::UniformBuffer::new(Vec::new());
        buffer.write(&extracted_asset.data).unwrap();
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: None,
            contents: buffer.as_ref(),
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pipeline.material2d_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        Ok(MaskAwareMaterialGPU { bind_group })
    }
}
