use flo_curves::{BoundingBox, Bounds, Coord2};
use lottie_ast::{rect, Animated, Bezier, Ellipse, KeyFrame, Rect, Vector2D};

use crate::Lottie;

pub trait AnimatedExt {
    type Target;
    fn initial_value(&self) -> Self::Target;
    fn value(&self, frame: u32) -> Self::Target;
    fn is_animated(&self) -> bool;
}

impl<T> AnimatedExt for Animated<T>
where
    T: Clone,
{
    type Target = T;

    fn initial_value(&self) -> Self::Target {
        self.keyframes[0].value.clone()
    }

    fn value(&self, mut frame: u32) -> Self::Target {
        if !self.is_animated() {
            return self.initial_value();
        }
        let len = self.keyframes.len() - 1;
        frame = std::cmp::max(self.keyframes[0].start_frame.unwrap_or(0), frame);
        frame = std::cmp::min(self.keyframes[len].start_frame.unwrap_or(0), frame);
        if let Some(window) = self.keyframes.windows(2).find(|window| {
            frame >= window[0].start_frame.unwrap() && frame < window[1].start_frame.unwrap()
        }) {
            window[0].value.clone()
        } else {
            self.keyframes[len].value.clone()
        }
    }

    fn is_animated(&self) -> bool {
        self.animated
    }
}

pub trait Shaped {
    fn bbox(&self, frame: u32) -> Rect<f32>;
}

impl Shaped for Ellipse {
    fn bbox(&self, frame: u32) -> Rect<f32> {
        let w = self.size.value(frame);
        let p = self.position.value(frame) - w / 2.0;
        Rect::new(p.to_point(), w.to_size())
    }
}

pub trait KeyFrameExt {
    fn alter_value<U>(&self, value: U) -> KeyFrame<U>;
}

impl<T> KeyFrameExt for KeyFrame<T> {
    fn alter_value<U>(&self, value: U) -> KeyFrame<U> {
        KeyFrame {
            value,
            start_frame: self.start_frame.clone(),
            easing_out: self.easing_out.clone(),
            easing_in: self.easing_in.clone(),
        }
    }
}

pub trait Renderer {
    fn load_lottie(&mut self, lottie: Lottie);
    fn render(&mut self);
}

pub trait PathExt {
    fn to_svg_d(&self) -> String;
    fn move_origin(&mut self, x: f32, y: f32);
}

impl PathExt for Vec<Bezier> {
    fn to_svg_d(&self) -> String {
        self.iter()
            .map(|b| b.to_svg_d())
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn move_origin(&mut self, x: f32, y: f32) {
        for b in self.iter_mut() {
            b.move_origin(x, y)
        }
    }
}

impl PathExt for Bezier {
    fn to_svg_d(&self) -> String {
        let mut result = String::new();
        let mut prev_c1: Option<Vector2D> = None;
        for ((p1, c1), c2) in self
            .vertices
            .iter()
            .zip(self.in_tangent.iter())
            .zip(self.out_tangent.iter())
        {
            if result.is_empty() {
                result.push_str(&format!(
                    "M {} {} C {} {} ",
                    p1.x,
                    p1.y,
                    p1.x + c2.x,
                    p1.y + c2.y
                ));
            } else if let Some(pc1) = prev_c1 {
                result.push_str(&format!(
                    "{} {} {} {} C {} {}",
                    p1.x + pc1.x,
                    p1.y + pc1.y,
                    p1.x,
                    p1.y,
                    p1.x + c2.x,
                    p1.y + c2.y
                ));
            }
            prev_c1 = Some(*c1);
        }
        result.truncate(result.rmatch_indices("C").next().unwrap().0);
        if self.closed {
            result.push('Z');
        }
        result
    }

    fn move_origin(&mut self, x: f32, y: f32) {
        for p1 in &mut self.vertices {
            p1.x += x;
            p1.y += y;
        }
    }
}

impl Shaped for Vec<Bezier> {
    fn bbox(&self, frame: u32) -> Rect<f32> {
        self.iter()
            .map(|b| b.bbox(frame))
            .reduce(|acc, item| acc.union(&item))
            .unwrap()
    }
}

impl Shaped for Bezier {
    fn bbox(&self, _: u32) -> Rect<f32> {
        let bbox = (0..(self.vertices.len() - 1))
            .map(|i| {
                let w1 = Coord2(self.vertices[i].x as f64, self.vertices[i].y as f64);
                let w2 = Coord2(
                    self.out_tangent[i].x as f64 + self.vertices[i].x as f64,
                    self.out_tangent[i].y as f64 + self.vertices[i].y as f64,
                );
                let w3 = Coord2(
                    self.in_tangent[i].x as f64 + self.vertices[i + 1].x as f64,
                    self.in_tangent[i].y as f64 + self.vertices[i + 1].y as f64,
                );
                let w4 = Coord2(self.vertices[i + 1].x as f64, self.vertices[i + 1].y as f64);
                flo_curves::bezier::bounding_box4(w1, w2, w3, w4)
            })
            .reduce(|acc: Bounds<Coord2>, bbox| acc.union_bounds(bbox))
            .unwrap();
        rect(
            bbox.min().0,
            bbox.min().1,
            bbox.max().0 - bbox.min().0,
            bbox.max().1 - bbox.min().1,
        )
        .cast()
    }
}
