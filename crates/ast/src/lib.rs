use std::str::FromStr;

pub use euclid::default::{Rect, Vector2D};
use serde::{Deserialize, Serialize};
pub use serde_json::Error;
use serde_json::Value;

mod helpers;
use helpers::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LottieModel {
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

impl LottieModel {
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
    #[serde(rename = "nm")]
    name: Option<String>,
    #[serde(rename = "ks", default)]
    transform: Option<Transform>,
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
    Shape {
        shapes: Vec<ShapeLayer>,
    },
    Text,
    Audio,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PreCompositionRef {
    #[serde(rename = "refId")]
    ref_id: String,
    #[serde(rename = "w")]
    width: u32,
    #[serde(rename = "h")]
    height: u32,
    #[serde(rename = "tm")]
    time_remapping: Option<AnimatedNumber>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transform {
    #[serde(rename = "a", default)]
    anchor: AnimatedVec2,
    #[serde(rename = "p", default)]
    pub position: AnimatedVec2,
    #[serde(rename = "s", default = "default_vec2_100")]
    pub scale: AnimatedVec2,
    #[serde(rename = "r", default)]
    rotation: AnimatedNumber,
    #[serde(rename = "o", default = "default_number_100")]
    opacity: AnimatedNumber,
    #[serde(rename = "sk", default)]
    skew: Option<AnimatedVec2>,
    #[serde(rename = "sa", default)]
    skew_axis: Option<AnimatedVec2>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepeaterTransform {
    #[serde(rename = "a")]
    anchor: AnimatedVec2,
    #[serde(rename = "p")]
    position: AnimatedVec2,
    #[serde(rename = "s")]
    scale: AnimatedVec2,
    #[serde(rename = "r")]
    rotation: AnimatedNumber,
    #[serde(rename = "so")]
    start_opacity: AnimatedNumber,
    #[serde(rename = "eo")]
    end_opacity: AnimatedNumber,
    #[serde(rename = "sk", default)]
    skew: Option<AnimatedVec2>,
    #[serde(rename = "sa", default)]
    skew_axis: Option<AnimatedVec2>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct KeyFrame<T> {
    #[serde(rename = "s")]
    pub value: T,
    #[serde(rename = "t", default)]
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
pub struct AnimatedVec2 {
    #[serde(
        deserialize_with = "bool_from_int",
        serialize_with = "int_from_bool",
        rename = "a"
    )]
    pub animated: bool,
    #[serde(
        deserialize_with = "vec2_from_array",
        serialize_with = "array_from_vec2",
        rename = "k"
    )]
    pub keyframes: Vec<KeyFrame<Vector2D<f32>>>,
}

impl Default for AnimatedVec2 {
    fn default() -> Self {
        Self {
            animated: false,
            keyframes: vec![KeyFrame::default()],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimatedNumber {
    #[serde(
        deserialize_with = "bool_from_int",
        serialize_with = "int_from_bool",
        rename = "a"
    )]
    pub animated: bool,
    #[serde(
        deserialize_with = "f32_from_array_or_number",
        serialize_with = "array_or_number_from_f32",
        rename = "k"
    )]
    pub keyframes: Vec<f32>,
}

impl Default for AnimatedNumber {
    fn default() -> Self {
        Self {
            animated: false,
            keyframes: vec![0.0],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimatedBezier {
    #[serde(
        deserialize_with = "bool_from_int",
        serialize_with = "int_from_bool",
        rename = "a"
    )]
    animated: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimatedColor {
    #[serde(
        deserialize_with = "bool_from_int",
        serialize_with = "int_from_bool",
        rename = "a"
    )]
    animated: bool,
    #[serde(
        deserialize_with = "rgb_from_array",
        serialize_with = "array_from_rgb",
        rename = "k"
    )]
    keyframes: Vec<KeyFrame<Rgb>>,
}

impl AnimatedColor {
    pub fn initial_color(&self) -> Rgb {
        self.keyframes[0].value.clone()
    }
}

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
    PolyStar {
        #[serde(rename = "p")]
        position: AnimatedVec2,
        #[serde(rename = "or")]
        outer_radius: AnimatedNumber,
        #[serde(rename = "os")]
        outer_roundness: AnimatedNumber,
        #[serde(rename = "r")]
        rotation: AnimatedNumber,
        #[serde(rename = "pt")]
        points: AnimatedNumber,
        #[serde(rename = "sy")]
        star_type: PolyStarType,
    },
    #[serde(rename = "sh")]
    Path {
        #[serde(rename = "ks")]
        d: AnimatedBezier,
    },
    #[serde(rename = "fl")]
    Fill(Fill),
    #[serde(rename = "st")]
    Stroke(Stroke),
    #[serde(rename = "gf")]
    GradientFill {
        #[serde(rename = "o")]
        opacity: AnimatedNumber,
        #[serde(rename = "r")]
        fill_rule: FillRule,
        #[serde(rename = "s")]
        start: AnimatedVec2,
        #[serde(rename = "e")]
        end: AnimatedVec2,
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
        opacity: AnimatedNumber,
        #[serde(rename = "w")]
        width: AnimatedNumber,
        #[serde(rename = "d")]
        dashes: Vec<StrokeDash>,
        #[serde(rename = "s")]
        start: AnimatedVec2,
        #[serde(rename = "e")]
        end: AnimatedVec2,
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
        copies: AnimatedNumber,
        #[serde(rename = "o")]
        offset: AnimatedNumber,
        #[serde(rename = "m")]
        composite: Composite,
        #[serde(rename = "tr")]
        transform: RepeaterTransform,
    },
    #[serde(rename = "tm")]
    Trim {
        #[serde(rename = "s")]
        start: AnimatedNumber,
        #[serde(rename = "e")]
        end: AnimatedNumber,
        #[serde(rename = "o")]
        offset: AnimatedNumber,
        #[serde(rename = "m")]
        multiple_shape: TrimMultipleShape,
    },
    #[serde(rename = "rd")]
    RoundedCorners {
        #[serde(rename = "r")]
        radius: AnimatedNumber,
    },
    #[serde(rename = "pb")]
    PuckerBloat {
        #[serde(rename = "a")]
        amount: AnimatedNumber,
    },
    #[serde(rename = "tw")]
    Twist {
        #[serde(rename = "a")]
        angle: AnimatedNumber,
        #[serde(rename = "c")]
        center: AnimatedVec2,
    },
    #[serde(rename = "mm")]
    Merge {
        #[serde(rename = "mm")]
        mode: MergeMode,
    },
    #[serde(rename = "op")]
    OffsetPath {
        #[serde(rename = "a")]
        amount: AnimatedNumber,
        #[serde(rename = "lj")]
        line_join: LineJoin,
        #[serde(rename = "ml")]
        miter_limit: f32,
    },
    #[serde(rename = "zz")]
    ZigZag {
        #[serde(rename = "r")]
        radius: AnimatedNumber,
        #[serde(rename = "s")]
        distance: AnimatedNumber,
        #[serde(rename = "pt")]
        ridges: AnimatedNumber,
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
    length: AnimatedNumber,
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

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum ShapeDirection {
    Clockwise = 1,
    CounterClockwise = 2,
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
    pub opacity: AnimatedNumber,
    #[serde(rename = "c")]
    pub color: AnimatedColor,
    #[serde(rename = "r")]
    fill_rule: FillRule,
}

impl Fill {
    pub fn transparent() -> Fill {
        Fill {
            opacity: AnimatedNumber {
                animated: false,
                keyframes: vec![0.0],
            },
            color: AnimatedColor {
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
    line_cap: LineCap,
    #[serde(rename = "lj")]
    line_join: LineJoin,
    #[serde(rename = "ml")]
    miter_limit: f32,
    #[serde(rename = "o")]
    opacity: AnimatedNumber,
    #[serde(rename = "w")]
    width: AnimatedNumber,
    #[serde(rename = "d")]
    dashes: Vec<StrokeDash>,
    #[serde(rename = "c")]
    color: AnimatedColor,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rectangle {
    #[serde(rename = "p")]
    pub position: AnimatedVec2,
    #[serde(rename = "s")]
    pub size: AnimatedVec2,
    #[serde(rename = "r")]
    pub radius: AnimatedNumber,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ellipse {
    #[serde(rename = "p")]
    pub position: AnimatedVec2,
    #[serde(rename = "s")]
    pub size: AnimatedVec2,
}

impl LottieModel {
    pub fn from_reader<R: std::io::Read>(r: R) -> Result<Self, serde_json::Error> {
        serde_json::from_reader(r)
    }
}

pub enum Assets {
    Image,
    Sound,
    Precomposition(Precomposition),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Precomposition {
    id: String,
    layers: Vec<Layer>,
    #[serde(rename = "nm")]
    name: Option<String>,
    #[serde(rename = "fr")]
    frame_rate: Option<u32>,
}
