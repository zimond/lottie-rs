use bevy::math::Vec3;
use bevy::prelude::ResMut;
use bevy::winit::WinitPlugin;
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
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

fn axis_system(mut lines: ResMut<DebugLines>) {
    lines.line(Vec3::new(0.0, 250.0, 0.0), Vec3::new(0.0, -250.0, 0.0), 1.0);
    lines.line(Vec3::new(250.0, 0.0, 0.0), Vec3::new(-250.0, 0.0, 0.0), 1.0);
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    let f = fs::File::open(&args.input).unwrap();
    let lottie = Lottie::from_reader(f).unwrap();
    let mut renderer = BevyRenderer::new();
    renderer.add_plugin(WinitPlugin);
    renderer.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new());
    // renderer.add_plugin(DebugLinesPlugin::default());
    // renderer.add_system(axis_system);
    renderer.load_lottie(lottie);
    renderer.render();
}
