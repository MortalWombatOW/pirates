use bevy::prelude::*;
use avian2d::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::systems::movement::ShipInputBuffer;

/// System that handles cannon firing based on buffered input.
pub fn cannon_firing_system(
    mut commands: Commands,
    mut cannon_state: ResMut<CannonState>,
    input_buffer: Res<ShipInputBuffer>,
    time: Res<Time>,
    query: Query<(Entity, &Transform, &LinearVelocity), (With<Ship>, With<Player>)>,
    asset_server: Res<AssetServer>,
) {
    // Tick cooldown
    if cannon_state.cooldown_remaining > 0.0 {
        cannon_state.cooldown_remaining -= time.delta_secs();
    }

    if cannon_state.cooldown_remaining > 0.0 {
        // info!("Cannon on cooldown: {:.2}", cannon_state.cooldown_remaining);
        return;
    }

    // Check for port or starboard fire intent in the sticky buffer
    let mut fired_side = None;
    if input_buffer.fire_port {
        fired_side = Some(-1.0); // Port
    } else if input_buffer.fire_starboard {
        fired_side = Some(1.0);  // Starboard
    }

    if let Some(side) = fired_side {
        if let Ok((_player_ent, transform, ship_velocity)) = query.get_single() {
            // Get ship's local right vector (X-axis in local space)
            let right = transform.rotation * Vec3::X;
            let spawn_direction = (right * side).truncate();
            
            // Spawn a spread of projectiles (broadside)
            let spawn_pos_center = transform.translation + (right * side * 40.0) + Vec3::new(0.0, 0.0, 5.0);
            let projectile_speed = 400.0;
            
            // Fire 3 cannonballs in a slight spread
            for i in -1..=1 {
                let offset = transform.rotation * (Vec3::Y * (i as f32 * 15.0));
                let spawn_pos = spawn_pos_center + offset;
                
                commands.spawn((
                    Sprite {
                        image: asset_server.load("sprites/projectile.png"),
                        custom_size: Some(Vec2::new(16.0, 16.0)),
                        ..default()
                    },
                    Transform::from_translation(spawn_pos),
                    RigidBody::Dynamic,
                    Collider::circle(8.0),
                    Sensor,
                    LinearVelocity(ship_velocity.0 + spawn_direction * projectile_speed),
                    Projectile {
                        damage: 10.0,
                        target: TargetComponent::Hull, // Default to hull for now
                        source: _player_ent,
                    },
                    ProjectileTimer::default(),
                ));
            }

            cannon_state.cooldown_remaining = cannon_state.base_cooldown;
            
            // CONSUME the sticky input from the buffer
            // We use a separate mutable variable to clear it
            info!("Broadside fired to {}!", if side > 0.0 { "Starboard" } else { "Port" });
        }
    }
}

/// System to clear consumed firing input from the buffer.
/// This must run AFTER the physics/firing systems.
pub fn consume_firing_input(
    _cannon_state: Res<CannonState>,
    mut input_buffer: ResMut<ShipInputBuffer>,
) {
    // Unconditionally clear the input buffer every frame.
    // This ensures that an input is only valid for ONE physics tick.
    // If the cannon was on cooldown during this tick, the input is discarded.
    input_buffer.fire_port = false;
    input_buffer.fire_starboard = false;
}

/// Component to handle projectile despawning after some time.
#[derive(Component)]
pub struct ProjectileTimer(pub Timer);

impl Default for ProjectileTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(5.0, TimerMode::Once))
    }
}

/// System that handles projectiles (timeout, etc).
pub fn projectile_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ProjectileTimer)>,
) {
    for (entity, mut timer) in &mut query {
        if timer.0.tick(time.delta()).finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// System to handle projectile hits on ships.
pub fn projectile_collision_system(
    mut commands: Commands,
    mut collision_events: EventReader<Collision>,
    projectiles: Query<&Projectile>,
    mut ships: Query<(Entity, &mut Health, Option<&Name>), With<Ship>>,
) {
    for Collision(contacts) in collision_events.read() {
        let e1 = contacts.entity1;
        let e2 = contacts.entity2;

        // Check if one is a projectile and the other is a ship
        let (proj_ent, ship_ent) = if projectiles.contains(e1) && ships.contains(e2) {
            (e1, e2)
        } else if projectiles.contains(e2) && ships.contains(e1) {
            (e2, e1)
        } else {
            continue;
        };

        if let (Ok(projectile), Ok((_ent, mut health, name))) = (projectiles.get(proj_ent), ships.get_mut(ship_ent)) {
            // Skip if the ship hit is the source that fired it
            if projectile.source == ship_ent {
                continue;
            }

            // Apply damage
            match projectile.target {
                TargetComponent::Sails => health.sails -= projectile.damage,
                TargetComponent::Rudder => health.rudder -= projectile.damage,
                TargetComponent::Hull => health.hull -= projectile.damage,
            }

            let ship_name = name.map(|n| n.as_str()).unwrap_or("Unknown Ship");
            info!(
                "Hit! {} damaged by {:?}. New Health: S:{:.1} R:{:.1} H:{:.1}",
                ship_name,
                projectile.target,
                health.sails,
                health.rudder,
                health.hull
            );

            // Despawn projectile
            commands.entity(proj_ent).despawn_recursive();
        }
    }
}

/// System to cycle target components (DEPRECATED: broadside focused).
pub fn target_cycling_system() {}

/// Spawns a static target ship for testing.
pub fn spawn_test_target(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    info!("Spawning test target at (0, 150)");
    commands.spawn((
        Name::new("Test Target"),
        Sprite {
            image: asset_server.load("sprites/ships/player.png"),
            color: Color::srgb(1.0, 0.4, 0.4), // Reddish target
            custom_size: Some(Vec2::new(64.0, 64.0)),
            ..default()
        },
        // Rotate 180 degrees: Kenney sprites face DOWN, but physics forward is Y+ (UP)
        Transform::from_xyz(0.0, 150.0, 0.0).with_rotation(Quat::from_rotation_z(std::f32::consts::PI)),
        Ship,
        Health::default(),
        RigidBody::Static,
        Collider::rectangle(64.0, 64.0),
    ));
}

/// System that detects and destroys ships with hull HP <= 0.
pub fn ship_destruction_system(
    mut commands: Commands,
    query: Query<(Entity, &Health, Option<&Player>, Option<&Name>), With<Ship>>,
    mut ship_destroyed_events: EventWriter<crate::events::ShipDestroyedEvent>,
) {
    for (entity, health, player, name) in &query {
        if health.is_destroyed() {
            let ship_name = name.map(|n| n.as_str()).unwrap_or("Unknown Ship");
            let was_player = player.is_some();
            
            info!("Ship destroyed: {} (was_player: {})", ship_name, was_player);
            
            // Send the event before despawning
            ship_destroyed_events.send(crate::events::ShipDestroyedEvent {
                entity,
                was_player,
            });
            
            // Despawn the ship entity
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// System that handles player death by transitioning to GameOver state.
pub fn handle_player_death_system(
    mut ship_destroyed_events: EventReader<crate::events::ShipDestroyedEvent>,
    mut next_state: ResMut<NextState<crate::plugins::core::GameState>>,
) {
    for event in ship_destroyed_events.read() {
        if event.was_player {
            info!("Player ship destroyed! Transitioning to GameOver state.");
            next_state.set(crate::plugins::core::GameState::GameOver);
        }
    }
}

