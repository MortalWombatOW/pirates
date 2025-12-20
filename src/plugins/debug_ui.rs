use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy_egui::{egui, EguiContexts};
use crate::plugins::core::GameState;
use crate::resources::Wind;

pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<FrameTimeDiagnosticsPlugin>() {
            app.add_plugins(FrameTimeDiagnosticsPlugin::default());
        }
        
        app.add_systems(Update, debug_panel);
    }
}

fn debug_panel(
    mut contexts: EguiContexts,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    diagnostics: Res<DiagnosticsStore>,
    wind: Option<Res<Wind>>,
) {
    egui::Window::new("Debug Panel").show(contexts.ctx_mut(), |ui| {
        ui.label(format!("Current State: {:?}", state.get()));
        
        if let Some(fps) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|diag| diag.smoothed())
        {
            ui.label(format!("FPS: {:.1}", fps));
        }

        // Wind display
        if let Some(wind) = wind {
            ui.separator();
            ui.heading("Wind");
            ui.label(format!(
                "Direction: {} ({:.0}°)",
                wind.cardinal_direction(),
                wind.direction.to_degrees().rem_euclid(360.0)
            ));
            ui.label(format!("Strength: {:.0}%", wind.strength * 100.0));
        }

        ui.separator();
        ui.heading("State Transitions");
        
        if ui.button("Main Menu").clicked() {
            next_state.set(GameState::MainMenu);
        }
        if ui.button("Port").clicked() {
            next_state.set(GameState::Port);
        }
        if ui.button("High Seas").clicked() {
            next_state.set(GameState::HighSeas);
        }
        if ui.button("Combat").clicked() {
            next_state.set(GameState::Combat);
        }
        if ui.button("Game Over").clicked() {
            next_state.set(GameState::GameOver);
        }
    });
}
