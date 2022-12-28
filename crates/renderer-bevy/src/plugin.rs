use bevy::prelude::*;
use bevy::render::mesh::{Indices, MeshVertexAttribute};
use bevy::render::render_resource::*;
use bevy::sprite::*;
use lyon::lyon_tessellation::*;
use lyon::path::Path as LyonPath;

use crate::material::LottieMaterial;
use crate::shape::*;

#[derive(Component, Clone, Copy)]
pub struct MaskMarker;

/// A vertex with all the necessary attributes to be inserted into a Bevy
/// [`Mesh`](bevy::render::mesh::Mesh).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: u32,
    /// Use a texture instead of solid color, when this field contains a valid
    /// coord, `color` is ignored
    pub texture_anchor: [f32; 2],
}

type IndexType = u32;

pub type VertexBuffers = lyon::tessellation::VertexBuffers<Vertex, IndexType>;

/// Zero-sized type used to implement various vertex construction traits from
/// Lyon.
pub enum VertexConstructor {
    Solid(Color),
    Texture { anchor: [f32; 2], opacity: f32 },
}

impl VertexConstructor {
    fn from_color(color: &SolidOrGradient, opacity: f32) -> Self {
        match color {
            SolidOrGradient::Solid(c) => {
                let mut c = *c;
                c.set_a(c.a() * opacity);
                VertexConstructor::Solid(c)
            }
            SolidOrGradient::Gradient(g) => unimplemented!(),
        }
    }
}

/// Enables the construction of a [`Vertex`] when using a `FillTessellator`.
impl FillVertexConstructor<Vertex> for VertexConstructor {
    fn new_vertex(&mut self, vertex: FillVertex) -> Vertex {
        match self {
            VertexConstructor::Solid(color) => Vertex {
                position: vertex.position().to_array(),
                color: color.as_linear_rgba_u32(),
                texture_anchor: [-1.0, -1.0],
            },
            VertexConstructor::Texture { anchor, opacity } => {
                let mut color = Color::WHITE;
                color.set_a(*opacity);
                Vertex {
                    position: vertex.position().to_array(),
                    color: color.as_linear_rgba_u32(),
                    texture_anchor: *anchor,
                }
            }
        }
    }
}

/// Enables the construction of a [`Vertex`] when using a `StrokeTessellator`.
impl StrokeVertexConstructor<Vertex> for VertexConstructor {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> Vertex {
        match self {
            VertexConstructor::Solid(color) => Vertex {
                position: vertex.position().to_array(),
                color: color.as_linear_rgba_u32(),
                texture_anchor: [-1.0, -1.0],
            },
            VertexConstructor::Texture { anchor, opacity } => {
                let mut color = Color::WHITE;
                color.set_a(*opacity);
                Vertex {
                    position: vertex.position().to_array(),
                    color: color.as_linear_rgba_u32(),
                    texture_anchor: *anchor,
                }
            }
        }
    }
}

/// [`SystemLabel`] for the system that builds the meshes for newly-added
/// or changed shapes. Resides in [`PostUpdate`](CoreStage::PostUpdate).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
pub struct BuildShapes;

#[derive(Resource, Deref, DerefMut)]
pub struct FillTessRes(FillTessellator);

#[derive(Resource, Deref, DerefMut)]
pub struct StrokeTessRes(StrokeTessellator);

pub struct LottiePlugin;

impl Plugin for LottiePlugin {
    fn build(&self, app: &mut App) {
        let fill_tess = FillTessellator::new();
        let stroke_tess = StrokeTessellator::new();
        app.insert_resource(FillTessRes(fill_tess))
            .insert_resource(StrokeTessRes(stroke_tess))
            .add_plugin(Material2dPlugin::<LottieMaterial>::default())
            .add_system_to_stage(CoreStage::PostUpdate, mesh_shapes_system.label(BuildShapes));
    }
}

/// Queries all the [`ShapeBundle`]s to mesh them when they are added
/// or re-mesh them when they are changed.
#[allow(clippy::type_complexity)]
fn mesh_shapes_system(
    mut meshes: ResMut<Assets<Mesh>>,
    mut fill_tess: ResMut<FillTessRes>,
    mut stroke_tess: ResMut<StrokeTessRes>,
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
fn fill(tess: &mut ResMut<FillTessRes>, path: &LyonPath, mode: &Fill, buffers: &mut VertexBuffers) {
    if let Err(e) = tess.tessellate_path(
        path,
        &mode.options,
        &mut BuffersBuilder::new(
            buffers,
            VertexConstructor::from_color(&mode.color, mode.opacity),
        ),
    ) {
        error!("FillTessellator error: {:?}", e);
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)] // lyon takes &StrokeOptions
fn stroke(
    tess: &mut ResMut<StrokeTessRes>,
    path: &LyonPath,
    mode: &Stroke,
    buffers: &mut VertexBuffers,
) {
    if let Err(e) = tess.tessellate_path(
        path,
        &mode.options,
        &mut BuffersBuilder::new(
            buffers,
            VertexConstructor::from_color(&mode.color, mode.opacity),
        ),
    ) {
        error!("StrokeTessellator error: {:?}", e);
    }
}

fn build_mesh(buffers: &VertexBuffers) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U32(buffers.indices.clone())));
    let verts = buffers
        .vertices
        .iter()
        .map(|v| v.position)
        .collect::<Vec<[f32; 2]>>();
    let len = verts.len();

    mesh.insert_attribute(
        MeshVertexAttribute::new("Vertex_Position", 0, VertexFormat::Float32x2),
        verts,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 0.0]; len]);
    mesh.insert_attribute(
        MeshVertexAttribute::new("Vertex_Color", 4, VertexFormat::Uint32),
        buffers
            .vertices
            .iter()
            .map(|v| v.color)
            .collect::<Vec<u32>>(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; len]);
    mesh
}
