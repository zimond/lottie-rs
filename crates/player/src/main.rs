use bevy::prelude::ResMut;
use bevy::winit::WinitPlugin;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use clap::Parser;
use lottie_core::{Lottie, Renderer};
use lottie_renderer_bevy::{BevyRenderer, LottieAnimationInfo};
use std::fs;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    input: String,
}

// fn axis_system(mut lines: ResMut<DebugLines>) {
//     lines.line(Vec3::new(0.0, 250.0, 0.0), Vec3::new(0.0, -250.0, 0.0), 1.0);
//     lines.line(Vec3::new(250.0, 0.0, 0.0), Vec3::new(-250.0, 0.0, 0.0), 1.0);
// }

fn controls_system(mut egui_ctx: ResMut<EguiContext>, mut info: ResMut<LottieAnimationInfo>) {
    let value = info.progress();
    let progress = egui::ProgressBar::new(value);
    let button_text = if info.paused() { "▶" } else { "⏸" };
    let button = egui::Button::new(button_text);
    let secs = egui::Label::new(format!("{:.2}", info.current_time()));
    egui::TopBottomPanel::bottom("slider_panel").show(egui_ctx.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.add(secs);
            if ui.add(button).clicked() {
                let paused = !info.paused();
                info.pause(paused);
            }
            ui.add_sized(ui.available_size(), progress);
        });
    });
}

fn main() {
    // env_logger::init();
    let args = Args::parse();
    let f = fs::File::open(&args.input).unwrap();
    let lottie = Lottie::from_reader(f).unwrap();
    let mut renderer = BevyRenderer::new();
    renderer.add_plugin(WinitPlugin);
    renderer.add_plugin(EguiPlugin);
    renderer.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new());
    renderer.add_system(controls_system);
    // renderer.add_plugin(DebugLinesPlugin::default());
    // renderer.add_system(axis_system);
    renderer.load_lottie(lottie);
    renderer.render();
}
