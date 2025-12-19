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
        Sprite {
            image: texture_handle,
            custom_size: Some(Vec2::splat(64.0)),
            color: Color::WHITE,
            ..default()
        },
        // Rotate 180 degrees: Kenney sprites face DOWN, but physics forward is Y+ (UP)
        Transform::from_xyz(0.0, 0.0, 1.0).with_rotation(Quat::from_rotation_z(std::f32::consts::PI)),
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

