use std::ops::DerefMut;
use std::time::Duration;

use bevy::prelude::*;
#[cfg(feature = "bevy_egui")]
use bevy_egui::{egui, EguiContexts};
use bevy_tweening::{Animator, AnimatorState, Targetable, TweenCompleted};

use crate::LottieAnimationInfo;

#[cfg(feature = "bevy_egui")]
pub(crate) fn controls_system(mut egui_ctx: EguiContexts, mut info: ResMut<LottieAnimationInfo>) {
    let value = info.progress();
    let progress = egui::ProgressBar::new(value);
    let button_text = if info.paused() { "▶" } else { "⏸" };
    let button = egui::Button::new(button_text);
    let secs = egui::Label::new(format!("{:.2}", info.frame_rate * info.current_time));
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

/// We fork `bevy-tweening`'s animator system to stop its `time.delta_second()`
/// based ticking logic, as we are controlling the elapsed time of animations in
/// `animate_system`, what `bevy-tweening`'s doing is not needed anymore. Also,
/// bevy's `Time::delta()` is returning inconsistent value in scheduled plugin,
/// which causes animation to act wierd.
pub fn component_animator_system<T: Component>(
    mut query: Query<(Entity, &mut T, &mut Animator<T>)>,
    events: ResMut<Events<TweenCompleted>>,
) {
    let mut events: Mut<Events<TweenCompleted>> = events.into();
    for (entity, target, mut animator) in query.iter_mut() {
        if animator.state != AnimatorState::Paused {
            // TODO: maybe support animation speed, maybe not
            // let speed = animator.speed();
            let mut target = ComponentTarget::new(target);
            animator
                .tweenable_mut()
                .tick(Duration::ZERO, &mut target, entity, &mut events);
        }
    }
}

struct ComponentTarget<'a, T: Component> {
    target: Mut<'a, T>,
}

impl<'a, T: Component> ComponentTarget<'a, T> {
    pub fn new(target: Mut<'a, T>) -> Self {
        Self { target }
    }
}

impl<'a, T: Component> Targetable<T> for ComponentTarget<'a, T> {
    fn target_mut(&mut self) -> &mut T {
        self.target.deref_mut()
    }
}
