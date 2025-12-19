use bevy::prelude::*;
use pirates::plugins::core::CorePlugin;
use pirates::plugins::input::InputPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Pirates".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(CorePlugin)
        .add_plugins(InputPlugin)
        .run();
}
