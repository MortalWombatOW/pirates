use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_egui::EguiPlugin;
use bevy_ecs_tilemap::prelude::*;
use bevy_hanabi::prelude::*;
use pirates::plugins::core::CorePlugin;
use pirates::plugins::input::InputPlugin;
use pirates::plugins::debug_ui::DebugUiPlugin;
use pirates::plugins::physics::PhysicsPlugin;
use pirates::plugins::combat::CombatPlugin;
use pirates::plugins::worldmap::WorldMapPlugin;
use pirates::plugins::port::PortPlugin;
use pirates::plugins::port_ui::PortUiPlugin;
use pirates::plugins::fleet_ui::FleetUiPlugin;
use pirates::plugins::companion::CompanionPlugin;
use pirates::plugins::main_menu::MainMenuPlugin;
use pirates::plugins::save::PersistencePlugin;
use pirates::plugins::compass_rose::CompassRosePlugin;
use pirates::plugins::scale_bar::ScaleBarPlugin;
use pirates::plugins::overlay_ui::OverlayUiPlugin;
use pirates::plugins::cartouche::CartouchePlugin;
use pirates::plugins::fade_controller::FadeControllerPlugin;
use pirates::systems::damage_effects::{
    setup_splatter_effects, spawn_damage_splatter,
};
use pirates::plugins::core::GameState;
use pirates::resources::CliArgs;

fn main() {
    // Parse CLI arguments before building the app
    let cli_args = CliArgs::parse();

    App::new()
        .insert_resource(cli_args)
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(TilemapPlugin)
        .add_plugins(EguiPlugin)
        .add_plugins(HanabiPlugin)
        .add_plugins(pirates::plugins::ui_theme::UiThemePlugin)
        .add_plugins(CorePlugin)
        .add_plugins(InputPlugin)
        .add_plugins(DebugUiPlugin)
        .add_plugins(PhysicsPlugin)
        .add_plugins(CombatPlugin)
        .add_plugins(WorldMapPlugin)
        .add_plugins(PortPlugin)
        .add_plugins(PortUiPlugin)
        .add_plugins(FleetUiPlugin)
        .add_plugins(CompanionPlugin)
        .add_plugins(MainMenuPlugin)
        .add_plugins(PersistencePlugin)
        .add_plugins(OverlayUiPlugin)
        .add_plugins(CompassRosePlugin)
        .add_plugins(ScaleBarPlugin)
        .add_plugins(CartouchePlugin)
        .add_plugins(FadeControllerPlugin)
        // .add_plugins(GraphicsPlugin) // Disabled to test raw rendering
        // Particle effect systems (8.5) - Damage splatter remains, wake effects removed (now fluid sim)
        .add_systems(Startup, setup_splatter_effects)
        .add_systems(
            Update,
            spawn_damage_splatter.run_if(in_state(GameState::HighSeas).or(in_state(GameState::Combat))),
        )
        .run();
}

