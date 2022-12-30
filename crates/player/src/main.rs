#![feature(path_file_prefix)]
// use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use clap::Parser;
use lottie_core::{Config, HeadlessConfig, Lottie, Renderer, Target, WindowConfig};
use lottie_renderer_bevy::BevyRenderer;
use std::fs;
use std::path::Path;

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
    let path = Path::new(&args.input);
    let mut root_path = path.to_path_buf();
    root_path.pop();
    let mut filename = path
        .file_prefix()
        .and_then(|name| name.to_str())
        .unwrap()
        .to_string();
    if filename.is_empty() {
        filename = "output".to_string();
    }
    let root_path = &*root_path.to_string_lossy();
    let f = fs::File::open(path).unwrap();
    let lottie = Lottie::from_reader(f, root_path).unwrap();
    let mut renderer = BevyRenderer::new();
    let config = if args.headless {
        Config::Headless(HeadlessConfig {
            target: Target::Default,
            filename,
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
