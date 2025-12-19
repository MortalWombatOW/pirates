use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use pirates::plugins::core::CorePlugin;
use pirates::plugins::input::InputPlugin;
use pirates::plugins::debug_ui::DebugUiPlugin;
use pirates::plugins::physics::PhysicsPlugin;
use pirates::plugins::combat::CombatPlugin;

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
        .add_plugins(CorePlugin)
        .add_plugins(InputPlugin)
        .add_plugins(DebugUiPlugin)
        .add_plugins(PhysicsPlugin)
        .add_plugins(CombatPlugin)
        .run();
}
