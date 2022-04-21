use std::fmt;

use super::*;
use serde::{
    de::{Error, SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};

pub fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Unsigned(other as u64),
            &"zero or one",
        )),
    }
}

pub fn int_from_bool<S>(b: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u8(if *b { 1 } else { 0 })
}

pub fn str_to_rgba<'de, D>(deserializer: D) -> Result<Rgba, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <&str>::deserialize(deserializer)?;
    Ok(s.parse().unwrap())
}

pub fn str_from_rgba<S>(b: &Rgba, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&b.to_string())
}

impl<'de> serde::Deserialize<'de> for LayerContent {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(d)?;

        #[derive(Serialize, Deserialize)]
        struct SolidColor {
            #[serde(
                rename = "sc",
                deserialize_with = "str_to_rgba",
                serialize_with = "str_from_rgba"
            )]
            color: Rgba,
            #[serde(rename = "sh")]
            height: f32,
            #[serde(rename = "sw")]
            width: f32,
        }

        Ok(match value.get("ty").and_then(Value::as_u64).unwrap() {
            0 => LayerContent::Precomposition(PreCompositionRef::deserialize(value).unwrap()),
            1 => {
                let color = SolidColor::deserialize(value).unwrap();
                LayerContent::SolidColor {
                    color: color.color,
                    height: color.height,
                    width: color.width,
                }
            }
            // 2 => LayerContent::Image(Type2::deserialize(value).unwrap()),
            3 => LayerContent::Empty,
            4 => {
                let shapes = value
                    .get("shapes")
                    .map(|v| Vec::<ShapeLayer>::deserialize(v))
                    .transpose()
                    .unwrap_or_default()
                    .unwrap_or_default();
                LayerContent::Shape(ShapeGroup { shapes })
            }
            // 5 => LayerContent::SolidColor(Type1::deserialize(value).unwrap()),
            // 6 => LayerContent::Image(Type2::deserialize(value).unwrap()),
            // 7 => LayerContent::Null(Type3::deserialize(value).unwrap()),
            type_ => panic!("unsupported type {:?}", type_),
        })
    }
}

impl Serialize for LayerContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        #[serde(untagged)]
        enum LayerContent_<'a> {
            // T1(&'a Type1),
            SolidColor { sc: String, sh: f32, sw: f32 },
            Shape { shapes: &'a Vec<ShapeLayer> },
        }

        #[derive(Serialize)]
        struct TypedLayerContent<'a> {
            #[serde(rename = "ty")]
            t: u64,
            #[serde(flatten)]
            content: LayerContent_<'a>,
        }

        let msg = match self {
            LayerContent::Shape(ShapeGroup { shapes }) => TypedLayerContent {
                t: 4,
                content: LayerContent_::Shape { shapes }
            },
            LayerContent::SolidColor { color, height, width } => TypedLayerContent {
                t: 1,
                content: LayerContent_::SolidColor { sc: color.to_string(), sh: *height, sw: *width }
            },
            _ => unimplemented!()
            // Message::T1(t) => TypedMessage {
            //     t: 1,
            //     msg: Message_::T1(t),
            // },
            // Message::T2(t) => TypedMessage {
            //     t: 2,
            //     msg: Message_::T2(t),
            // },
            // Message::T3(t) => TypedMessage {
            //     t: 3,
            //     msg: Message_::T3(t),
            // },
        };
        msg.serialize(serializer)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum AnimatedVec_ {
    Plain(Vec<f32>),
    Animated(Vec<KeyFrame<Vec<f32>>>),
}

pub fn vec2_from_array<'de, D>(deserializer: D) -> Result<Vec<KeyFrame<Vector2D<f32>>>, D::Error>
where
    D: Deserializer<'de>,
{
    let result = AnimatedVec_::deserialize(deserializer)?;
    match result {
        AnimatedVec_::Plain(v) => {
            debug_assert_eq!(v.len(), 2);
            Ok(vec![KeyFrame {
                value: euclid::vec2(v[0], v[1]),
                start_frame: None,
                easing_in: None,
                easing_out: None,
            }])
        }
        AnimatedVec_::Animated(v) => Ok(v
            .into_iter()
            .map(|keyframe| KeyFrame {
                value: euclid::vec2(keyframe.value[0], keyframe.value[1]),
                start_frame: keyframe.start_frame,
                easing_in: keyframe.easing_in,
                easing_out: keyframe.easing_out,
            })
            .collect()),
    }
}

pub fn array_from_vec2<S>(
    b: &Vec<KeyFrame<Vector2D<f32>>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(b.len() * 2))?;
    for item in b {
        seq.serialize_element(&item.value.x)?;
        seq.serialize_element(&item.value.y)?;
    }
    seq.end()
}

pub fn rgb_from_array<'de, D>(deserializer: D) -> Result<Vec<KeyFrame<Rgb>>, D::Error>
where
    D: Deserializer<'de>,
{
    let result = AnimatedVec_::deserialize(deserializer)?;
    match result {
        AnimatedVec_::Plain(v) => {
            debug_assert_eq!(v.len(), 3);
            Ok(vec![KeyFrame {
                value: Rgb::new_f32(v[0], v[1], v[2]),
                start_frame: None,
                easing_in: None,
                easing_out: None,
            }])
        }
        AnimatedVec_::Animated(v) => Ok(v
            .into_iter()
            .map(|keyframe| KeyFrame {
                value: Rgb::new_f32(keyframe.value[0], keyframe.value[1], keyframe.value[2]),
                start_frame: keyframe.start_frame,
                easing_in: keyframe.easing_in,
                easing_out: keyframe.easing_out,
            })
            .collect()),
    }
}

pub fn array_from_rgb<S>(b: &Vec<KeyFrame<Rgb>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(b.len() * 3))?;
    for item in b {
        seq.serialize_element(&(item.value.r as f32 / 255.0))?;
        seq.serialize_element(&(item.value.g as f32 / 255.0))?;
        seq.serialize_element(&(item.value.b as f32 / 255.0))?;
    }
    seq.end()
}

pub fn f32_from_array_or_number<'de, D>(deserializer: D) -> Result<Vec<f32>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(ArrayOrNumberVisitor)
}

pub fn array_or_number_from_f32<S>(b: &Vec<f32>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if b.len() == 1 {
        serializer.serialize_f32(b[0])
    } else {
        let mut seq = serializer.serialize_seq(Some(b.len()))?;
        for item in b {
            seq.serialize_element(item)?;
        }
        seq.end()
    }
}

struct ArrayOrNumberVisitor;

impl<'de> Visitor<'de> for ArrayOrNumberVisitor {
    type Value = Vec<f32>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("f32 / [f32]")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
        let mut result = vec![];
        loop {
            if let Some(e) = access.next_element::<f32>()? {
                result.push(e);
            } else {
                break;
            }
        }
        Ok(result)
    }

    fn visit_f32<E: Error>(self, v: f32) -> Result<Self::Value, E> {
        Ok(vec![v])
    }

    fn visit_i64<E: Error>(self, v: i64) -> Result<Self::Value, E> {
        Ok(vec![v as f32])
    }

    fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
        Ok(vec![v as f32])
    }
}

impl<'de> Deserialize<'de> for AnimatedColorList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

impl Serialize for AnimatedColorList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }
}

pub fn default_vec2_100() -> AnimatedVec2 {
    AnimatedVec2 {
        animated: false,
        keyframes: vec![KeyFrame::from_value(Vector2D::new(100.0, 100.0))],
    }
}

pub fn default_number_100() -> AnimatedNumber {
    AnimatedNumber {
        animated: false,
        keyframes: vec![100.0],
    }
}

pub fn u32_from_number<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(NumberVistor)
}

struct NumberVistor;

impl<'de> Visitor<'de> for NumberVistor {
    type Value = u32;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("u32 / f32")
    }

    fn visit_f32<E: Error>(self, v: f32) -> Result<Self::Value, E> {
        Ok(v.round() as u32)
    }

    fn visit_f64<E: Error>(self, v: f64) -> Result<Self::Value, E> {
        Ok(v.round() as u32)
    }

    fn visit_i64<E: Error>(self, v: i64) -> Result<Self::Value, E> {
        Ok(v as u32)
    }

    fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
        Ok(v as u32)
    }
}
