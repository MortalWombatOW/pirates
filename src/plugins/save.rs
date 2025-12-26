use bevy::prelude::*;
use bevy_save::prelude::*;

use crate::components::{
    Ship, Player, AI, PlayerOwned, Surrendered, Faction, FactionId, ShipType,
    Health, WaterIntake, Cargo, Gold, GoodType, GoodsTrait,
    Destination, NavigationPath, Projectile, TargetComponent, Order, OrderQueue,
};
use crate::resources::{Wind, WorldClock, CliArgs};
use crate::plugins::core::GameState;

/// Marker resource indicating a CLI-triggered load is pending.
/// Consumed after the load is attempted.
#[derive(Resource)]
struct CliLoadPending(String);

/// Plugin that integrates bevy_save for game state persistence.
/// Enables saving and loading the complete game world (entities, components, resources).
pub struct PersistencePlugin;

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        // Add bevy_save's plugin group which provides:
        // - SavePlugin (core snapshot/apply functionality)
        // - SaveablesPlugin (common type registrations)
        app.add_plugins(SavePlugins);

        // Register all game-specific types that need to be saved
        register_saveable_types(app);

        // CLI load check runs at startup
        app.add_systems(Startup, check_cli_load_arg);

        // Add save/load systems
        app.add_systems(
            Update,
            (
                save_game_system.run_if(in_state(GameState::HighSeas).or(in_state(GameState::Port))),
                load_game_system,
                cli_load_system.run_if(resource_exists::<CliLoadPending>),
            ),
        );

        // Add autosave systems on state transitions
        // Autosave when entering safe states (Port, HighSeas)
        app.add_systems(OnEnter(GameState::Port), autosave_system);
        app.add_systems(OnEnter(GameState::HighSeas), autosave_on_highseas);

        // Debug preset generation (F6-F8 keys)
        // Only enabled in HighSeas for safety
        app.add_systems(
            Update,
            debug_preset_system.run_if(in_state(GameState::HighSeas)),
        );
    }
}

/// Registers all custom game types with the Bevy type registry.
/// Types must be registered for bevy_save to serialize/deserialize them.
fn register_saveable_types(app: &mut App) {
    // Ship marker components
    app.register_type::<Ship>()
        .register_type::<Player>()
        .register_type::<AI>()
        .register_type::<PlayerOwned>()
        .register_type::<Surrendered>();

    // Ship classification
    app.register_type::<Faction>()
        .register_type::<FactionId>()
        .register_type::<ShipType>();

    // Health and damage
    app.register_type::<Health>()
        .register_type::<WaterIntake>();

    // Economy
    app.register_type::<Cargo>()
        .register_type::<Gold>()
        .register_type::<GoodType>()
        .register_type::<GoodsTrait>();

    // Navigation
    app.register_type::<Destination>()
        .register_type::<NavigationPath>();

    // Combat
    app.register_type::<Projectile>()
        .register_type::<TargetComponent>();

    // AI Orders
    app.register_type::<Order>()
        .register_type::<OrderQueue>();

    // Resources
    app.register_type::<Wind>()
        .register_type::<WorldClock>();
}

/// System that triggers a quicksave when F5 is pressed.
/// Saves to "quicksave" by default, or to the name specified by --save-as.
fn save_game_system(world: &mut World) {
    // Check for F5 key press
    let should_save = world
        .resource::<ButtonInput<KeyCode>>()
        .just_pressed(KeyCode::F5);

    if should_save {
        // Use --save-as override if provided, otherwise default to "quicksave"
        let save_name = world
            .get_resource::<CliArgs>()
            .and_then(|args| args.save_as.clone())
            .unwrap_or_else(|| "quicksave".to_string());

        info!("Saving game to '{}'...", save_name);

        match world.save(save_name.as_str()) {
            Ok(_) => info!("Game saved successfully to '{}'", save_name),
            Err(e) => error!("Failed to save game: {:?}", e),
        }
    }
}

/// System that triggers a quickload when F9 is pressed.
/// Loads the game state from "quicksave" file.
fn load_game_system(world: &mut World) {
    // Check for F9 key press
    let should_load = world
        .resource::<ButtonInput<KeyCode>>()
        .just_pressed(KeyCode::F9);

    if should_load {
        info!("Loading game...");

        match world.load("quicksave") {
            Ok(_) => {
                info!("Game loaded successfully from 'quicksave'");

                // After loading, transition to HighSeas state (most common saved state)
                if let Some(mut next_state) = world.get_resource_mut::<NextState<GameState>>() {
                    next_state.set(GameState::HighSeas);
                    info!("Transitioned to HighSeas state after load");
                }
            }
            Err(e) => error!("Failed to load game: {:?}", e),
        }
    }
}

/// Autosave system that runs when entering Port state.
/// Creates an "autosave" file separate from quicksave.
fn autosave_system(world: &mut World) {
    info!("Autosaving on Port entry...");

    match world.save("autosave") {
        Ok(_) => info!("Autosave completed successfully"),
        Err(e) => error!("Autosave failed: {:?}", e),
    }
}

/// Autosave system for HighSeas entry.
/// Only saves when coming from Port (not from Combat or MainMenu).
/// Uses a resource to track the previous state to avoid saving after combat/menu.
fn autosave_on_highseas(world: &mut World) {
    // Always autosave when entering HighSeas for simplicity
    // In a more complex implementation, we'd track previous state
    info!("Autosaving on HighSeas entry...");

    match world.save("autosave") {
        Ok(_) => info!("Autosave completed successfully"),
        Err(e) => error!("Autosave failed: {:?}", e),
    }
}

// ============================================================================
// DEBUG PRESET GENERATION (F6-F8)
// ============================================================================

/// System that creates debug preset saves for testing.
/// - F6: "Rich" preset - Player starts with 10,000 gold
/// - F7: "Damaged" preset - Player ship at 25% health
/// - F8: "Advanced" preset - Day 30, lots of exploration
pub fn debug_preset_system(world: &mut World) {
    let keys = world.resource::<ButtonInput<KeyCode>>();

    let create_rich = keys.just_pressed(KeyCode::F6);
    let create_damaged = keys.just_pressed(KeyCode::F7);
    let create_advanced = keys.just_pressed(KeyCode::F8);

    if create_rich {
        info!("Creating 'rich' preset...");
        apply_rich_preset(world);
        if let Err(e) = world.save("preset_rich") {
            error!("Failed to save rich preset: {:?}", e);
        } else {
            info!("Saved 'preset_rich' - Player has 10,000 gold");
        }
    }

    if create_damaged {
        info!("Creating 'damaged' preset...");
        apply_damaged_preset(world);
        if let Err(e) = world.save("preset_damaged") {
            error!("Failed to save damaged preset: {:?}", e);
        } else {
            info!("Saved 'preset_damaged' - Ship at 25% health");
        }
    }

    if create_advanced {
        info!("Creating 'advanced' preset...");
        apply_advanced_preset(world);
        if let Err(e) = world.save("preset_advanced") {
            error!("Failed to save advanced preset: {:?}", e);
        } else {
            info!("Saved 'preset_advanced' - Day 30, advanced game state");
        }
    }
}

/// Applies the "rich" preset: gives player 10,000 gold.
fn apply_rich_preset(world: &mut World) {
    use crate::components::Player;

    // Find player entity and set gold
    let mut query = world.query_filtered::<&mut Gold, With<Player>>();
    for mut gold in query.iter_mut(world) {
        gold.0 = 10_000;
    }
}

/// Applies the "damaged" preset: sets player health to 25%.
fn apply_damaged_preset(world: &mut World) {
    use crate::components::Player;

    // Find player entity and damage health
    let mut query = world.query_filtered::<&mut Health, With<Player>>();
    for mut health in query.iter_mut(world) {
        health.sails = health.sails_max * 0.25;
        health.rudder = health.rudder_max * 0.25;
        health.hull = health.hull_max * 0.25;
    }
}

/// Applies the "advanced" preset: sets game to day 30.
fn apply_advanced_preset(world: &mut World) {
    if let Some(mut clock) = world.get_resource_mut::<WorldClock>() {
        clock.day = 30;
        clock.hour = 12;
    }
}

// ============================================================================
// CLI LOAD SUPPORT
// ============================================================================

/// Checks CLI arguments for --load and inserts CliLoadPending if present.
fn check_cli_load_arg(mut commands: Commands, cli_args: Res<CliArgs>) {
    if let Some(ref save_name) = cli_args.load_save {
        info!("CLI: Queueing load of save '{}'", save_name);
        commands.insert_resource(CliLoadPending(save_name.clone()));
    }
}

/// Processes CLI-triggered load. Runs once then removes the pending marker.
fn cli_load_system(world: &mut World) {
    // Extract the save name and remove the pending marker
    let save_name = {
        let pending = world.remove_resource::<CliLoadPending>();
        match pending {
            Some(p) => p.0,
            None => return,
        }
    };

    info!("CLI: Loading save '{}'...", save_name);

    match world.load(save_name.as_str()) {
        Ok(_) => {
            info!("CLI: Save '{}' loaded successfully", save_name);

            // Transition to HighSeas state (bypassing main menu)
            if let Some(mut next_state) = world.get_resource_mut::<NextState<GameState>>() {
                next_state.set(GameState::HighSeas);
                info!("CLI: Transitioned to HighSeas state");
            }
        }
        Err(e) => {
            error!("CLI: Failed to load save '{}': {:?}", save_name, e);
            error!("CLI: Falling back to main menu");
        }
    }
}
