use crate::model::*;
use lyon_path::geom::euclid::approxeq::ApproxEq;
use lyon_path::geom::euclid::vec2;
use lyon_path::math::Angle;
use lyon_path::path::{Builder, Path};
use lyon_path::Winding;

pub struct StyledShapeIter {
    shapes: Vec<ShapeLayer>,
    shape_index: usize,
    stroke_index: usize,
}

impl StyledShapeIter {
    pub fn shape_count(&self) -> usize {
        self.shapes.len()
    }
}

impl<'a> Iterator for StyledShapeIter {
    type Item = StyledShape;

    fn next(&mut self) -> Option<Self::Item> {
        while self.shape_index < self.shapes.len()
            && !self.shapes[self.shape_index as usize].shape.is_shape()
            && !self.shapes[self.shape_index as usize].shape.is_group()
        {
            self.shape_index += 1;
            self.stroke_index = self.shape_index;
        }
        if self.shape_index >= self.shapes.len() {
            return None;
        }
        let shape = self.shapes[self.shape_index as usize].clone();
        let mut fill = None;
        let mut transform = Transform::default();
        let mut stroke = None;
        for index in (self.shape_index as usize + 1)..self.shapes.len() {
            let shape = &self.shapes[index];
            if let Shape::Transform(t) = &shape.shape {
                transform = t.clone();
                break;
            } else if shape.shape.is_shape() {
                break;
            }
        }
        let mut find_stroke = false;
        let mut target_stroke_index = self.stroke_index;
        for index in (self.shape_index as usize + 1)..self.shapes.len() {
            let shape = &self.shapes[index];
            if shape.shape.is_style() && !shape.hidden {
                match &shape.shape {
                    Shape::Fill(f) if fill.is_none() => fill = Some(AnyFill::Solid(f.clone())),
                    Shape::GradientFill(f) if fill.is_none() => {
                        fill = Some(AnyFill::Gradient(f.clone()))
                    }
                    Shape::Stroke(s) => {
                        find_stroke = true;
                        if index > self.stroke_index && stroke.is_none() {
                            stroke = Some(AnyStroke::Solid(s.clone()));
                            target_stroke_index = index;
                        }
                    }
                    Shape::GradientStroke(s) => {
                        find_stroke = true;
                        if index > self.stroke_index && stroke.is_none() {
                            stroke = Some(AnyStroke::Gradient(s.clone()));
                            target_stroke_index = index;
                        }
                    }
                    _ => {}
                }
            } else if let Shape::Transform(_) = &shape.shape {
                break;
            }
        }
        self.stroke_index = target_stroke_index;
        if stroke.is_none() && find_stroke {
            self.shape_index += 1;
            self.stroke_index = self.shape_index as usize;
            return self.next();
        }
        if fill.is_none()
            && stroke.is_none()
            && !self.shapes[self.shape_index as usize].shape.is_group()
        {
            self.shape_index += 1;
            self.stroke_index = self.shape_index as usize;
            return self.next();
        }
        let fill = fill.unwrap_or_else(|| AnyFill::Solid(Fill::transparent()));
        if !find_stroke {
            self.shape_index += 1;
            self.stroke_index = self.shape_index as usize;
        }
        // collect trim info
        let mut trims = vec![];
        for (_index, shape) in self.shapes.iter().rev().enumerate() {
            if let Shape::Trim(trim) = &shape.shape {
                match trim.multiple_shape {
                    TrimMultipleShape::Individually => trims.push(TrimInfo {
                        trim: trim.clone(),
                        shapes: vec![shape.shape.clone()],
                    }),
                    TrimMultipleShape::Simultaneously => {
                        // TODO: implement this
                        continue;
                    }
                }
            }
        }
        Some(StyledShape {
            shape,
            styles: vec![],
            stroke,
            fill,
            transform,
            trims,
        })
    }
}

pub trait StyledShapeIterator {
    fn styled_shapes(&self) -> StyledShapeIter;
}

impl StyledShapeIterator for ShapeGroup {
    fn styled_shapes(&self) -> StyledShapeIter {
        StyledShapeIter {
            shape_index: 0,
            stroke_index: 0,
            shapes: self.shapes.clone(),
        }
    }
}

pub enum AnyFill {
    Solid(Fill),
    Gradient(GradientFill),
}

impl AnyFill {
    pub fn opacity(&self) -> &Animated<f32> {
        match &self {
            AnyFill::Solid(s) => &s.opacity,
            AnyFill::Gradient(g) => &g.opacity,
        }
    }
}

pub enum AnyStroke {
    Solid(Stroke),
    Gradient(GradientStroke),
}

impl AnyStroke {
    pub fn width(&self) -> &Animated<f32> {
        match &self {
            AnyStroke::Solid(s) => &s.width,
            AnyStroke::Gradient(g) => &g.width,
        }
    }

    pub fn line_cap(&self) -> LineCap {
        match &self {
            AnyStroke::Solid(s) => s.line_cap,
            AnyStroke::Gradient(g) => g.line_cap,
        }
    }

    pub fn line_join(&self) -> LineJoin {
        match &self {
            AnyStroke::Solid(s) => s.line_join,
            AnyStroke::Gradient(g) => g.line_join,
        }
    }

    pub fn opacity(&self) -> &Animated<f32> {
        match &self {
            AnyStroke::Solid(s) => &s.opacity,
            AnyStroke::Gradient(g) => &g.opacity,
        }
    }
}

#[derive(Clone)]
pub struct TrimInfo {
    pub trim: Trim,
    pub shapes: Vec<Shape>,
}

pub struct StyledShape {
    pub shape: ShapeLayer,
    pub fill: AnyFill,
    pub stroke: Option<AnyStroke>,
    pub transform: Transform,
    pub styles: Vec<ShapeLayer>,
    pub trims: Vec<TrimInfo>,
}

impl Shape {
    pub fn is_style(&self) -> bool {
        match &self {
            Shape::Fill { .. }
            | Shape::Stroke { .. }
            | Shape::GradientFill { .. }
            | Shape::GradientStroke { .. } => true,
            _ => false,
        }
    }

    pub fn is_shape(&self) -> bool {
        match &self {
            Shape::Rectangle { .. }
            | Shape::Ellipse { .. }
            | Shape::PolyStar { .. }
            | Shape::Path { .. } => true,
            _ => false,
        }
    }

    pub fn is_group(&self) -> bool {
        match &self {
            Shape::Group { .. } => true,
            _ => false,
        }
    }
}

/// Allows a shape to generate a [Path](lyon_path::Path) at a certain `frame`
pub trait PathFactory {
    fn path(&self, frame: f32) -> Path;
    fn is_animated(&self) -> bool;
}

impl PathFactory for Ellipse {
    fn path(&self, frame: f32) -> Path {
        let size = self.size.value(frame) / 2.0;
        let position = self.position.value(frame);
        let mut builder = Builder::new();
        let winding = match self.direction {
            ShapeDirection::Clockwise => Winding::Positive,
            ShapeDirection::CounterClockwise => Winding::Negative,
        };
        builder.add_ellipse(position.to_point(), size, Angle::zero(), winding);
        builder.build()
    }

    fn is_animated(&self) -> bool {
        self.size.is_animated() || self.position.is_animated()
    }
}

impl PathFactory for Vec<Bezier> {
    fn path(&self, frame: f32) -> Path {
        let mut builder = Builder::new();
        for b in self.iter() {
            let mut prev_p: Option<Vector2D>;
            match b.verticies.first() {
                Some(p) => {
                    builder.begin(p.to_point());
                    prev_p = Some(*p);
                }
                None => continue,
            }
            for ((p, c1), c2) in b
                .verticies
                .iter()
                .skip(1)
                .zip(b.out_tangent.iter())
                .zip(b.in_tangent.iter().skip(1))
            {
                if let Some(p0) = prev_p {
                    let p1 = p0 + *c1;
                    let p2 = *p + *c2;
                    if c1.approx_eq(&Vector2D::zero()) && c2.approx_eq(&Vector2D::zero()) {
                        builder.line_to(p.to_point());
                    } else if p1.approx_eq(&p2) {
                        builder.quadratic_bezier_to(p1.to_point(), p.to_point());
                    } else {
                        builder.cubic_bezier_to(p1.to_point(), p2.to_point(), p.to_point());
                    }
                }
                prev_p = Some(*p);
            }
            if b.closed {
                let index = b.verticies.len() - 1;
                builder.cubic_bezier_to(
                    (b.verticies[index] + b.out_tangent[index]).to_point(),
                    (b.verticies[0] + b.in_tangent[0]).to_point(),
                    b.verticies[0].to_point(),
                );
            }
            builder.end(b.closed);
        }
        builder.build()
    }

    fn is_animated(&self) -> bool {
        false
    }
}

impl PathFactory for PolyStar {
    fn path(&self, frame: f32) -> Path {
        let mut builder = Builder::new();
        const PI: f32 = std::f32::consts::PI;
        const MAGIC_NUM: f32 = 0.47829 / 0.28;
        let cp = self.position.value(frame);
        let num_points = self.points.value(frame) as u32 * 2;
        let mut long_flag = false;
        let outer_rad = self.outer_radius.value(frame);
        let inner_rad = self
            .inner_radius
            .as_ref()
            .map(|s| s.value(frame))
            .unwrap_or(0.0);
        let outer_round = self.outer_roundness.value(frame) / 100.0;
        let inner_round = self
            .inner_roundness
            .as_ref()
            .map(|s| s.value(frame))
            .unwrap_or(0.0)
            / 100.0;
        let outer: Vector2D = vec2(outer_rad, outer_round);
        let inner = vec2(inner_rad, inner_round);
        let mut current_ang = (self.rotation.value(frame) - 90.0) * PI / 180.0;
        let angle_per_point = 2.0 * PI / num_points as f32;
        let has_roundness = outer_round != 0.0 && inner_round != 0.0;
        let angle_dir = if self.direction == ShapeDirection::Clockwise {
            1.0
        } else {
            -1.0
        };

        let mut p = vec2(current_ang.cos(), current_ang.sin()) * outer.x;
        builder.begin(p.to_point() + cp);
        current_ang += angle_per_point * angle_dir;
        for _ in 0..num_points {
            if !long_flag && self.star_type == PolyStarType::Polygon {
                current_ang += angle_per_point;
                long_flag = !long_flag;
                continue;
            }
            let (cp1_info, cp2_info) = if long_flag {
                (inner, outer)
            } else {
                (outer, inner)
            };
            let prev = p;
            p = vec2(current_ang.cos(), current_ang.sin()) * cp2_info.x;
            if has_roundness {
                let cp1_theta = prev.y.atan2(prev.x) - PI / 2.0 * angle_dir;
                let cp1_d = vec2(cp1_theta.cos(), cp1_theta.sin());
                let cp2_theta = p.y.atan2(p.x) - PI / 2.0 * angle_dir;
                let cp2_d = vec2(cp2_theta.cos(), cp2_theta.sin());
                let cp1 = cp1_d * (cp1_info.x * cp1_info.y * MAGIC_NUM / num_points as f32 * 2.0);
                let cp2 = cp2_d * (cp2_info.x * cp2_info.y * MAGIC_NUM / num_points as f32 * 2.0);
                builder.cubic_bezier_to(
                    (prev - cp1).to_point() + cp,
                    (p + cp2).to_point() + cp,
                    p.to_point() + cp,
                );
            } else {
                builder.line_to(p.to_point() + cp);
            }
            current_ang += angle_per_point;
            long_flag = !long_flag;
        }
        builder.end(true);
        builder.build()
    }

    fn is_animated(&self) -> bool {
        // FIXME:
        false
    }
}

impl PathFactory for Rectangle {
    fn path(&self, frame: f32) -> Path {
        let mut builder = Builder::new();
        let center = self.position.value(frame).to_point();
        let size = self.size.value(frame) / 2.0;
        let mut pts = vec![
            center - size,
            center + vec2(size.x, -size.y),
            center + size,
            center + vec2(-size.x, size.y),
        ];
        if self.direction == ShapeDirection::CounterClockwise {
            pts.reverse();
        }
        builder.begin(pts[0]);
        builder.line_to(pts[1]);
        builder.line_to(pts[2]);
        builder.line_to(pts[3]);
        builder.end(true);
        builder.build()
    }

    fn is_animated(&self) -> bool {
        self.position.is_animated() || self.radius.is_animated() || self.size.is_animated()
    }
}
