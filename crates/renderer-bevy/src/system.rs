use bevy::prelude::ResMut;
#[cfg(feature = "bevy_egui")]
use bevy_egui::{egui, EguiContext, EguiPlugin};

use crate::LottieAnimationInfo;

#[cfg(feature = "bevy_egui")]
pub(crate) fn controls_system(
    mut egui_ctx: ResMut<EguiContext>,
    mut info: ResMut<LottieAnimationInfo>,
) {
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
