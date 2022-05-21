use std::fs;
use std::io::Error;

use lottie_ast::Transform;

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
