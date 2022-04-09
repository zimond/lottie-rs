use std::fs;
use std::io::Error;

use lottie_ast::LottieModel;

#[test]
fn test_bouncy_ball_example() -> Result<(), Error> {
    let file = fs::File::open("tests/bouncy_ball.json")?;
    let d = &mut serde_json::Deserializer::from_reader(file);
    let lottie: LottieModel = serde_path_to_error::deserialize(d).unwrap();
    println!("{:?}", lottie);
    Ok(())
}
