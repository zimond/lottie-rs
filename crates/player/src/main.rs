// use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use clap::Parser;
use lottie_core::{Config, HeadlessConfig, Lottie, Renderer, Target, WindowConfig};
use lottie_renderer_bevy::BevyRenderer;
use std::fs;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    input: String,
    #[clap(long, action)]
    headless: bool,
}

// fn axis_system(mut lines: ResMut<DebugLines>) {
//     lines.line(Vec3::new(0.0, 250.0, 0.0), Vec3::new(0.0, -250.0, 0.0), 1.0);
//     lines.line(Vec3::new(250.0, 0.0, 0.0), Vec3::new(-250.0, 0.0, 0.0), 1.0);
// }

fn main() {
    let args = Args::parse();
    let f = fs::File::open(&args.input).unwrap();
    let lottie = Lottie::from_reader(f).unwrap();
    let mut renderer = BevyRenderer::new(lottie.model.width, lottie.model.height);
    let config = if args.headless {
        Config::Headless(HeadlessConfig {
            target: Target::Default,
        })
    } else {
        Config::Window(WindowConfig {
            show_controls: true,
            show_debug: true,
        })
    };
    // renderer.add_plugin(DebugLinesPlugin::default());
    // renderer.add_system(axis_system);
    renderer.load_lottie(lottie, config);
    renderer.render();
}
