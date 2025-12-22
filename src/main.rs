use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_ecs_tilemap::prelude::*;
use pirates::plugins::core::CorePlugin;
use pirates::plugins::input::InputPlugin;
use pirates::plugins::debug_ui::DebugUiPlugin;
use pirates::plugins::physics::PhysicsPlugin;
use pirates::plugins::combat::CombatPlugin;
use pirates::plugins::worldmap::WorldMapPlugin;
use pirates::plugins::port_ui::PortUiPlugin;
use pirates::plugins::fleet_ui::FleetUiPlugin;
use pirates::plugins::companion::CompanionPlugin;
use pirates::plugins::main_menu::MainMenuPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Pirates".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .add_plugins(TilemapPlugin)
        .add_plugins(CorePlugin)
        .add_plugins(InputPlugin)
        .add_plugins(DebugUiPlugin)
        .add_plugins(PhysicsPlugin)
        .add_plugins(CombatPlugin)
        .add_plugins(WorldMapPlugin)
        .add_plugins(CompanionPlugin)
        .add_plugins(PortUiPlugin)
        .add_plugins(FleetUiPlugin)
        .add_plugins(MainMenuPlugin)
        .run();
}

