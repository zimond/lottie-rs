use std::cmp::max;
use std::sync::Arc;
use std::time::Duration;

use bevy::prelude::Transform;
use bevy_tweening::{Delay, EaseMethod, Lens, Sequence, Tween, TweeningType};
use flo_curves::bezier::{curve_intersects_line, Curve};
use flo_curves::{BezierCurveFactory, Coord2};
use lottie_core::{KeyFrame, Transform as LottieTransform};

use crate::lens::TransformLens;

pub(crate) trait TweenProducer<T, L>
where
    L: Lens<T> + Send + Sync + 'static,
    T: 'static,
{
    type Key;
    fn tween(
        &self,
        frame_rate: u32,
        producer: fn(start: Self::Key, end: Self::Key) -> L,
    ) -> Sequence<T>;
}

impl<L, T, V> TweenProducer<T, L> for Vec<KeyFrame<V>>
where
    L: Lens<T> + Send + Sync + 'static,
    T: 'static,
    V: Clone,
{
    type Key = V;
    fn tween(
        &self,
        frame_rate: u32,
        producer: fn(start: Self::Key, end: Self::Key) -> L,
    ) -> Sequence<T> {
        let mut tween: Option<Sequence<T>> = None;
        for p in self.windows(2) {
            let p0 = &p[0];
            let p1 = &p[1];
            let start = p0.value.clone();
            let end = p1.value.clone();
            let ease_out = p0.easing_out.clone().unwrap();
            let ease_in = p0.easing_in.clone().unwrap();
            let frames = p1.start_frame.unwrap() - p0.start_frame.unwrap();
            let secs = frames as f32 / frame_rate as f32;
            let curve = Curve::from_points(
                Coord2(0.0, 0.0),
                (
                    Coord2(ease_out.x[0] as f64, ease_out.y[0] as f64),
                    Coord2(ease_in.x[0] as f64, ease_in.y[0] as f64),
                ),
                Coord2(1.0, 1.0),
            );
            let t = Tween::new(
                EaseMethod::CustomFunction(Arc::new(move |x| {
                    let intersection = curve_intersects_line(
                        &curve,
                        &(Coord2(x as f64, 0.0), Coord2(x as f64, 1.0)),
                    );
                    if intersection.is_empty() {
                        x
                    } else {
                        intersection[0].2 .1 as f32
                    }
                })),
                TweeningType::Once,
                Duration::from_secs_f32(secs),
                producer(start, end),
            );
            let t = if self[0].start_frame.unwrap() > 0 && tween.is_none() {
                Delay::new(Duration::from_secs_f32(
                    self[0].start_frame.unwrap() as f32 / (frame_rate as f32),
                ))
                .then(t)
            } else {
                Sequence::from_single(t)
            };
            tween = Some(match tween {
                Some(seq) => seq.then(t),
                None => Sequence::from_single(t),
            });
        }
        tween.unwrap()
    }
}

impl TweenProducer<Transform, TransformLens> for LottieTransform {
    type Key = LottieTransform;

    fn tween(
        &self,
        frame_rate: u32,
        producer: fn(start: Self::Key, end: Self::Key) -> TransformLens,
    ) -> Sequence<Transform> {
        let anchor_frames = self
            .anchor
            .as_ref()
            .and_then(|a| a.keyframes.last().unwrap().start_frame)
            .unwrap_or(0);
        let pos_frames = self
            .position
            .as_ref()
            .and_then(|a| a.keyframes.last().unwrap().start_frame)
            .unwrap_or(0);
        let scale_frames = self
            .scale.keyframes.last().unwrap().start_frame
            .unwrap_or(0);
        let rotation_frames = self
            .rotation.keyframes.last().unwrap().start_frame
            .unwrap_or(0);
        let frames = max(
            max(max(anchor_frames, pos_frames), scale_frames),
            rotation_frames,
        );
        let secs = frames as f32 / frame_rate as f32;
        let mut transform =  producer(self.clone(), self.clone());
        transform.frames = frames;
        let tween = Tween::new(
            EaseMethod::Linear,
            TweeningType::Once,
            Duration::from_secs_f32(secs), transform,
        );
        Sequence::from_single(tween)
    }
    //
}
