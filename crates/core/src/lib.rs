pub use lottie_ast::*;

mod error;
mod traits;

pub use error::Error;
pub use traits::*;

#[derive(Clone)]
pub struct Lottie {
    pub model: LottieModel,
    id_counter: u32,
}

impl Lottie {
    pub fn from_reader<R: std::io::Read>(r: R) -> Result<Self, lottie_ast::Error> {
        let mut model = LottieModel::from_reader(r)?;
        // assign ids to shapes
        let mut id_counter = 1;
        for (index, layer) in model.layers.iter_mut().enumerate() {
            layer.id = index as u32;
            if let LayerContent::Shape(ShapeGroup { shapes }) = &mut layer.content {
                for layer in shapes {
                    assign_id(layer, &mut id_counter);
                }
            }
        }
        Ok(Lottie { model, id_counter })
    }
}

fn assign_id(layer: &mut ShapeLayer, id_counter: &mut u32) {
    layer.id = *id_counter;
    *id_counter += 1;
    if let Shape::Group { shapes } = &mut layer.shape {
        for shape in shapes {
            assign_id(shape, id_counter);
        }
    }
}

pub trait ShapeIterator {
    fn shapes(&self) -> ShapeIter;
}

impl ShapeIterator for ShapeGroup {
    fn shapes(&self) -> ShapeIter {
        let shapes = flatten(&self.shapes);
        ShapeIter { shapes, index: 0 }
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

pub struct ShapeIter {
    shapes: Vec<ShapeLayer>,
    index: usize,
}

impl<'a> Iterator for ShapeIter {
    type Item = StyledShape;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.shapes.len() {
            self.shapes.pop();
        }
        while self.index < self.shapes.len() && !self.shapes[self.index].shape.is_shape() {
            self.index += 1;
        }
        if self.index >= self.shapes.len() {
            return None;
        }
        let shape = self.shapes[self.index].clone();
        let mut fill = None;
        let mut transform = None;
        let mut strokes = vec![];
        let mut styles = vec![];
        self.index += 1;
        while self.index < self.shapes.len() {
            if self.shapes[self.index].shape.is_style() {
                if let Shape::Fill(f) = &self.shapes[self.index].shape {
                    if fill.is_none() {
                        fill = Some(f.clone());
                    }
                } else if let Shape::Stroke(stroke) = &self.shapes[self.index].shape {
                    strokes.push(stroke.clone());
                } else {
                    styles.push(self.shapes[self.index].clone());
                }
                self.index += 1;
            } else if let Shape::Transform(t) = &self.shapes[self.index].shape {
                transform = Some(t.clone());
                self.index += 1;
            } else if self.shapes[self.index].shape.is_shape() {
                self.index -= 1;
                break;
            }
        }
        let fill = match fill {
            Some(f) => f,
            None if strokes.is_empty() => return self.next(),
            _ => Fill::transparent(),
        };
        let transform = match transform {
            Some(f) => f,
            None => return self.next(),
        };
        Some(StyledShape {
            shape,
            styles,
            strokes,
            fill,
            transform,
        })
    }
}

pub struct StyledShape {
    pub shape: ShapeLayer,
    pub fill: Fill,
    pub strokes: Vec<Stroke>,
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
