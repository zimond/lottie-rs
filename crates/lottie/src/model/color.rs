use alloc::vec;
use core::fmt;
use core::str::FromStr;

use super::helpers::{FromTo, Value};

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
        parse_rgba(s).map_err(|_| ())
    }
}

impl fmt::Display for Rgba {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
    }
}

fn parse_rgba(input: &str) -> Result<Rgba, &'static str> {
    let hex = input.strip_prefix('#').unwrap_or(input);
    let bytes = match hex.len() {
        6 => parse_hex_triplet(hex).map(|(r, g, b)| (r, g, b, 255))?,
        8 => parse_hex_quad(hex)?,
        _ => return Err("expected #RRGGBB or #RRGGBBAA"),
    };
    Ok(Rgba::new_u8(bytes.0, bytes.1, bytes.2, bytes.3))
}

fn parse_hex_triplet(hex: &str) -> Result<(u8, u8, u8), &'static str> {
    let bytes = hex.as_bytes();
    Ok((
        hex_pair(bytes, 0)?,
        hex_pair(bytes, 2)?,
        hex_pair(bytes, 4)?,
    ))
}

fn parse_hex_quad(hex: &str) -> Result<(u8, u8, u8, u8), &'static str> {
    let bytes = hex.as_bytes();
    Ok((
        hex_pair(bytes, 0)?,
        hex_pair(bytes, 2)?,
        hex_pair(bytes, 4)?,
        hex_pair(bytes, 6)?,
    ))
}

fn hex_pair(bytes: &[u8], start: usize) -> Result<u8, &'static str> {
    let hi = hex_val(bytes.get(start).copied().ok_or("missing digit")?)?;
    let lo = hex_val(bytes.get(start + 1).copied().ok_or("missing digit")?)?;
    Ok((hi << 4) | lo)
}

fn hex_val(byte: u8) -> Result<u8, &'static str> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err("invalid hex digit"),
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
