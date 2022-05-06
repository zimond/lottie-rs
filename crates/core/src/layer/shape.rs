use flo_curves::{BoundingBox, Bounds, Coord2};
use lottie_ast::*;

use crate::AnimatedExt;

pub struct ShapeIter {
    shapes: Vec<ShapeLayer>,
    shape_index: isize,
    stroke_index: usize,
}

impl<'a> Iterator for ShapeIter {
    type Item = StyledShape;

    fn next(&mut self) -> Option<Self::Item> {
        while self.shape_index >= 0 && !self.shapes[self.shape_index as usize].shape.is_shape() {
            self.shape_index -= 1;
        }
        if self.shape_index < 0 {
            return None;
        }
        let shape = self.shapes[self.shape_index as usize].clone();
        let mut fill = None;
        let mut transform = None;
        let mut stroke = None;
        for index in (self.shape_index as usize + 1)..self.shapes.len() {
            let shape = &self.shapes[index];
            if let Shape::Transform(t) = &shape.shape {
                transform = Some(t.clone());
                self.stroke_index = index;
                break;
            }
        }
        if transform.is_none() && self.stroke_index <= self.shape_index as usize {
            self.stroke_index = self.shapes.len();
        }
        let mut find_stroke = false;
        let mut target_stroke_index = self.stroke_index;
        for index in (self.shape_index as usize + 1)..self.shapes.len() {
            let shape = &self.shapes[index];
            if shape.shape.is_style() {
                if let Shape::Fill(f) = &shape.shape && fill.is_none() {
                    fill = Some(f.clone());
                } else if let Shape::Stroke(s) = &shape.shape {
                    find_stroke = true;
                    if index < self.stroke_index {
                        stroke = Some(s.clone());
                        target_stroke_index = index;
                    }
                }
            }
        }
        self.stroke_index = target_stroke_index;
        if stroke.is_none() && find_stroke {
            self.shape_index -= 1;
            return self.next();
        }
        if fill.is_none() && stroke.is_none() {
            self.shape_index -= 1;
            return self.next();
        }
        let fill = fill.unwrap_or_else(Fill::transparent);
        Some(StyledShape {
            shape,
            styles: vec![],
            stroke,
            fill,
            transform: transform.unwrap_or_default(),
        })
    }
}

pub trait ShapeIterator {
    fn shapes(&self) -> ShapeIter;
}

impl ShapeIterator for ShapeGroup {
    fn shapes(&self) -> ShapeIter {
        let shapes = flatten(&self.shapes);
        ShapeIter {
            shape_index: shapes.len() as isize - 1,
            stroke_index: 0,
            shapes,
        }
    }
}

fn flatten(shapes: &Vec<ShapeLayer>) -> Vec<ShapeLayer> {
    shapes
        .iter()
        .flat_map(|shape| {
            if let Shape::Group { shapes } = &shape.shape {
                flatten(shapes).into_iter()
            } else {
                vec![shape.clone()].into_iter()
            }
        })
        .collect()
}

pub struct StyledShape {
    pub shape: ShapeLayer,
    pub fill: Fill,
    pub stroke: Option<Stroke>,
    pub transform: Transform,
    pub styles: Vec<ShapeLayer>,
}

pub trait ShapeExt {
    fn is_style(&self) -> bool;
    fn is_shape(&self) -> bool;
}

impl ShapeExt for Shape {
    fn is_style(&self) -> bool {
        match &self {
            Shape::Fill { .. }
            | Shape::Stroke { .. }
            | Shape::GradientFill { .. }
            | Shape::GradientStroke { .. } => true,
            _ => false,
        }
    }

    fn is_shape(&self) -> bool {
        match &self {
            Shape::Rectangle { .. }
            | Shape::Ellipse { .. }
            | Shape::PolyStar { .. }
            | Shape::Path { .. } => true,
            _ => false,
        }
    }
}

pub trait Shaped {
    fn bbox(&self, frame: u32) -> Rect<f32>;
}

impl Shaped for Ellipse {
    fn bbox(&self, frame: u32) -> Rect<f32> {
        let s = self.size.value(frame);
        let p = self.position.value(frame) - s / 2.0;
        Rect::new(p.to_point(), s.to_size())
    }
}

pub trait PathExt {
    fn to_svg_d(&self) -> String;
    fn move_origin(&mut self, x: f32, y: f32);
    fn inverse_y_orientation(&mut self);
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

    fn inverse_y_orientation(&mut self) {
        for i in self.iter_mut() {
            i.inverse_y_orientation();
        }
    }
}

impl PathExt for Bezier {
    fn to_svg_d(&self) -> String {
        let mut result = String::new();
        let mut prev_c1: Option<Vector2D> = None;
        for ((p1, c1), c2) in self
            .verticies
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
                    "{} {} {} {} C {} {} ",
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
        for p1 in &mut self.verticies {
            p1.x += x;
            p1.y += y;
        }
    }

    fn inverse_y_orientation(&mut self) {
        for p in &mut self.verticies {
            p.y *= -1.0;
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
        let bbox = (0..(self.verticies.len() - 1))
            .map(|i| {
                let w1 = Coord2(self.verticies[i].x as f64, self.verticies[i].y as f64);
                let w2 = Coord2(
                    self.out_tangent[i].x as f64 + self.verticies[i].x as f64,
                    self.out_tangent[i].y as f64 + self.verticies[i].y as f64,
                );
                let w3 = Coord2(
                    self.in_tangent[i].x as f64 + self.verticies[i + 1].x as f64,
                    self.in_tangent[i].y as f64 + self.verticies[i + 1].y as f64,
                );
                let w4 = Coord2(
                    self.verticies[i + 1].x as f64,
                    self.verticies[i + 1].y as f64,
                );
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

impl Shaped for Shape {
    fn bbox(&self, frame: u32) -> Rect<f32> {
        match &self {
            Shape::Ellipse(e) => e.bbox(frame),
            Shape::Path { d } => d.value(frame).bbox(frame),
            _ => unimplemented!(),
        }
    }
}
