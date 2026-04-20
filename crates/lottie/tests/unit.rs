use std::fs;
use std::io::Error;
use std::path::PathBuf;

use lottie::prelude::{Animated, GradientFill, Stroke, TextRange, Transform, Vector2D};

#[test]
fn test_transform_complex() -> Result<(), Error> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/segments/transform_complex.json");
    let bytes = fs::read(path)?;
    let d = &mut serde_json::Deserializer::from_slice(&bytes);
    let _: Transform = match serde_path_to_error::deserialize(d) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", e.path().to_string());
            panic!("abort");
        }
    };
    Ok(())
}

#[test]
fn test_stroke() -> Result<(), Error> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/segments/stroke.json");
    let bytes = fs::read(path)?;
    let d = &mut serde_json::Deserializer::from_slice(&bytes);
    let _: Stroke = match serde_path_to_error::deserialize(d) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", e.path().to_string());
            panic!("abort");
        }
    };
    Ok(())
}

#[test]
fn test_gradient_fill() -> Result<(), Error> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/segments/gradient_fill.json");
    let bytes = fs::read(path)?;
    let d = &mut serde_json::Deserializer::from_slice(&bytes);
    let _: GradientFill = serde_path_to_error::deserialize(d).unwrap();
    Ok(())
}

#[test]
fn test_text_range() -> Result<(), Error> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/segments/text_range.json");
    let bytes = fs::read(path)?;
    let d = &mut serde_json::Deserializer::from_slice(&bytes);
    let d: TextRange = serde_path_to_error::deserialize(d).unwrap();
    println!("{:?}", d);
    Ok(())
}

#[test]
fn test_legacy_animated_position() -> Result<(), Error> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/segments/animated_position_legacy.json");
    let bytes = fs::read(path)?;
    let d = &mut serde_json::Deserializer::from_slice(&bytes);
    let d: Animated<Vector2D> = serde_path_to_error::deserialize(d).unwrap();
    println!("{:?}", d);
    Ok(())
}
