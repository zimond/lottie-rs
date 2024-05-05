use bevy::ecs::reflect::ReflectComponent;
use bevy::math::Vec2;
use bevy::prelude::{Bundle, Color, Component, Deref, GlobalTransform, Transform, Visibility};
use bevy::reflect::Reflect;
use bevy::render::view::{InheritedVisibility, ViewVisibility};
use bevy::sprite::Mesh2dHandle;
use lyon::path::Path as LyonPath;
use lyon::tessellation::{FillOptions, StrokeOptions};

/// Marker shape
#[derive(Component, Clone, Copy)]
pub struct Shape;

#[derive(Component, Clone, Deref)]
pub struct Path(pub LyonPath);

#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub struct DrawMode {
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
}

#[derive(Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Fill {
    pub color: SolidOrGradient,
    #[reflect(ignore)]
    pub options: FillOptions,
    pub opacity: f32,
}

#[derive(Clone, Reflect, Component)]
#[reflect(Component)]
pub struct Stroke {
    pub color: SolidOrGradient,
    #[reflect(ignore)]
    pub options: StrokeOptions,
    pub opacity: f32,
}

#[derive(Clone, Reflect, PartialEq)]
#[reflect(PartialEq)]
pub enum SolidOrGradient {
    Solid(Color),
    Gradient(Gradient),
}

#[derive(Clone, PartialEq, Reflect, Component)]
#[reflect(Component)]
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
    pub view_visibility: ViewVisibility,
    pub inherited_visibility: InheritedVisibility,
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
            view_visibility: ViewVisibility::default(),
            inherited_visibility: InheritedVisibility::default(),
        }
    }
}
