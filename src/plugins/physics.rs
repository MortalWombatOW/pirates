use bevy::prelude::*;
use avian2d::prelude::*;
use crate::plugins::core::GameState;
use crate::systems::spawn_player_ship;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default())
            .insert_resource(Time::<Fixed>::from_hz(60.0))
            .insert_resource(Gravity(Vec2::ZERO))
            .add_systems(OnEnter(GameState::Combat), spawn_player_ship);
    }
}
