use alloc::vec;
use alloc::vec::Vec;
use alloc::collections::VecDeque;
use core::cmp::Ordering;
use serde::{Deserialize, Serialize};

use crate::Lerp;

use super::helpers::{self, *};

#[derive(Clone, Copy, Debug)]
struct Point2 {
    x: f32,
    y: f32,
}

fn bezier_point(p0: Point2, p1: Point2, p2: Point2, p3: Point2, t: f32) -> Point2 {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let t2 = t * t;
    Point2 {
        x: mt2 * mt * p0.x
            + 3.0 * mt2 * t * p1.x
            + 3.0 * mt * t2 * p2.x
            + t2 * t * p3.x,
        y: mt2 * mt * p0.y
            + 3.0 * mt2 * t * p1.y
            + 3.0 * mt * t2 * p2.y
            + t2 * t * p3.y,
    }
}

fn bezier_derivative(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let mt = 1.0 - t;
    3.0 * mt * mt * (p1 - p0) + 6.0 * mt * t * (p2 - p1) + 3.0 * t * t * (p3 - p2)
}

fn solve_bezier_x(x1: f32, x2: f32, target: f32) -> f32 {
    let mut t = target.clamp(0.0, 1.0);
    let mut lower = 0.0;
    let mut upper = 1.0;

    for _ in 0..8 {
        let x = bezier_point(
            Point2 { x: 0.0, y: 0.0 },
            Point2 { x: x1, y: 0.0 },
            Point2 { x: x2, y: 0.0 },
            Point2 { x: 1.0, y: 0.0 },
            t,
        )
        .x;
        let diff = x - target;
        if diff.abs() < 1e-5 {
            return t;
        }
        match diff.partial_cmp(&0.0).unwrap_or(Ordering::Equal) {
            Ordering::Less => lower = t,
            Ordering::Greater => upper = t,
            Ordering::Equal => return t,
        }
        let derivative = bezier_derivative(0.0, x1, x2, 1.0, t);
        if derivative.abs() < 1e-6 {
            break;
        }
        let next = t - diff / derivative;
        if (lower..=upper).contains(&next) {
            t = next;
        } else {
            break;
        }
    }

    for _ in 0..16 {
        t = 0.5 * (lower + upper);
        let x = bezier_point(
            Point2 { x: 0.0, y: 0.0 },
            Point2 { x: x1, y: 0.0 },
            Point2 { x: x2, y: 0.0 },
            Point2 { x: 1.0, y: 0.0 },
            t,
        )
        .x;
        if (x - target).abs() < 1e-5 {
            break;
        }
        if x < target {
            lower = t;
        } else {
            upper = t;
        }
    }
    t
}

fn split_bezier(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    split_t: f32,
) -> ((f32, f32, f32, f32), (f32, f32, f32, f32), f32) {
    let p0 = Point2 { x: 0.0, y: 0.0 };
    let p1 = Point2 { x: x1, y: y1 };
    let p2 = Point2 { x: x2, y: y2 };
    let p3 = Point2 { x: 1.0, y: 1.0 };

    let p01 = lerp_point(p0, p1, split_t);
    let p12 = lerp_point(p1, p2, split_t);
    let p23 = lerp_point(p2, p3, split_t);
    let p012 = lerp_point(p01, p12, split_t);
    let p123 = lerp_point(p12, p23, split_t);
    let p0123 = lerp_point(p012, p123, split_t);

    let left = normalize_segment(p01, p012, p0123);
    let right = normalize_segment(
        Point2 {
            x: p123.x - p0123.x,
            y: p123.y - p0123.y,
        },
        Point2 {
            x: p23.x - p0123.x,
            y: p23.y - p0123.y,
        },
        Point2 {
            x: 1.0 - p0123.x,
            y: 1.0 - p0123.y,
        },
    );

    (left, right, p0123.y)
}

fn lerp_point(a: Point2, b: Point2, t: f32) -> Point2 {
    Point2 {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
    }
}

fn normalize_segment(p1: Point2, p2: Point2, end: Point2) -> (f32, f32, f32, f32) {
    let x_scale = if end.x.abs() < f32::EPSILON { 1.0 } else { end.x };
    let y_scale = if end.y.abs() < f32::EPSILON { 1.0 } else { end.y };
    (
        p1.x / x_scale,
        p1.y / y_scale,
        p2.x / x_scale,
        p2.y / y_scale,
    )
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
        bound = "T: Clone + FromTo<helpers::Value>",
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
            .find(|keyframe| frame > keyframe.start_frame && frame <= keyframe.end_frame)
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
        let x = solve_bezier_x(ease_out.x[0], ease_in.x[0], t);
        let curve = bezier_point(
            Point2 { x: 0.0, y: 0.0 },
            Point2 {
                x: ease_out.x[0],
                y: ease_out.y[0],
            },
            Point2 {
                x: ease_in.x[0],
                y: ease_in.y[0],
            },
            Point2 { x: 1.0, y: 1.0 },
            x,
        );
        let ratio = curve.y;
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
        let split_t = solve_bezier_x(ease_out.x[0], ease_in.x[0], x);
        let ratio = bezier_point(
            Point2 { x: 0.0, y: 0.0 },
            Point2 {
                x: ease_out.x[0],
                y: ease_out.y[0],
            },
            Point2 {
                x: ease_in.x[0],
                y: ease_in.y[0],
            },
            Point2 { x: 1.0, y: 1.0 },
            split_t,
        )
        .y;
        let value = self.end_value.lerp(&self.start_value, ratio);
        let (curve_a, curve_b, _) = split_bezier(
            ease_out.x[0],
            ease_out.y[0],
            ease_in.x[0],
            ease_in.y[0],
            split_t,
        );
        let keyframe_a = KeyFrame {
            start_value: self.start_value.clone(),
            end_value: value.clone(),
            easing_in: Some(Easing {
                x: vec![curve_a.2],
                y: vec![curve_a.3],
            }),
            easing_out: Some(Easing {
                x: vec![curve_a.0],
                y: vec![curve_a.1],
            }),
            start_frame: self.start_frame,
            end_frame: frame,
        };
        let keyframe_b = KeyFrame {
            start_value: value,
            end_value: self.end_value.clone(),
            easing_in: Some(Easing {
                x: vec![curve_b.2],
                y: vec![curve_b.3],
            }),
            easing_out: Some(Easing {
                x: vec![curve_b.0],
                y: vec![curve_b.1],
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
