use serde::{Deserialize, Serialize};

use crate::{Easing, FromTo, KeyFrame, Value};

#[derive(Deserialize)]
#[serde(transparent)]
pub(super) struct AnimatedHelper {
    data: TolerantAnimatedHelper,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum TolerantAnimatedHelper {
    Plain(Value),
    AnimatedHelper(Vec<LegacyTolerantKeyframe>),
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
struct LegacyKeyFrame<T> {
    #[serde(rename = "s")]
    pub value: T,
    #[serde(rename = "e")]
    pub end_value: T,
    #[serde(
        rename = "t",
        default,
        deserialize_with = "super::optional_u32_from_number"
    )]
    pub start_frame: Option<u32>,
    #[serde(rename = "o", default)]
    pub easing_out: Option<Easing>,
    #[serde(rename = "i", default)]
    pub easing_in: Option<Easing>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum LegacyTolerantKeyframe {
    KeyFrame(KeyFrame<Value>),
    LegacyKeyFrame(LegacyKeyFrame<Value>),
    TOnly { t: u32 },
}

impl<'a, T> From<&'a Vec<KeyFrame<T>>> for AnimatedHelper {
    fn from(_: &'a Vec<KeyFrame<T>>) -> Self {
        todo!()
    }
}

impl<T> From<AnimatedHelper> for Vec<KeyFrame<T>>
where
    T: FromTo<Value>,
{
    fn from(animated: AnimatedHelper) -> Self {
        match animated.data {
            TolerantAnimatedHelper::Plain(v) => {
                vec![KeyFrame {
                    value: T::from(v),
                    start_frame: None,
                    easing_in: None,
                    easing_out: None,
                }]
            }
            TolerantAnimatedHelper::AnimatedHelper(v) => v
                .into_iter()
                .filter_map(|k| match k {
                    LegacyTolerantKeyframe::KeyFrame(k) => Some(k),
                    LegacyTolerantKeyframe::TOnly { t } => None,
                    _ => None,
                })
                .map(|keyframe| KeyFrame {
                    value: T::from(keyframe.value),
                    start_frame: keyframe.start_frame,
                    easing_in: keyframe.easing_in,
                    easing_out: keyframe.easing_out,
                })
                .collect(),
        }
    }
}
