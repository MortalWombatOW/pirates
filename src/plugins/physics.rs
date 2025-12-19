use bevy::prelude::*;
use avian2d::prelude::*;
use crate::plugins::core::GameState;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default())
            .insert_resource(Time::<Fixed>::from_hz(60.0))
            .insert_resource(Gravity(Vec2::ZERO))
            .add_systems(OnEnter(GameState::Combat), spawn_test_physics_entity);
    }
}

fn spawn_test_physics_entity(mut commands: Commands) {
    println!("Spawning test physics entity at (0, 0, 0)...");
    commands.spawn((
        Name::new("Test Physics Entity"),
        Sprite {
            color: Color::srgb(1.0, 0.0, 0.0),
            custom_size: Some(Vec2::splat(64.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        RigidBody::Dynamic,
        Collider::circle(32.0),
        LinearVelocity(Vec2::ZERO),
        AngularVelocity(0.5),
    ));
    println!("Test physics entity spawned!");
}
