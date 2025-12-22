//! Main Menu UI plugin.
//!
//! Displays archetype selection and game start options.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::plugins::core::GameState;
use crate::resources::{ArchetypeId, ArchetypeRegistry, MetaProfile, UnlockCondition};

/// Plugin for the Main Menu UI.
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedArchetype>()
            .add_systems(
                Update,
                main_menu_ui_system.run_if(in_state(GameState::MainMenu)),
            );
    }
}

/// Resource storing the player's selected starting archetype.
#[derive(Resource, Debug)]
pub struct SelectedArchetype(pub ArchetypeId);

impl Default for SelectedArchetype {
    fn default() -> Self {
        Self(ArchetypeId::Default)
    }
}

/// Renders the main menu with archetype selection.
fn main_menu_ui_system(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    mut selected: ResMut<SelectedArchetype>,
    registry: Res<ArchetypeRegistry>,
    profile: Res<MetaProfile>,
) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);

            // Title
            ui.heading(egui::RichText::new("PIRATES").size(48.0).strong());
            ui.add_space(10.0);
            ui.label(egui::RichText::new("A Naval Roguelike").size(16.0).italics());

            ui.add_space(40.0);
            ui.separator();
            ui.add_space(20.0);

            // Archetype Selection Header
            ui.heading("Choose Your Captain");
            ui.add_space(10.0);
            ui.label(format!(
                "Runs completed: {} | Deaths: {} | Lifetime gold: {}",
                profile.runs_completed, profile.deaths, profile.lifetime_gold
            ));
            ui.add_space(20.0);

            // Archetype Grid
            egui::Grid::new("archetype_grid")
                .num_columns(2)
                .spacing([20.0, 15.0])
                .show(ui, |ui| {
                    for &archetype_id in ArchetypeId::all() {
                        let Some(config) = registry.get(archetype_id) else {
                            continue;
                        };

                        let is_unlocked = profile.unlocked_archetypes.contains(&archetype_id);
                        let is_selected = selected.0 == archetype_id;

                        // Archetype card
                        let card_response = ui.add_enabled(
                            is_unlocked,
                            egui::Button::new(
                                egui::RichText::new(config.name)
                                    .size(18.0)
                                    .strong()
                                    .color(if is_selected {
                                        egui::Color32::GOLD
                                    } else if is_unlocked {
                                        egui::Color32::WHITE
                                    } else {
                                        egui::Color32::DARK_GRAY
                                    }),
                            )
                            .min_size(egui::vec2(180.0, 40.0))
                            .fill(if is_selected {
                                egui::Color32::from_rgb(60, 40, 20)
                            } else {
                                egui::Color32::from_rgb(30, 30, 40)
                            }),
                        );

                        if card_response.clicked() && is_unlocked {
                            selected.0 = archetype_id;
                        }

                        // Description column
                        ui.vertical(|ui| {
                            if is_unlocked {
                                ui.label(config.description);
                                ui.label(format!(
                                    "Start: {} gold, {}",
                                    config.starting_gold,
                                    format_ship_type(config.ship_type)
                                ));
                            } else {
                                ui.label(
                                    egui::RichText::new("🔒 Locked")
                                        .color(egui::Color32::DARK_GRAY),
                                );
                                ui.label(
                                    egui::RichText::new(format_unlock_condition(
                                        &config.unlock_condition,
                                    ))
                                    .color(egui::Color32::GRAY)
                                    .small(),
                                );
                            }
                        });

                        ui.end_row();
                    }
                });

            ui.add_space(40.0);

            // Start Button
            let start_button = ui.add(
                egui::Button::new(
                    egui::RichText::new("⛵ Set Sail")
                        .size(24.0)
                        .strong()
                        .color(egui::Color32::WHITE),
                )
                .min_size(egui::vec2(200.0, 50.0))
                .fill(egui::Color32::from_rgb(40, 80, 120)),
            );

            if start_button.clicked() {
                info!("Starting new game with archetype: {:?}", selected.0);
                next_state.set(GameState::HighSeas);
            }

            ui.add_space(20.0);

            // Selected archetype info
            if let Some(config) = registry.get(selected.0) {
                ui.separator();
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new(format!("Selected: {}", config.name))
                        .size(14.0)
                        .color(egui::Color32::GOLD),
                );
            }
        });
    });
}

/// Formats a ShipType for display.
fn format_ship_type(ship_type: crate::components::ship::ShipType) -> &'static str {
    use crate::components::ship::ShipType;
    match ship_type {
        ShipType::Sloop => "Sloop",
        ShipType::Frigate => "Frigate",
        ShipType::Schooner => "Schooner",
        ShipType::Raft => "Raft",
    }
}

/// Formats an UnlockCondition for display.
fn format_unlock_condition(condition: &UnlockCondition) -> String {
    match condition {
        UnlockCondition::AlwaysUnlocked => "Always available".to_string(),
        UnlockCondition::RunsCompleted(n) => format!("Complete {} runs", n),
        UnlockCondition::LifetimeGold(n) => format!("Earn {} lifetime gold", n),
        UnlockCondition::QuickDeath(hours) => {
            format!("Die within {} hours of starting", hours)
        }
    }
}
