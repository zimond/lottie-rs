use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use lottie_core::{Config, Error, HeadlessConfig, Lottie, Renderer};
use lottie_renderer_bevy::BevyRenderer;
use rstest::rstest;
use smol::stream::StreamExt;

#[rstest]
fn check_fixture(
    #[files("../../fixtures/ui/checked/**/*.json")] path: PathBuf,
) -> Result<(), Error> {
    let f = File::open(&path)?;
    let lottie = Lottie::from_reader(f, "../../")?;
    let (mut renderer, frame_stream) = BevyRenderer::new();
    renderer.load_lottie(
        lottie,
        Config::Headless(HeadlessConfig {
            target: lottie_core::Target::Default,
            filename: String::from("test.webp"),
        }),
    );
    renderer.render();
    let filename = path.file_stem().unwrap();
    let mut path = path.clone();
    path.pop();
    path.push(&format!("{}.png", filename.to_str().unwrap_or_default()));
    let decoder = png::Decoder::new(File::open(&path)?);
    let mut reader = decoder.read_info().unwrap();
    // Allocate the output buffer.
    let mut buf = vec![0; reader.output_buffer_size()];
    // Read the next frame. An APNG might contain multiple frames.
    let info = reader.next_frame(&mut buf).unwrap();
    // Grab the bytes of the image.
    let bytes = &buf[..info.buffer_size()];
    let correct = bytes.to_vec();
    smol::block_on(async {
        smol::pin!(frame_stream);
        let mut i = 0;
        while let Some(frame) = frame_stream.next().await {
            let mut p = path.clone();
            p.pop();
            p.push(&format!(
                "{}_{}.png",
                filename.to_str().unwrap_or_default(),
                i
            ));
            i += 1;
            let f = File::create(&p).unwrap();
            let w = BufWriter::new(f);
            let mut encoder = png::Encoder::new(w, frame.width, frame.height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455)); // 1.0 / 2.2, scaled by 100000
            encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2)); // 1.0 / 2.2, unscaled, but rounded
            let source_chromaticities = png::SourceChromaticities::new(
                // Using unscaled instantiation here
                (0.31270, 0.32900),
                (0.64000, 0.33000),
                (0.30000, 0.60000),
                (0.15000, 0.06000),
            );
            encoder.set_source_chromaticities(source_chromaticities);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&frame.data)
                .unwrap();
            // assert_eq!(correct, frame.data);
        }
    });
    Ok(())
}
