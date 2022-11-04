use serde::Deserialize;

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
    AnimatedHelper(Vec<LegacyTolerantKeyFrame>),
}

fn default_none<T>() -> Option<T> {
    None
}

#[derive(Deserialize, Default, Debug, Clone)]
struct LegacyKeyFrame<T> {
    #[serde(rename = "s")]
    start_value: T,
    #[serde(rename = "e", default = "default_none")]
    end_value: Option<T>,
    #[serde(rename = "t", default)]
    start_frame: f32,
    #[serde(skip)]
    end_frame: f32,
    #[serde(rename = "o", default)]
    easing_out: Option<Easing>,
    #[serde(rename = "i", default)]
    easing_in: Option<Easing>,
    #[serde(rename = "h", default, deserialize_with = "super::bool_from_int")]
    hold: bool,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum LegacyTolerantKeyFrame {
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
                let mut result: Vec<LegacyKeyFrame<Value>> = vec![];
                // Sometimes keyframes especially from TextData do not have an ending frame, so
                // we double check here to avoid removing them.
                let mut has_t_only_frame = false;
                for k in v {
                    match k {
                        LegacyTolerantKeyFrame::LegacyKeyFrame(mut k) => {
                            if let Some(prev) = result.last_mut() {
                                prev.end_frame = k.start_frame;
                            }
                            if k.hold {
                                k.end_value = Some(k.start_value.clone());
                            }
                            result.push(k)
                        }
                        LegacyTolerantKeyFrame::TOnly { t } => {
                            if let Some(prev) = result.last_mut() {
                                prev.end_frame = t;
                            }
                            has_t_only_frame = true;
                            break;
                        }
                    }
                }
                if result.len() > 1 {
                    for i in 0..(result.len() - 1) {
                        if result[i].end_value.is_none() {
                            result[i].end_value = Some(result[i + 1].start_value.clone());
                        }
                    }
                }
                if has_t_only_frame
                    && result
                        .last()
                        .map(|keyframe| keyframe.end_value.is_none())
                        .unwrap_or(false)
                {
                    result.pop();
                }
                result
                    .into_iter()
                    .map(|keyframe| KeyFrame {
                        end_value: T::from(
                            keyframe
                                .end_value
                                .unwrap_or_else(|| keyframe.start_value.clone()),
                        ),
                        start_value: T::from(keyframe.start_value),
                        start_frame: keyframe.start_frame,
                        end_frame: keyframe.end_frame.max(keyframe.start_frame),
                        easing_in: keyframe.easing_in,
                        easing_out: keyframe.easing_out,
                    })
                    .collect()
            }
        }
    }
}
