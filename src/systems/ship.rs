use bevy::prelude::*;
use avian2d::prelude::*;

use crate::components::{Ship, Player, Health, Cargo, Gold};

/// Spawns the player's ship with all required components.
/// This function is designed to be called from an `OnEnter(GameState::Combat)` system.
pub fn spawn_player_ship(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    println!("Spawning player ship at (0, 0)...");
    
    // Try to load the sprite, fall back to colored sprite if not available
    let texture_handle: Handle<Image> = asset_server.load("sprites/ships/player.png");
    
    commands.spawn((
        Name::new("Player Ship"),
        // Marker components
        Ship,
        Player,
        // Data components
        Health::default(),
        Cargo::new(100),
        Gold(100),
        // Visual components
        Sprite {
            image: texture_handle,
            custom_size: Some(Vec2::splat(64.0)),
            color: Color::WHITE,
            ..default()
        },
        // Physics components
        Transform::from_xyz(0.0, 0.0, 1.0), // Z=1 to render above background
        RigidBody::Dynamic,
        Collider::rectangle(48.0, 64.0), // Ship-shaped collider
        LinearVelocity(Vec2::ZERO),
        AngularVelocity(0.0),
        ExternalForce::default(),
        // Linear and angular damping to simulate water resistance
        LinearDamping(1.0),
        AngularDamping(2.0),
    ));
    
    println!("Player ship spawned!");
}
