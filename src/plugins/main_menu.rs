//! Main Menu UI plugin.
//!
//! Displays archetype selection and game start options.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_save::prelude::*;

use crate::plugins::core::GameState;
use crate::resources::{ArchetypeId, ArchetypeRegistry, MetaProfile, UnlockCondition};
use crate::resources::ui_assets::UiAssets;

/// Plugin for the Main Menu UI.
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedArchetype>()
            .init_resource::<SaveFileExists>()
            .add_event::<LoadGameEvent>()
            .add_systems(Startup, check_save_file_exists)
            .add_systems(
                Update,
                (
                    main_menu_ui_system.run_if(in_state(GameState::MainMenu)),
                    handle_load_game_event,
                ),
            );
    }
}

/// Event triggered when the player clicks "Continue".
#[derive(Event)]
pub struct LoadGameEvent;

/// Resource tracking whether a save file exists.
#[derive(Resource, Default)]
pub struct SaveFileExists(pub bool);

/// Resource storing the player's selected starting archetype.
#[derive(Resource, Debug)]
pub struct SelectedArchetype(pub ArchetypeId);

impl Default for SelectedArchetype {
    fn default() -> Self {
        Self(ArchetypeId::Default)
    }
}

/// Checks if an autosave file exists at startup.
fn check_save_file_exists(mut save_exists: ResMut<SaveFileExists>) {
    // Check for autosave file in platform-specific save directory
    if let Some(data_dir) = dirs::data_dir() {
        let save_path = data_dir.join("pirates").join("saves").join("autosave.sav");
        save_exists.0 = save_path.exists();
        if save_exists.0 {
            info!("Found existing save file at {:?}", save_path);
        }
    }
}

/// Handles the LoadGameEvent by loading the autosave.
fn handle_load_game_event(world: &mut World) {
    // Check if there are any load events
    let has_event = world
        .resource_mut::<Events<LoadGameEvent>>()
        .drain()
        .next()
        .is_some();

    if has_event {
        info!("Loading autosave from main menu...");

        match world.load("autosave") {
            Ok(_) => {
                info!("Autosave loaded successfully");
                if let Some(mut next_state) = world.get_resource_mut::<NextState<GameState>>() {
                    next_state.set(GameState::HighSeas);
                }
            }
            Err(e) => {
                error!("Failed to load autosave: {:?}", e);
            }
        }
    }
}

/// Renders the main menu with archetype selection.
fn main_menu_ui_system(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    mut selected: ResMut<SelectedArchetype>,
    mut load_events: EventWriter<LoadGameEvent>,
    registry: Res<ArchetypeRegistry>,
    profile: Res<MetaProfile>,
    save_exists: Res<SaveFileExists>,
    ui_assets: Res<UiAssets>,
    time: Res<Time>,
    mut typewriter: Local<crate::components::TypewriterRegistry>,
) {
    // Update typewriter animations
    typewriter.tick_all(time.delta_secs());
    
    let texture_id = contexts.add_image(ui_assets.parchment_texture.clone());

    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        // Draw parchment background
        crate::plugins::ui_theme::draw_parchment_bg(ui, texture_id);

        ui.vertical_centered(|ui| {
            ui.add_space(40.0);

            // Animated Title - writes on character by character
            let title = typewriter.get_or_start("title", "PIRATES", 0.12);
            ui.heading(egui::RichText::new(title.visible_text()).size(48.0).strong());
            ui.add_space(10.0);
            
            // Animated subtitle - starts after title finishes
            let subtitle_text = if title.is_complete() {
                typewriter.get_or_start("subtitle", "A Naval Roguelike", 0.05).visible_text()
            } else {
                ""
            };
            ui.label(egui::RichText::new(subtitle_text).size(16.0).italics());

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

            // Continue Button (only shown if save exists)
            if save_exists.0 {
                let continue_button = ui.add(
                    egui::Button::new(
                        egui::RichText::new("▶ Continue")
                            .size(24.0)
                            .strong()
                            .color(egui::Color32::WHITE),
                    )
                    .min_size(egui::vec2(200.0, 50.0))
                    .fill(egui::Color32::from_rgb(60, 100, 60)),
                );

                if continue_button.clicked() {
                    info!("Loading saved game...");
                    load_events.send(LoadGameEvent);
                }

                ui.add_space(10.0);
            }

            // Start Button (New Game)
            let start_button = ui.add(
                egui::Button::new(
                    egui::RichText::new("⛵ New Voyage")
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
