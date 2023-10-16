use std::collections::VecDeque;

use flo_curves::bezier::{curve_intersects_line, Curve};
use flo_curves::{BezierCurve, BezierCurveFactory, Coord2};
use serde::{Deserialize, Serialize};

use crate::Lerp;

use super::helpers::{self, *};

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

impl<T: Clone + Lerp<Target = T>> Animated<T> {
    pub fn from_value(value: T) -> Self {
        Animated {
            animated: false,
            keyframes: vec![KeyFrame {
                start_value: value.clone(),
                end_value: value,
                start_frame: 0.0,
                end_frame: 0.0,
                easing_out: None,
                easing_in: None,
            }],
        }
    }

    pub fn initial_value(&self) -> T {
        self.keyframes[0].start_value.clone()
    }

    pub fn value(&self, frame: f32) -> T {
        if !self.is_animated() {
            return self.initial_value();
        }
        let len = self.keyframes.len() - 1;
        if let Some(keyframe) = self
            .keyframes
            .iter()
            .find(|keyframe| frame >= keyframe.start_frame && frame < keyframe.end_frame)
        {
            let frames = keyframe.end_frame - keyframe.start_frame;
            let x = (frame - keyframe.start_frame) / frames;
            keyframe.value(x)
        } else if frame >= self.keyframes[len].end_frame {
            self.keyframes[len].end_value.clone()
        } else {
            self.keyframes[0].start_value.clone()
        }
    }

    pub fn is_animated(&self) -> bool {
        self.keyframes.len() > 1 || self.keyframes[0].easing_in.is_some()
    }

    pub fn align_to_sorted_frames(&mut self, mut frames: impl Iterator<Item = f32>) {
        let mut keyframes = vec![];
        let mut original_keyframes = self
            .keyframes
            .split_off(0)
            .into_iter()
            .collect::<VecDeque<_>>();
        while let Some(frame) = frames.next() {
            while let Some(keyframe) = original_keyframes.pop_front() {
                if keyframe.end_frame <= frame {
                    keyframes.push(keyframe);
                } else if keyframe.start_frame >= frame {
                    break;
                } else {
                    let (a, b) = keyframe.split(frame);
                    if let Some(a) = a {
                        keyframes.push(a);
                    }
                    if let Some(b) = b {
                        original_keyframes.push_front(b);
                    }
                }
            }
        }
        self.keyframes = keyframes;
    }

    pub fn keyframes(&self) -> impl Iterator<Item = f32> + '_ {
        self.keyframes
            .iter()
            .flat_map(|k| [k.start_frame, k.end_frame].into_iter())
    }
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

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct KeyFrame<T> {
    #[serde(rename = "s")]
    pub start_value: T,
    #[serde(skip)]
    pub end_value: T,
    #[serde(rename = "t", default)]
    pub start_frame: f32,
    // TODO: could end_frame & next start_frame create a gap?
    #[serde(skip)]
    pub end_frame: f32,
    #[serde(rename = "o", default)]
    pub easing_out: Option<Easing>,
    #[serde(rename = "i", default)]
    pub easing_in: Option<Easing>,
}

impl<T: Clone> KeyFrame<T> {
    pub fn from_value(value: T) -> Self {
        KeyFrame {
            start_value: value.clone(),
            end_value: value,
            start_frame: 0.0,
            end_frame: 0.0,
            easing_out: None,
            easing_in: None,
        }
    }

    pub fn alter_value<U>(&self, start: U, end: U) -> KeyFrame<U> {
        KeyFrame {
            start_value: start,
            end_value: end,
            start_frame: self.start_frame,
            end_frame: self.end_frame,
            easing_out: self.easing_out.clone(),
            easing_in: self.easing_in.clone(),
        }
    }
}

impl<T: Clone + Lerp<Target = T>> KeyFrame<T> {
    pub fn value(&self, t: f32) -> T {
        let ease_out = self.easing_out.clone().unwrap_or_else(|| Easing {
            x: vec![0.0],
            y: vec![0.0],
        });
        let ease_in = self.easing_in.clone().unwrap_or_else(|| Easing {
            x: vec![1.0],
            y: vec![1.0],
        });
        debug_assert!(t <= 1.0 && t >= 0.0);
        let curve = Curve::from_points(
            Coord2(0.0, 0.0),
            (
                Coord2(ease_out.x[0] as f64, ease_out.y[0] as f64),
                Coord2(ease_in.x[0] as f64, ease_in.y[0] as f64),
            ),
            Coord2(1.0, 1.0),
        );
        let intersection =
            curve_intersects_line(&curve, &(Coord2(t as f64, 0.0), Coord2(t as f64, 1.0)));
        let ratio = if intersection.is_empty() {
            t
        } else {
            intersection[0].2 .1 as f32
        };
        self.end_value.lerp(&self.start_value, ratio)
    }

    pub fn split(&self, frame: f32) -> (Option<Self>, Option<Self>) {
        if frame <= self.start_frame {
            return (None, Some(self.clone()));
        } else if frame >= self.end_frame {
            return (Some(self.clone()), None);
        }

        let ease_out = self.easing_out.clone().unwrap_or_else(|| Easing {
            x: vec![0.0],
            y: vec![0.0],
        });
        let ease_in = self.easing_in.clone().unwrap_or_else(|| Easing {
            x: vec![1.0],
            y: vec![1.0],
        });
        let frames = self.end_frame - self.start_frame;
        let x = (frame - self.start_frame) / frames;
        let curve = Curve::from_points(
            Coord2(0.0, 0.0),
            (
                Coord2(ease_out.x[0] as f64, ease_out.y[0] as f64),
                Coord2(ease_in.x[0] as f64, ease_in.y[0] as f64),
            ),
            Coord2(1.0, 1.0),
        );
        let intersection =
            curve_intersects_line(&curve, &(Coord2(x as f64, 0.0), Coord2(x as f64, 1.0)));
        let ratio = if intersection.is_empty() {
            x
        } else {
            intersection[0].2 .1 as f32
        };
        let value = self.end_value.lerp(&self.start_value, ratio);
        let (mut curve_a, mut curve_b): (Curve<Coord2>, _) = curve.subdivide(x as f64);
        scale_curve(&mut curve_a);
        scale_curve(&mut curve_b);
        let keyframe_a = KeyFrame {
            start_value: self.start_value.clone(),
            end_value: value.clone(),
            easing_in: Some(Easing {
                x: vec![curve_a.control_points.1 .0 as f32],
                y: vec![curve_a.control_points.1 .1 as f32],
            }),
            easing_out: Some(Easing {
                x: vec![curve_a.control_points.0 .0 as f32],
                y: vec![curve_a.control_points.0 .1 as f32],
            }),
            start_frame: self.start_frame,
            end_frame: frame,
        };
        let keyframe_b = KeyFrame {
            start_value: value,
            end_value: self.end_value.clone(),
            easing_in: Some(Easing {
                x: vec![curve_b.control_points.1 .0 as f32],
                y: vec![curve_b.control_points.1 .1 as f32],
            }),
            easing_out: Some(Easing {
                x: vec![curve_b.control_points.0 .0 as f32],
                y: vec![curve_b.control_points.0 .1 as f32],
            }),
            start_frame: frame,
            end_frame: self.end_frame,
        };
        (Some(keyframe_a), Some(keyframe_b))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Easing {
    #[serde(deserialize_with = "array_from_array_or_number")]
    pub x: Vec<f32>,
    #[serde(deserialize_with = "array_from_array_or_number")]
    pub y: Vec<f32>,
}

fn scale_curve(curve: &mut Curve<Coord2>) {
    curve.control_points.0 = curve.control_points.0 - curve.start_point;
    curve.control_points.1 = curve.control_points.1 - curve.start_point;
    curve.end_point = curve.end_point - curve.start_point;
    curve.start_point = (0.0, 0.0).into();

    let x_scale = 1.0 / curve.end_point.0;
    let y_scale = 1.0 / curve.end_point.1;
    curve.control_points.0 .0 *= x_scale;
    curve.control_points.0 .1 *= y_scale;
    curve.control_points.1 .0 *= x_scale;
    curve.control_points.1 .1 *= y_scale;
    curve.control_points.0 .0 *= x_scale;
    curve.end_point = (1.0, 1.0).into()
}
