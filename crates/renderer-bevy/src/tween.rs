use std::sync::Arc;
use std::time::Duration;

use bevy_tweening::{EaseMethod, Lens, Sequence, Tween};
use flo_curves::bezier::{curve_intersects_line, Curve};
use flo_curves::{BezierCurveFactory, Coord2};
use lottie_core::KeyFrame;

/// Produce [`Tweenable`](bevy_tweening::Tweenable) by using a `producer` to
/// create lens of type `L` from two instances of data of type
/// `TweenProducer::Key`
pub(crate) trait TweenProducer<T, L>
where
    L: Lens<T> + Send + Sync + 'static,
    T: 'static,
{
    type Key;
    fn tween<F>(&self, frame_rate: f32, producer: F) -> Sequence<T>
    where
        F: Fn(Self::Key, Self::Key) -> L;
}

impl<L, T, V> TweenProducer<T, L> for Vec<KeyFrame<V>>
where
    L: Lens<T> + Send + Sync + 'static,
    T: 'static,
    V: Clone,
{
    type Key = V;
    fn tween<F>(&self, frame_rate: f32, producer: F) -> Sequence<T>
    where
        F: Fn(Self::Key, Self::Key) -> L,
    {
        let mut seq = Sequence::with_capacity(self.len() + 1);
        for k in self.iter() {
            let start = k.start_value.clone();
            let end = k.end_value.clone();
            let ease_out = k.easing_out.clone().unwrap_or_default();
            let ease_in = k.easing_in.clone().unwrap_or_default();
            let frames = k.end_frame - k.start_frame;
            if frames <= 0.0 {
                continue;
            }
            let secs = frames as f32 / frame_rate as f32;
            let curve = Curve::from_points(
                Coord2(0.0, 0.0),
                (
                    Coord2(
                        ease_out.x.get(0).cloned().unwrap_or(0.0) as f64,
                        ease_out.y.get(0).cloned().unwrap_or(0.0) as f64,
                    ),
                    Coord2(
                        ease_in.x.get(0).cloned().unwrap_or(1.0) as f64,
                        ease_in.y.get(0).cloned().unwrap_or(1.0) as f64,
                    ),
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
                Duration::from_secs_f32(secs.max(f32::EPSILON)),
                producer(start, end),
            );
            seq = seq.then(t);
        }
        seq
    }
}
