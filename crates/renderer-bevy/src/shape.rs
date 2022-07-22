use bevy::math::Vec2;
use bevy::prelude::{
    Bundle, Color, Component, ComputedVisibility, Deref, GlobalTransform, Transform, Visibility,
};
use bevy::sprite::Mesh2dHandle;
use lyon::path::Path as LyonPath;
use lyon::tessellation::{FillOptions, StrokeOptions};

/// Marker shape
#[derive(Component, Clone, Copy)]
pub struct Shape;

#[derive(Component, Clone, Deref)]
pub struct Path(pub LyonPath);

#[derive(Component, Clone)]
pub struct DrawMode {
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
}

#[derive(Clone)]
pub struct Fill {
    pub color: SolidOrGradient,
    pub options: FillOptions,
    pub opacity: f32,
}

#[derive(Clone)]
pub struct Stroke {
    pub color: SolidOrGradient,
    pub options: StrokeOptions,
    pub opacity: f32,
}

#[derive(Clone)]
pub enum SolidOrGradient {
    Solid(Color),
    Gradient(Gradient),
}

#[derive(Clone)]
pub struct Gradient {
    start: Vec2,
    end: Vec2,
    ty: (),
}

#[derive(Bundle)]
pub struct ShapeBundle {
    pub path: Path,
    pub shape: Shape,
    pub draw_mode: DrawMode,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub mesh: Mesh2dHandle,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

impl ShapeBundle {
    pub fn new(path: LyonPath, draw_mode: DrawMode, transform: Transform) -> Self {
        ShapeBundle {
            path: Path(path),
            shape: Shape,
            draw_mode,
            transform,
            global_transform: GlobalTransform::default(),
            mesh: Mesh2dHandle::default(),
            visibility: Visibility::default(),
            computed_visibility: ComputedVisibility::default(),
        }
    }
}
