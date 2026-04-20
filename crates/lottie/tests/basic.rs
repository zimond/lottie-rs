use std::fs;
use std::io::Error;
use std::path::PathBuf;

use lottie::prelude::Model;

#[test]
fn test_bouncy_ball_example() -> Result<(), Error> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/ui/bouncy_ball.json");
    let bytes = fs::read(path)?;
    let d = &mut serde_json::Deserializer::from_slice(&bytes);
    let _: Model = match serde_path_to_error::deserialize(d) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", e.path().to_string());
            panic!("abort");
        }
    };
    Ok(())
}
