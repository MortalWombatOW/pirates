//! FadeController plugin - provides smooth alpha fade animations.
//!
//! Registers the `animate_fades` system that lerps FadeController.current_alpha
//! toward target_alpha each frame.

use bevy::prelude::*;

use crate::components::fade_controller::FadeController;
use crate::plugins::core::GameState;

pub struct FadeControllerPlugin;

impl Plugin for FadeControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            animate_fades.run_if(in_state(GameState::HighSeas)),
        );
    }
}

/// System that animates all FadeController components.
/// Lerps current_alpha toward target_alpha using delta time.
fn animate_fades(
    time: Res<Time>,
    mut query: Query<&mut FadeController>,
) {
    let dt = time.delta_secs();

    for mut fade in &mut query {
        if !fade.is_fading() {
            continue;
        }

        let direction = if fade.target_alpha > fade.current_alpha { 1.0 } else { -1.0 };
        let delta = fade.fade_speed * dt * direction;

        fade.current_alpha = if direction > 0.0 {
            (fade.current_alpha + delta).min(fade.target_alpha)
        } else {
            (fade.current_alpha + delta).max(fade.target_alpha)
        };
    }
}
