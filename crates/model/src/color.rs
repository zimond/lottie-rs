use std::str::FromStr;

use crate::helpers::{FromTo, Value};

#[derive(Debug, Clone, Copy)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub fn new_f32(r: f32, g: f32, b: f32, a: f32) -> Rgba {
        Rgba {
            r: (r * 255.0) as u8,
            g: (g * 255.0) as u8,
            b: (b * 255.0) as u8,
            a: (a * 255.0) as u8,
        }
    }

    pub fn new_u8(r: u8, g: u8, b: u8, a: u8) -> Rgba {
        Rgba { r, g, b, a }
    }
}

impl Default for Rgba {
    fn default() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }
}

impl FromStr for Rgba {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        if s.starts_with("#") {
            chars.next();
        }
        let (rgb, a) = read_color::rgb_maybe_a(&mut chars).unwrap();
        Ok(Rgba::new_u8(rgb[0], rgb[1], rgb[2], a.unwrap_or(255)))
    }
}

impl ToString for Rgba {
    fn to_string(&self) -> String {
        todo!()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn new_f32(r: f32, g: f32, b: f32) -> Rgb {
        Rgb {
            r: (r * 255.0) as u8,
            g: (g * 255.0) as u8,
            b: (b * 255.0) as u8,
        }
    }

    pub fn new_u8(r: u8, g: u8, b: u8) -> Rgb {
        Rgb { r, g, b }
    }
}

impl FromTo<Value> for Rgba {
    fn from(v: Value) -> Self {
        let v = v.as_f32_vec().unwrap();
        if v[0] > 1.0 && v[0] <= 255.0 {
            Rgba::new_u8(
                v[0] as u8,
                v[1] as u8,
                v[2] as u8,
                v.get(3).cloned().unwrap_or(255.0) as u8,
            )
        } else {
            Rgba::new_f32(v[0], v[1], v[2], v.get(3).cloned().unwrap_or(1.0))
        }
    }

    fn to(self) -> Value {
        Value::List(vec![
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ])
    }
}
