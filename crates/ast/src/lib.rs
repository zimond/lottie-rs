use std::str::FromStr;

pub use euclid::default::Rect;
pub use euclid::rect;
use serde::{Deserialize, Serialize};
pub use serde_json::Error;
pub type Vector2D = euclid::default::Vector2D<f32>;

mod helpers;
use helpers::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Model {
    #[serde(rename = "nm")]
    pub name: Option<String>,
    #[serde(rename = "v")]
    version: String,
    #[serde(rename = "ip", deserialize_with = "u32_from_number")]
    pub start_frame: u32,
    #[serde(rename = "op", deserialize_with = "u32_from_number")]
    pub end_frame: u32,
    #[serde(rename = "fr")]
    pub frame_rate: u32,
    #[serde(rename = "w")]
    pub width: u32,
    #[serde(rename = "h")]
    pub height: u32,
    pub layers: Vec<Layer>,
    #[serde(default)]
    pub assets: Vec<Precomposition>,
}

impl Model {
    pub fn from_reader<R: std::io::Read>(r: R) -> Result<Self, serde_json::Error> {
        serde_json::from_reader(r)
    }

    pub fn duration(&self) -> f32 {
        (self.end_frame - self.start_frame) as f32 / self.frame_rate as f32
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Layer {
    #[serde(
        deserialize_with = "bool_from_int",
        serialize_with = "int_from_bool",
        rename = "ddd",
        default
    )]
    is_3d: bool,
    #[serde(rename = "hd", default)]
    pub hidden: bool,
    #[serde(rename = "ind", default)]
    pub index: Option<u32>,
    #[serde(rename = "parent", default)]
    pub parent_index: Option<u32>,
    #[serde(skip)]
    pub id: u32,
    #[serde(rename = "ip", deserialize_with = "u32_from_number")]
    pub start_frame: u32,
    #[serde(rename = "op", deserialize_with = "u32_from_number")]
    pub end_frame: u32,
    #[serde(rename = "st", deserialize_with = "u32_from_number")]
    pub start_time: u32,
    #[serde(rename = "nm")]
    name: Option<String>,
    #[serde(rename = "ks", default)]
    pub transform: Option<Transform>,
    #[serde(flatten)]
    pub content: LayerContent,
}

#[derive(Debug, Clone)]
pub enum LayerContent {
    Precomposition(PreCompositionRef),
    SolidColor {
        color: Rgba,
        height: f32,
        width: f32,
    },
    Image,
    Empty,
    Shape(ShapeGroup),
    Text,
    Audio,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PreCompositionRef {
    #[serde(rename = "refId")]
    pub ref_id: String,
    #[serde(rename = "w")]
    width: u32,
    #[serde(rename = "h")]
    height: u32,
    #[serde(rename = "tm")]
    time_remapping: Option<Animated<f32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transform {
    #[serde(rename = "a", default)]
    pub anchor: Animated<Vector2D>,
    #[serde(rename = "p", default)]
    pub position: Animated<Vector2D>,
    #[serde(rename = "s", default = "default_vec2_100")]
    pub scale: Animated<Vector2D>,
    #[serde(rename = "r", default)]
    pub rotation: Animated<f32>,
    #[serde(rename = "o", default = "default_number_100")]
    opacity: Animated<f32>,
    #[serde(rename = "sk", default)]
    skew: Option<Animated<Vector2D>>,
    #[serde(rename = "sa", default)]
    skew_axis: Option<Animated<Vector2D>>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            anchor: Default::default(),
            position: Default::default(),
            scale: default_vec2_100(),
            rotation: Default::default(),
            opacity: default_number_100(),
            skew: Default::default(),
            skew_axis: Default::default(),
        }
    }
}

impl Transform {
    pub fn is_identity(&self) -> bool {
        false
        // TODO:
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepeaterTransform {
    #[serde(rename = "a", default)]
    anchor: Animated<Vector2D>,
    #[serde(rename = "p")]
    position: Animated<Vector2D>,
    #[serde(rename = "s")]
    scale: Animated<Vector2D>,
    #[serde(rename = "r")]
    rotation: Animated<f32>,
    #[serde(rename = "so")]
    start_opacity: Animated<f32>,
    #[serde(rename = "eo")]
    end_opacity: Animated<f32>,
    #[serde(rename = "sk", default)]
    skew: Option<Animated<Vector2D>>,
    #[serde(rename = "sa", default)]
    skew_axis: Option<Animated<Vector2D>>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct KeyFrame<T> {
    #[serde(rename = "s")]
    pub value: T,
    #[serde(rename = "t", default, deserialize_with = "optional_u32_from_number")]
    pub start_frame: Option<u32>,
    #[serde(rename = "o", default)]
    pub easing_out: Option<Easing>,
    #[serde(rename = "i", default)]
    pub easing_in: Option<Easing>,
}

impl<T> KeyFrame<T> {
    pub fn from_value(value: T) -> Self {
        KeyFrame {
            value,
            start_frame: None,
            easing_out: None,
            easing_in: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Easing {
    pub x: Vec<f32>,
    pub y: Vec<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Animated<T> {
    #[serde(
        deserialize_with = "bool_from_int",
        serialize_with = "int_from_bool",
        rename = "a",
        default
    )]
    pub animated: bool,
    #[serde(
        deserialize_with = "keyframes_from_array",
        serialize_with = "array_from_keyframes",
        bound = "T: FromTo<helpers::Value>",
        rename = "k"
    )]
    pub keyframes: Vec<KeyFrame<T>>,
}

impl<T> Default for Animated<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            animated: false,
            keyframes: vec![KeyFrame::default()],
        }
    }
}

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct Animated {
//     #[serde(
//         deserialize_with = "bool_from_int",
//         serialize_with = "int_from_bool",
//         rename = "a"
//     )]
//     pub animated: bool,
//     #[serde(
//         deserialize_with = "f32_from_array_or_number",
//         serialize_with = "array_or_number_from_f32",
//         rename = "k"
//     )]
//     pub keyframes: Vec<KeyFrame<f32>>,
// }

// impl Default for Animated {
//     fn default() -> Self {
//         Self {
//             animated: false,
//             keyframes: vec![KeyFrame::from_value(0.0)],
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct Rgba {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl FromStr for Rgba {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl ToString for Rgba {
    fn to_string(&self) -> String {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn new_f32(r: f32, g: f32, b: f32) -> Rgb {
        Rgb {
            r: (r * 255.0) as u8,
            g: (g * 255.0) as u8,
            b: (b * 255.0) as u8,
        }
    }

    pub fn new_u8(r: u8, g: u8, b: u8) -> Rgb {
        Rgb { r, g, b }
    }
}

#[derive(Debug, Clone)]
pub struct AnimatedColorList {
    animated: bool,
    colors: Vec<Rgba>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShapeLayer {
    #[serde(rename = "nm", default)]
    name: Option<String>,
    #[serde(rename = "hd", default)]
    pub hidden: bool,
    #[serde(skip)]
    pub id: u32,
    #[serde(flatten)]
    pub shape: Shape,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "ty")]
pub enum Shape {
    #[serde(rename = "rc")]
    Rectangle(Rectangle),
    #[serde(rename = "el")]
    Ellipse(Ellipse),
    #[serde(rename = "sr")]
    PolyStar(PolyStar),
    #[serde(rename = "sh")]
    Path {
        #[serde(rename = "ks")]
        d: Animated<Vec<Bezier>>,
    },
    #[serde(rename = "fl")]
    Fill(Fill),
    #[serde(rename = "st")]
    Stroke(Stroke),
    #[serde(rename = "gf")]
    GradientFill {
        #[serde(rename = "o")]
        opacity: Animated<f32>,
        #[serde(rename = "r")]
        fill_rule: FillRule,
        #[serde(rename = "s")]
        start: Animated<Vector2D>,
        #[serde(rename = "e")]
        end: Animated<Vector2D>,
        #[serde(rename = "t")]
        gradient_ty: GradientType,
        #[serde(rename = "g")]
        colors: AnimatedColorList,
    },
    #[serde(rename = "gs")]
    GradientStroke {
        #[serde(rename = "lc")]
        line_cap: LineCap,
        #[serde(rename = "lj")]
        line_join: LineJoin,
        #[serde(rename = "ml")]
        miter_limit: f32,
        #[serde(rename = "o")]
        opacity: Animated<f32>,
        #[serde(rename = "w")]
        width: Animated<f32>,
        #[serde(rename = "d", default)]
        dashes: Vec<StrokeDash>,
        #[serde(rename = "s")]
        start: Animated<Vector2D>,
        #[serde(rename = "e")]
        end: Animated<Vector2D>,
        #[serde(rename = "t")]
        gradient_ty: GradientType,
        #[serde(rename = "g")]
        colors: AnimatedColorList,
    },
    #[serde(rename = "gr")]
    Group {
        // TODO: add np property
        #[serde(rename = "it")]
        shapes: Vec<ShapeLayer>,
    },
    #[serde(rename = "tr")]
    Transform(Transform),
    #[serde(rename = "rp")]
    Repeater {
        #[serde(rename = "c")]
        copies: Animated<f32>,
        #[serde(rename = "o")]
        offset: Animated<f32>,
        #[serde(rename = "m")]
        composite: Composite,
        #[serde(rename = "tr")]
        transform: RepeaterTransform,
    },
    #[serde(rename = "tm")]
    Trim {
        #[serde(rename = "s")]
        start: Animated<f32>,
        #[serde(rename = "e")]
        end: Animated<f32>,
        #[serde(rename = "o")]
        offset: Animated<f32>,
        #[serde(rename = "m")]
        multiple_shape: TrimMultipleShape,
    },
    #[serde(rename = "rd")]
    RoundedCorners {
        #[serde(rename = "r")]
        radius: Animated<f32>,
    },
    #[serde(rename = "pb")]
    PuckerBloat {
        #[serde(rename = "a")]
        amount: Animated<f32>,
    },
    #[serde(rename = "tw")]
    Twist {
        #[serde(rename = "a")]
        angle: Animated<f32>,
        #[serde(rename = "c")]
        center: Animated<Vector2D>,
    },
    #[serde(rename = "mm")]
    Merge {
        #[serde(rename = "mm")]
        mode: MergeMode,
    },
    #[serde(rename = "op")]
    OffsetPath {
        #[serde(rename = "a")]
        amount: Animated<f32>,
        #[serde(rename = "lj")]
        line_join: LineJoin,
        #[serde(rename = "ml")]
        miter_limit: f32,
    },
    #[serde(rename = "zz")]
    ZigZag {
        #[serde(rename = "r")]
        radius: Animated<f32>,
        #[serde(rename = "s")]
        distance: Animated<f32>,
        #[serde(rename = "pt")]
        ridges: Animated<f32>,
    },
}

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum PolyStarType {
    Star = 1,
    Polygon = 2,
}

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum FillRule {
    NonZero = 1,
    EvenOdd = 2,
}

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum LineCap {
    Butt = 1,
    Round = 2,
    Square = 3,
}

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum LineJoin {
    Miter = 1,
    Round = 2,
    Bevel = 3,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StrokeDash {
    #[serde(rename = "v")]
    length: Animated<f32>,
    #[serde(rename = "n")]
    ty: StrokeDashType,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum StrokeDashType {
    #[serde(rename = "d")]
    Dash,
    #[serde(rename = "g")]
    Gap,
    #[serde(rename = "o")]
    Offset,
}

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum GradientType {
    Linear = 1,
    Radial = 2,
}

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum Composite {
    Above = 1,
    Below = 2,
}

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum TrimMultipleShape {
    Individually = 1,
    Simultaneously = 2,
}

#[derive(
    serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy, PartialEq,
)]
#[repr(u8)]
pub enum ShapeDirection {
    Clockwise = 1,
    CounterClockwise = 2,
}

impl Default for ShapeDirection {
    fn default() -> Self {
        ShapeDirection::Clockwise
    }
}

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum MergeMode {
    #[serde(other)]
    Unsupported = 1,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fill {
    #[serde(rename = "o")]
    pub opacity: Animated<f32>,
    #[serde(rename = "c")]
    pub color: Animated<Rgb>,
    #[serde(rename = "r")]
    fill_rule: FillRule,
}

impl Fill {
    pub fn transparent() -> Fill {
        Fill {
            opacity: Animated {
                animated: false,
                keyframes: vec![KeyFrame::from_value(0.0)],
            },
            color: Animated {
                animated: false,
                keyframes: vec![KeyFrame::from_value(Rgb::new_u8(0, 0, 0))],
            },
            fill_rule: FillRule::NonZero,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stroke {
    #[serde(rename = "lc")]
    pub line_cap: LineCap,
    #[serde(rename = "lj")]
    pub line_join: LineJoin,
    #[serde(rename = "ml")]
    miter_limit: f32,
    #[serde(rename = "o")]
    pub opacity: Animated<f32>,
    #[serde(rename = "w")]
    pub width: Animated<f32>,
    #[serde(rename = "d", default)]
    dashes: Vec<StrokeDash>,
    #[serde(rename = "c")]
    pub color: Animated<Rgb>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rectangle {
    #[serde(rename = "d", default)]
    pub direction: ShapeDirection,
    #[serde(rename = "p")]
    pub position: Animated<Vector2D>,
    #[serde(rename = "s")]
    pub size: Animated<Vector2D>,
    #[serde(rename = "r")]
    pub radius: Animated<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ellipse {
    #[serde(rename = "d", default)]
    pub direction: ShapeDirection,
    #[serde(rename = "p")]
    pub position: Animated<Vector2D>,
    #[serde(rename = "s")]
    pub size: Animated<Vector2D>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PolyStar {
    #[serde(rename = "d", default)]
    pub direction: ShapeDirection,
    #[serde(rename = "p")]
    pub position: Animated<Vector2D>,
    #[serde(rename = "or")]
    pub outer_radius: Animated<f32>,
    #[serde(rename = "os")]
    pub outer_roundness: Animated<f32>,
    #[serde(rename = "ir")]
    pub inner_radius: Animated<f32>,
    #[serde(rename = "is")]
    pub inner_roundness: Animated<f32>,
    #[serde(rename = "r")]
    pub rotation: Animated<f32>,
    #[serde(rename = "pt")]
    pub points: Animated<f32>,
    #[serde(rename = "sy")]
    pub star_type: PolyStarType,
}

pub enum Assets {
    Image,
    Sound,
    Precomposition(Precomposition),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Precomposition {
    pub id: String,
    pub layers: Vec<Layer>,
    #[serde(rename = "nm")]
    name: Option<String>,
    #[serde(rename = "fr")]
    pub frame_rate: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShapeGroup {
    pub shapes: Vec<ShapeLayer>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bezier {
    #[serde(rename = "c")]
    pub closed: bool,
    #[serde(
        rename = "v",
        deserialize_with = "vec_from_array",
        serialize_with = "array_from_vec"
    )]
    pub verticies: Vec<Vector2D>,
    #[serde(
        rename = "i",
        deserialize_with = "vec_from_array",
        serialize_with = "array_from_vec"
    )]
    pub in_tangent: Vec<Vector2D>,
    #[serde(
        rename = "o",
        deserialize_with = "vec_from_array",
        serialize_with = "array_from_vec"
    )]
    pub out_tangent: Vec<Vector2D>,
}
