use std::fs::File;
use std::path::PathBuf;

use lottie::{Config, Error, HeadlessConfig, Lottie, Renderer};
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
            target: lottie::Target::Default,
            filename: String::from("test.webp"),
            frame: None,
        }),
    );
    renderer.render();
    let filename = path.file_stem().unwrap();
    let mut path = path.clone();
    path.pop();
    path.push(filename.to_str().unwrap_or_default());
    let mut checked_frames = glob::glob(&format!("{}_*.png", path.to_str().unwrap_or_default()))
        .unwrap()
        .filter_map(|entry| {
            let path = entry.ok()?;
            let path = path.to_str()?.split('/').last()?;
            let index = path.split(&['_', '.']).skip(1).next().unwrap_or_default();
            let index = index.parse::<u32>().ok()?;
            Some(index)
        })
        .collect::<Vec<_>>();
    checked_frames.sort();
    smol::block_on(async {
        smol::pin!(frame_stream);
        let mut i = 0;
        while let Some(frame) = frame_stream.next().await {
            if !checked_frames.contains(&i) {
                i += 1;
                continue;
            }
            let mut p = path.clone();
            p.pop();
            p.push(&format!(
                "{}_{}.png",
                filename.to_str().unwrap_or_default(),
                i
            ));

            let decoder = png::Decoder::new(File::open(&p).unwrap());
            let mut reader = decoder.read_info().unwrap();
            // Allocate the output buffer.
            let mut buf = vec![0; reader.output_buffer_size()];
            // Read the next frame. An APNG might contain multiple frames.
            let info = reader.next_frame(&mut buf).unwrap();
            // Grab the bytes of the image.
            let bytes = &buf[..info.buffer_size()];
            let correct = bytes.to_vec();

            i += 1;

            assert_eq!(correct, frame.data);
        }
    });
    Ok(())
}
