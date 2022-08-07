use std::fs;
use std::io::Error;

use lottie_model::{Animated, GradientFill, Rgb, Stroke, Transform};

#[test]
pub fn test_transform_complex() -> Result<(), Error> {
    let file = fs::File::open("../../fixtures/unit/transform_complex.json")?;
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
pub fn test_stroke() -> Result<(), Error> {
    let file = fs::File::open("../../fixtures/unit/stroke.json")?;
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
pub fn test_gradient_fill() -> Result<(), Error> {
    let file = fs::File::open("../../fixtures/unit/gradient_fill.json")?;
    let d = &mut serde_json::Deserializer::from_reader(file);
    let _: GradientFill = serde_path_to_error::deserialize(d).unwrap();
    Ok(())
}
