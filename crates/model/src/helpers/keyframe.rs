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
    pub start_value: T,
    #[serde(rename = "e")]
    pub end_value: T,
    #[serde(rename = "t", default)]
    pub start_frame: f32,
    #[serde(skip)]
    pub end_frame: f32,
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
    TOnly { t: f32 },
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
                    start_value: T::from(v.clone()),
                    end_value: T::from(v),
                    start_frame: 0.0,
                    end_frame: 0.0,
                    easing_in: None,
                    easing_out: None,
                }]
            }
            TolerantAnimatedHelper::AnimatedHelper(v) => {
                let mut result = vec![];
                for k in v {
                    match k {
                        LegacyTolerantKeyframe::KeyFrame(k) => {
                            result.push(k);
                        }
                        LegacyTolerantKeyframe::LegacyKeyFrame(k) => {
                            let LegacyKeyFrame {
                                start_value,
                                end_value,
                                start_frame,
                                end_frame,
                                easing_out,
                                easing_in,
                            } = k;
                            if let Some(prev) = result.last_mut() {
                                prev.end_frame = start_frame;
                            }
                            result.push(KeyFrame {
                                start_value,
                                end_value,
                                start_frame,
                                end_frame,
                                easing_in,
                                easing_out,
                            })
                        }
                        LegacyTolerantKeyframe::TOnly { t } => {
                            if let Some(prev) = result.last_mut() {
                                prev.end_frame = t;
                            }
                            break;
                        }
                    }
                }
                result
                    .into_iter()
                    .map(|keyframe| KeyFrame {
                        start_value: T::from(keyframe.start_value),
                        end_value: T::from(keyframe.end_value),
                        start_frame: keyframe.start_frame,
                        end_frame: keyframe.end_frame,
                        easing_in: keyframe.easing_in,
                        easing_out: keyframe.easing_out,
                    })
                    .collect()
            }
        }
    }
}
