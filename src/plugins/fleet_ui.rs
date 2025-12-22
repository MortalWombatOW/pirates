use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::resources::PlayerFleet;

/// Plugin for the Fleet Management UI.
pub struct FleetUiPlugin;

impl Plugin for FleetUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<FleetUiState>()
            .add_systems(Update, (
                toggle_fleet_ui_system,
                fleet_ui_system,
            ));
    }
}

/// Resource to track UI state.
#[derive(Resource, Default)]
pub struct FleetUiState {
    pub is_open: bool,
}

/// System to toggle the UI with 'F' key.
fn toggle_fleet_ui_system(
    mut ui_state: ResMut<FleetUiState>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::KeyF) {
        ui_state.is_open = !ui_state.is_open;
        info!("Fleet UI toggled: {}", ui_state.is_open);
    }
}

/// Main system to render the Fleet UI.
fn fleet_ui_system(
    mut contexts: EguiContexts,
    ui_state: Res<FleetUiState>,
    player_fleet: Res<PlayerFleet>,
) {
    if !ui_state.is_open {
        return;
    }

    egui::Window::new("Fleet Management")
        .default_width(300.0)
        .default_height(400.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Your Fleet");
            ui.separator();

            if player_fleet.ships.is_empty() {
                ui.label("No ships in fleet.");
                ui.weak("Capture enemy ships to build your fleet!");
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, ship) in player_fleet.ships.iter().enumerate() {
                        ui.group(|ui| {
                            ui.strong(format!("{}. {}", i + 1, ship.name));
                            
                            // Health bar
                            let health_pct = ship.hull_health / ship.max_hull_health;
                            ui.horizontal(|ui| {
                                ui.label("Health:");
                                ui.add(egui::ProgressBar::new(health_pct)
                                    .text(format!("{:.0}/{:.0}", ship.hull_health, ship.max_hull_health))
                                    .fill(if health_pct > 0.5 { egui::Color32::from_rgb(100, 180, 100) } else { egui::Color32::from_rgb(180, 80, 80) })
                                );
                            });

                            // Cargo summary
                            ui.horizontal(|ui| {
                                ui.label("Cargo:");
                                if let Some(cargo) = &ship.cargo {
                                    ui.label(format!("{}/{}", cargo.total_units(), cargo.capacity));
                                } else {
                                    ui.label("None");
                                }
                            });
                        });
                        ui.add_space(5.0);
                    }
                });
            }
        });
}
