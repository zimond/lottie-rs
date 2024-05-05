use std::fs;
use std::io::Error;

use lottie::prelude::{Animated, GradientFill, Stroke, TextRange, Transform, Vector2D};

#[test]
fn test_transform_complex() -> Result<(), Error> {
    let file = fs::File::open("../../fixtures/segments/transform_complex.json")?;
    let d = &mut serde_json::Deserializer::from_reader(file);
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
    let file = fs::File::open("../../fixtures/segments/stroke.json")?;
    let d = &mut serde_json::Deserializer::from_reader(file);
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
    let file = fs::File::open("../../fixtures/segments/gradient_fill.json")?;
    let d = &mut serde_json::Deserializer::from_reader(file);
    let _: GradientFill = serde_path_to_error::deserialize(d).unwrap();
    Ok(())
}

#[test]
fn test_text_range() -> Result<(), Error> {
    let file = fs::File::open("../../fixtures/segments/text_range.json")?;
    let d = &mut serde_json::Deserializer::from_reader(file);
    let d: TextRange = serde_path_to_error::deserialize(d).unwrap();
    println!("{:?}", d);
    Ok(())
}

#[test]
fn test_legacy_animated_position() -> Result<(), Error> {
    let file = fs::File::open("../../fixtures/segments/animated_position_legacy.json")?;
    let d = &mut serde_json::Deserializer::from_reader(file);
    let d: Animated<Vector2D> = serde_path_to_error::deserialize(d).unwrap();
    println!("{:?}", d);
    Ok(())
}
