use crate::{Bezier, Rgb, Value, Vector2D};

pub trait FromTo<T> {
    fn from(v: T) -> Self;
    fn to(self) -> T;
}

impl FromTo<Value> for Vector2D {
    fn from(v: Value) -> Self {
        let v = v.as_f32_vec().unwrap();
        Vector2D::new(v[0], v[1])
    }

    fn to(self) -> Value {
        todo!()
    }
}

impl FromTo<Value> for f32 {
    fn from(v: Value) -> Self {
        let v = v.as_f32_vec().unwrap();
        v[0]
    }

    fn to(self) -> Value {
        Value::Primitive(self)
    }
}

impl FromTo<Value> for Rgb {
    fn from(v: Value) -> Self {
        let v = v.as_f32_vec().unwrap();
        Rgb::new_f32(v[0], v[1], v[2])
    }

    fn to(self) -> Value {
        Value::List(vec![
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        ])
    }
}

impl FromTo<Value> for Vec<Bezier> {
    fn from(v: Value) -> Self {
        match v {
            Value::ComplexBezier(b) => b,
            _ => todo!(),
        }
    }

    fn to(self) -> Value {
        Value::ComplexBezier(self)
    }
}
