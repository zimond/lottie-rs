use bevy::winit::WinitPlugin;
use clap::Parser;
use lottie_core::{Lottie, Renderer};
use lottie_renderer_bevy::BevyRenderer;
use std::fs;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    input: String,
}

fn main() {
    let args = Args::parse();
    let f = fs::File::open(&args.input).unwrap();
    let lottie = Lottie::from_reader(f).unwrap();
    let mut renderer = BevyRenderer::new();
    renderer.add_plugin(WinitPlugin);
    renderer.load_lottie(lottie);
    renderer.render();
}
