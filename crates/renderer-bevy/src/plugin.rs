use bevy::core_pipeline::core_2d::Transparent2d;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::extract_component::ExtractComponentPlugin;
use bevy::render::mesh::{Indices, MeshVertexAttribute};
use bevy::render::render_asset::{RenderAssetPlugin, RenderAssets};
use bevy::render::render_phase::*;
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::BevyDefault;
use bevy::render::view::VisibleEntities;
use bevy::render::{Extract, RenderApp, RenderStage};
use bevy::sprite::*;
use bevy::utils::FloatOrd;
use lyon::lyon_tessellation::*;
use lyon::path::Path as LyonPath;

use crate::material::MaskAwareMaterial;
use crate::shape::*;

#[derive(Component, Clone, Copy)]
pub struct MaskMarker;

/// A vertex with all the necessary attributes to be inserted into a Bevy
/// [`Mesh`](bevy::render::mesh::Mesh).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: u32,
}

type IndexType = u32;

pub type VertexBuffers = lyon::tessellation::VertexBuffers<Vertex, IndexType>;

/// Zero-sized type used to implement various vertex construction traits from
/// Lyon.
pub struct VertexConstructor {
    pub color: Color,
}

/// Enables the construction of a [`Vertex`] when using a `FillTessellator`.
impl FillVertexConstructor<Vertex> for VertexConstructor {
    fn new_vertex(&mut self, vertex: FillVertex) -> Vertex {
        Vertex {
            position: vertex.position().to_array(),
            color: self.color.as_linear_rgba_u32(),
        }
    }
}

/// Enables the construction of a [`Vertex`] when using a `StrokeTessellator`.
impl StrokeVertexConstructor<Vertex> for VertexConstructor {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> Vertex {
        Vertex {
            position: vertex.position().to_array(),
            color: self.color.as_linear_rgba_u32(),
        }
    }
}

/// [`SystemLabel`] for the system that builds the meshes for newly-added
/// or changed shapes. Resides in [`PostUpdate`](CoreStage::PostUpdate).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
pub struct BuildShapes;

pub struct MaskedShapePlugin;

impl Plugin for MaskedShapePlugin {
    fn build(&self, app: &mut App) {
        let fill_tess = FillTessellator::new();
        let stroke_tess = StrokeTessellator::new();
        app.insert_resource(fill_tess)
            .insert_resource(stroke_tess)
            .add_plugin(MaskedMesh2dPlugin)
            .add_system_to_stage(CoreStage::PostUpdate, mesh_shapes_system.label(BuildShapes));
    }
}

/// Queries all the [`ShapeBundle`]s to mesh them when they are added
/// or re-mesh them when they are changed.
#[allow(clippy::type_complexity)]
fn mesh_shapes_system(
    mut meshes: ResMut<Assets<Mesh>>,
    mut fill_tess: ResMut<FillTessellator>,
    mut stroke_tess: ResMut<StrokeTessellator>,
    mut query: Query<(&DrawMode, &Path, &mut Mesh2dHandle), Or<(Changed<Path>, Changed<DrawMode>)>>,
) {
    for (tess_mode, path, mut mesh) in query.iter_mut() {
        let mut buffers = VertexBuffers::new();

        if let Some(fill_mode) = tess_mode.fill.as_ref() {
            fill(&mut fill_tess, &path.0, fill_mode, &mut buffers);
        }
        if let Some(stroke_mode) = tess_mode.stroke.as_ref() {
            stroke(&mut stroke_tess, &path.0, stroke_mode, &mut buffers);
        }

        mesh.0 = meshes.add(build_mesh(&buffers));
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)] // lyon takes &FillOptions
fn fill(
    tess: &mut ResMut<FillTessellator>,
    path: &LyonPath,
    mode: &Fill,
    buffers: &mut VertexBuffers,
) {
    if let Err(e) = tess.tessellate_path(
        path,
        &mode.options,
        &mut BuffersBuilder::new(buffers, VertexConstructor { color: mode.color }),
    ) {
        error!("FillTessellator error: {:?}", e);
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)] // lyon takes &StrokeOptions
fn stroke(
    tess: &mut ResMut<StrokeTessellator>,
    path: &LyonPath,
    mode: &Stroke,
    buffers: &mut VertexBuffers,
) {
    if let Err(e) = tess.tessellate_path(
        path,
        &mode.options,
        &mut BuffersBuilder::new(buffers, VertexConstructor { color: mode.color }),
    ) {
        error!("StrokeTessellator error: {:?}", e);
    }
}

fn build_mesh(buffers: &VertexBuffers) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U32(buffers.indices.clone())));
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        buffers
            .vertices
            .iter()
            .map(|v| [v.position[0], v.position[1], 0.0])
            .collect::<Vec<[f32; 3]>>(),
    );
    mesh.insert_attribute(
        MeshVertexAttribute::new("Vertex_Color", 1, VertexFormat::Uint32),
        buffers
            .vertices
            .iter()
            .map(|v| v.color)
            .collect::<Vec<u32>>(),
    );

    mesh
}

/// Custom pipeline for 2d meshes with vertex colors
pub struct MaskedMesh2dPipeline {
    /// this pipeline wraps the standard [`Mesh2dPipeline`]
    pub mesh2d_pipeline: Mesh2dPipeline,
    pub material2d_layout: BindGroupLayout,
    mask_layout: BindGroupLayout,
}

impl FromWorld for MaskedMesh2dPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let layout =
            <MaskAwareMaterial as bevy::sprite::Material2d>::bind_group_layout(&render_device);
        let mask_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(Vec2::min_size().get()),
                    },
                    count: None,
                },
            ],
            label: Some("mask_mesh2d_view_layout"),
        });
        Self {
            mesh2d_pipeline: Mesh2dPipeline::from_world(world),
            material2d_layout: layout,
            mask_layout,
        }
    }
}

#[derive(Eq, PartialEq, Clone, Hash)]
pub struct MaskMesh2dKey {
    pub mesh: Mesh2dPipelineKey,
    pub material: (),
}

// We implement `SpecializedPipeline` to customize the default rendering from
// `Mesh2dPipeline`
impl SpecializedRenderPipeline for MaskedMesh2dPipeline {
    type Key = MaskMesh2dKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        // Customize how to store the meshes' vertex attributes in the vertex buffer
        // Our meshes only have position and color
        let formats = vec![
            // Position
            VertexFormat::Float32x3,
            // Color
            VertexFormat::Uint32,
        ];

        let vertex_layout =
            VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, formats);

        RenderPipelineDescriptor {
            vertex: VertexState {
                // Use our custom shader
                shader: COLORED_MESH2D_SHADER_HANDLE.typed::<Shader>(),
                entry_point: "vertex".into(),
                shader_defs: Vec::new(),
                // Use our custom vertex buffer
                buffers: vec![vertex_layout],
            },
            fragment: Some(FragmentState {
                // Use our custom shader
                shader: COLORED_MESH2D_SHADER_HANDLE.typed::<Shader>(),
                shader_defs: Vec::new(),
                entry_point: "fragment".into(),
                targets: vec![ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            // Use the two standard uniforms for 2d meshes
            layout: Some(vec![
                // Bind group 0 is the view uniform
                self.mesh2d_pipeline.view_layout.clone(),
                // Bind group 1 is the mesh uniform
                self.mesh2d_pipeline.mesh_layout.clone(),
                // Bind group 2 is the mask uniform
                self.mask_layout.clone(),
            ]),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: key.mesh.primitive_topology(),
                strip_index_format: None,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: key.mesh.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some("masked_mesh2d_pipeline".into()),
        }
    }
}

// This specifies how to render a colored 2d mesh
type DrawMaskedMesh2d = (
    // Set the pipeline
    SetItemPipeline,
    // Set the view uniform as bind group 0
    SetMesh2dViewBindGroup<0>,
    // Set the mesh uniform as bind group 1
    SetMesh2dBindGroup<1>,
    SetMaterial2dBindGroup<MaskAwareMaterial, 2>,
    // Draw the mesh
    DrawMesh2d,
);

/// Plugin that renders [`Shape`]s
pub struct MaskedMesh2dPlugin;

/// Handle to the custom shader with a unique random ID
pub const COLORED_MESH2D_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 13828845428412094821);

impl Plugin for MaskedMesh2dPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<MaskAwareMaterial>()
            .add_plugin(ExtractComponentPlugin::<Handle<MaskAwareMaterial>>::extract_visible())
            .add_plugin(RenderAssetPlugin::<MaskAwareMaterial>::default());
        // Load our custom shader
        let mut shaders = app.world.resource_mut::<Assets<Shader>>();
        shaders.set_untracked(
            COLORED_MESH2D_SHADER_HANDLE,
            Shader::from_wgsl(include_str!("../../../assets/shader.wgsl")),
        );

        // Register our custom draw function and pipeline, and add our render systems
        let render_app = app.get_sub_app_mut(RenderApp).unwrap();
        render_app
            .add_render_command::<Transparent2d, DrawMaskedMesh2d>()
            .init_resource::<MaskedMesh2dPipeline>()
            .init_resource::<SpecializedRenderPipelines<MaskedMesh2dPipeline>>()
            .add_system_to_stage(RenderStage::Extract, extract_colored_mesh2d)
            .add_system_to_stage(RenderStage::Queue, queue_colored_mesh2d);
    }
}

/// Extract the [`Shape`] marker component into the render app
pub fn extract_colored_mesh2d(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    query: Extract<Query<(Entity, &ComputedVisibility), With<Shape>>>,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, computed_visibility) in query.iter() {
        if !computed_visibility.is_visible {
            continue;
        }
        values.push((entity, (Shape,)));
    }
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}

/// Queue the 2d meshes marked with [`Shape`] using our custom pipeline
/// and draw function
#[allow(clippy::too_many_arguments)]
pub fn queue_colored_mesh2d(
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    colored_mesh2d_pipeline: Res<MaskedMesh2dPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<MaskedMesh2dPipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    msaa: Res<Msaa>,
    render_meshes: Res<RenderAssets<Mesh>>,
    render_materials: Res<RenderAssets<MaskAwareMaterial>>,
    colored_mesh2d: Query<(&Handle<MaskAwareMaterial>, &Mesh2dHandle, &Mesh2dUniform), With<Shape>>,
    mut views: Query<(&VisibleEntities, &mut RenderPhase<Transparent2d>)>,
) {
    if colored_mesh2d.is_empty() {
        return;
    }
    // Iterate each view (a camera is a view)
    for (visible_entities, mut transparent_phase) in views.iter_mut() {
        let draw_masked_2d = transparent_draw_functions
            .read()
            .get_id::<DrawMaskedMesh2d>()
            .unwrap();

        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples);

        // Queue all entities visible to that view
        for visible_entity in &visible_entities.entities {
            if let Ok((material_handle, mesh2d_handle, mesh2d_uniform)) =
                colored_mesh2d.get(*visible_entity)
            {
                // Get our specialized pipeline
                let mut mesh2d_key = mesh_key;
                if let Some(material) = render_materials.get(material_handle) {
                    if let Some(mesh) = render_meshes.get(&mesh2d_handle.0) {
                        mesh2d_key |=
                            Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology);

                        let pipeline_id = pipelines.specialize(
                            &mut pipeline_cache,
                            &colored_mesh2d_pipeline,
                            MaskMesh2dKey {
                                mesh: mesh2d_key,
                                material: (),
                            },
                        );

                        let mesh_z = mesh2d_uniform.transform.w_axis.z;
                        transparent_phase.add(Transparent2d {
                            entity: *visible_entity,
                            draw_function: draw_masked_2d,
                            pipeline: pipeline_id,
                            // The 2d render items are sorted according to their z value before
                            // rendering, in order to get correct
                            // transparency
                            sort_key: FloatOrd(mesh_z),
                            // This material is not batched
                            batch_range: None,
                        });
                    }
                }
            }
        }
    }
}
