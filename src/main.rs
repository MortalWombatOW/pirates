use bevy::prelude::*;
use pirates::plugins::core::CorePlugin;

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
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    println!("Pirates game started!");
}
