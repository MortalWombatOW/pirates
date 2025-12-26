use bevy::prelude::*;
use avian2d::prelude::*;

use crate::components::{Ship, Player, Health, Cargo, Gold, AI, Faction, FactionId, CombatEntity};

/// Spawns the player's ship with all required components.
/// This function is designed to be called from an `OnEnter(GameState::Combat)` system.
pub fn spawn_player_ship(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    println!("Spawning player ship at (0, 0)...");
    
    let texture_handle: Handle<Image> = asset_server.load("sprites/ships/player.png");
    
    // Spawn in groups to avoid Bevy's tuple size limit (15 elements max)
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
        // Kenney sprites face DOWN (Y-), so we flip vertically to align with physics forward (Y+)
        Sprite {
            image: texture_handle,
            custom_size: Some(Vec2::splat(64.0)),
            color: Color::WHITE,
            flip_y: true,  // Flip sprite instead of rotating Transform
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
        CombatEntity,
    ))
    .insert((
        // Physics rigid body
        RigidBody::Dynamic,
        Collider::rectangle(48.0, 64.0),
        // Explicit mass properties
        Mass(1000.0), // 1 metric ton
        AngularInertia(20000.0), // Higher inertia makes it harder to spin/stop spinning
    ))
    .insert((
        // Physics velocities
        LinearVelocity(Vec2::ZERO),
        AngularVelocity(0.0),
    ))
    .insert((
        // External forces (controlled by movement system)
        ExternalForce::default(),
        ExternalTorque::default(),
    ))
    .insert((
        // Water resistance (damping)
        // LinearDamping is set to 0.0 because we handle directional drag 
        // manually in ship_physics_system to simulate the keel effect.
        LinearDamping(0.0),
        AngularDamping(2.5),
    ));
    
    println!("Player ship spawned!");
}

/// Spawns an AI-controlled enemy ship at the given position.
/// Returns the Entity ID of the spawned ship.
pub fn spawn_enemy_ship(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec2,
    faction: FactionId,
) -> Entity {
    info!("Spawning enemy ship at ({}, {})...", position.x, position.y);
    
    let texture_handle: Handle<Image> = asset_server.load("sprites/ships/enemy.png");
    
    commands.spawn((
        Name::new("Enemy Ship"),
        // Marker components
        Ship,
        AI,
        Faction(faction),
        // Data components
        Health::default(),
        // Visual components
        Sprite {
            image: texture_handle,
            custom_size: Some(Vec2::splat(64.0)),
            flip_y: true,  // Kenney sprites face DOWN (Y-)
            ..default()
        },
        Transform::from_xyz(position.x, position.y, 1.0),
        CombatEntity,
    ))
    .insert((
        // Physics rigid body
        RigidBody::Dynamic,
        Collider::rectangle(48.0, 64.0),
        // Explicit mass properties - same as player
        Mass(1000.0),
        AngularInertia(20000.0),
    ))
    .insert((
        // Physics velocities
        LinearVelocity(Vec2::ZERO),
        AngularVelocity(0.0),
    ))
    .insert((
        // External forces (controlled by AI system)
        ExternalForce::default(),
        ExternalTorque::default(),
    ))
    .insert((
        // Water resistance (damping)
        LinearDamping(0.0),
        AngularDamping(2.5),
    ))
    .id()
}
