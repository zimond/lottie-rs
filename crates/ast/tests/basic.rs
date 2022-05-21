use std::fs;
use std::io::Error;

use lottie_ast::Model;

#[test]
fn test_bouncy_ball_example() -> Result<(), Error> {
    let file = fs::File::open("../../fixtures/2016.json")?;
    let d = &mut serde_json::Deserializer::from_reader(file);
    let _: Model = match serde_path_to_error::deserialize(d) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", e.path().to_string());
            panic!("abort");
        }
    };
    Ok(())
}
