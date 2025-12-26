use bevy::prelude::*;
use bevy::utils::HashSet;
use avian2d::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::systems::movement::ShipInputBuffer;

/// System that handles cannon firing based on buffered input.
/// Gunner companion provides -30% cannon cooldown.
pub fn cannon_firing_system(
    mut commands: Commands,
    mut cannon_state: ResMut<CannonState>,
    input_buffer: Res<ShipInputBuffer>,
    time: Res<Time>,
    query: Query<(Entity, &Transform, &LinearVelocity), (With<Ship>, With<Player>)>,
    companion_query: Query<&crate::components::companion::CompanionRole>,
    asset_server: Res<AssetServer>,
    mut cannon_fired_events: EventWriter<crate::events::CannonFiredEvent>,
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
                    CombatEntity,
                ));
            }

            // Check if player has a Gunner companion (provides -30% cooldown reduction)
            let has_gunner = companion_query.iter().any(|role| *role == crate::components::companion::CompanionRole::Gunner);
            let gunner_bonus = if has_gunner { 0.7 } else { 1.0 };
            
            cannon_state.cooldown_remaining = cannon_state.base_cooldown * gunner_bonus;
            
            // Emit cannon fired event for screen shake
            cannon_fired_events.send(crate::events::CannonFiredEvent {
                position: transform.translation.truncate(),
                side,
            });
            
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
/// Uses Local HashSet to deduplicate multiple collision events for same projectile.
pub fn projectile_collision_system(
    mut commands: Commands,
    mut collision_events: EventReader<Collision>,
    projectiles: Query<(&Projectile, &Transform)>,
    mut ships: Query<(Entity, &mut Health, &Transform, Option<&Name>, Option<&mut WaterIntake>), With<Ship>>,
    asset_server: Res<AssetServer>,
    mut ship_hit_events: EventWriter<crate::events::ShipHitEvent>,
    mut processed_projectiles: Local<HashSet<Entity>>,
) {
    // Clear the set at start of each frame
    processed_projectiles.clear();
    
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

        // Skip if we've already processed this projectile this frame
        if processed_projectiles.contains(&proj_ent) {
            continue;
        }
        processed_projectiles.insert(proj_ent);

        if let (Ok((projectile, proj_transform)), Ok((entity, mut health, _ship_transform, name, water_intake))) = 
            (projectiles.get(proj_ent), ships.get_mut(ship_ent)) 
        {
            // Skip if the ship hit is the source that fired it
            if projectile.source == ship_ent {
                continue;
            }

            // Apply damage
            match projectile.target {
                TargetComponent::Sails => health.sails -= projectile.damage,
                TargetComponent::Rudder => health.rudder -= projectile.damage,
                TargetComponent::Hull => {
                    health.hull -= projectile.damage;
                    
                    // Add or increase WaterIntake on hull damage
                    let intake_rate_increase = projectile.damage * 0.1; // 0.1 units/sec per damage point
                    if let Some(mut intake) = water_intake {
                        intake.increase_rate(intake_rate_increase);
                        info!("Hull breach worsened! Water intake rate: {:.2}/s", intake.rate);
                    } else {
                        // Add new WaterIntake component
                        commands.entity(entity).insert(WaterIntake::new(intake_rate_increase));
                        info!("Hull breached! Water intake started at {:.2}/s", intake_rate_increase);
                    }
                }
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

            // Emit ShipHitEvent for damage splatter VFX
            let hit_pos = proj_transform.translation.truncate();
            ship_hit_events.send(crate::events::ShipHitEvent {
                ship_entity: ship_ent,
                hit_position: hit_pos,
                damage: projectile.damage,
            });

            // Spawn loot at the projectile impact location
            spawn_loot(&mut commands, &asset_server, hit_pos);

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
        // Kenney sprites face DOWN (Y-), flip to align with physics forward (Y+)
        Sprite {
            image: asset_server.load("sprites/ships/player.png"),
            color: Color::srgb(1.0, 0.4, 0.4), // Reddish target
            custom_size: Some(Vec2::new(64.0, 64.0)),
            flip_y: true,
            ..default()
        },
        Transform::from_xyz(0.0, 150.0, 0.0),
        Ship,
        Health::default(),
        RigidBody::Static,
        Collider::rectangle(64.0, 64.0),
    ));
}

/// System that detects and destroys ships with hull HP <= 0.
/// Captures player death data for legacy wreck creation before despawning.
pub fn ship_destruction_system(
    mut commands: Commands,
    query: Query<(
        Entity,
        &Health,
        Option<&Player>,
        Option<&Name>,
        Option<&Transform>,
        Option<&crate::components::Gold>,
        Option<&crate::components::Cargo>,
    ), With<Ship>>,
    mut ship_destroyed_events: EventWriter<crate::events::ShipDestroyedEvent>,
    mut death_data: ResMut<crate::resources::PlayerDeathData>,
) {
    for (entity, health, player, name, transform, gold, cargo) in &query {
        if health.is_destroyed() {
            let ship_name = name.map(|n| n.as_str()).unwrap_or("Unknown Ship");
            let was_player = player.is_some();

            info!("Ship destroyed: {} (was_player: {})", ship_name, was_player);

            // Capture player death data before despawning
            if was_player {
                death_data.position = transform.map(|t| t.translation.truncate());
                death_data.gold = gold.map(|g| g.0).unwrap_or(0);
                death_data.cargo = cargo
                    .map(|c| {
                        c.goods
                            .iter()
                            .map(|(good_type, qty)| (format!("{:?}", good_type), *qty))
                            .collect()
                    })
                    .unwrap_or_default();
                death_data.ship_name = ship_name.to_string();

                info!(
                    "Captured death data: pos={:?}, gold={}, cargo_items={}",
                    death_data.position,
                    death_data.gold,
                    death_data.cargo.len()
                );
            }

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

/// Helper function to spawn a loot entity at the given position.
fn spawn_loot(commands: &mut Commands, asset_server: &Res<AssetServer>, position: Vec2) {
    let loot_value = 5; // Base gold value per loot drop
    
    commands.spawn((
        Name::new("Loot"),
        Sprite {
            image: asset_server.load("sprites/loot/gold.png"),
            color: Color::srgb(1.0, 0.85, 0.0), // Golden tint
            custom_size: Some(Vec2::new(16.0, 16.0)),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, 1.0),
        RigidBody::Dynamic,
        Collider::circle(8.0),
        Sensor, // Use sensor so loot doesn't physically block ships
        LinearVelocity(Vec2::new(
            rand::random::<f32>() * 40.0 - 20.0,
            rand::random::<f32>() * 40.0 - 20.0,
        )),
        LinearDamping(1.5), // Loot slows down over time
        Loot::gold(loot_value),
        LootTimer::default(),
        CombatEntity,
    ));
    
    info!("Loot spawned at ({:.1}, {:.1})", position.x, position.y);
}

/// System to handle loot collection by the player.
pub fn loot_collection_system(
    mut commands: Commands,
    mut collision_events: EventReader<Collision>,
    loot_query: Query<(Entity, &Loot)>,
    mut player_query: Query<(&mut Gold, Option<&mut Cargo>), With<Player>>,
) {
    for Collision(contacts) in collision_events.read() {
        let e1 = contacts.entity1;
        let e2 = contacts.entity2;
        
        // Check if one is loot and the other is the player
        let (loot_ent, player_ent) = if loot_query.contains(e1) && player_query.contains(e2) {
            (e1, e2)
        } else if loot_query.contains(e2) && player_query.contains(e1) {
            (e2, e1)
        } else {
            continue;
        };
        
        if let (Ok((_, loot)), Ok((mut gold, cargo))) = (loot_query.get(loot_ent), player_query.get_mut(player_ent)) {
            // Add gold value
            gold.add(loot.value);
            info!("Collected loot! +{} gold (Total: {})", loot.value, gold.0);
            
            // If loot has a good type and player has cargo, add to cargo
            if let (Some(good_type), Some(mut cargo)) = (loot.good_type, cargo) {
                let added = cargo.add(good_type, 1);
                if added > 0 {
                    info!("Also collected 1x {:?}", good_type);
                }
            }
            
            // Despawn collected loot
            commands.entity(loot_ent).despawn_recursive();
        }
    }
}

/// System to despawn loot after its timer expires.
pub fn loot_timer_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut LootTimer)>,
) {
    for (entity, mut timer) in &mut query {
        if timer.0.tick(time.delta()).finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// System that applies forces to all RigidBody entities within CurrentZone bounds.
/// Runs in FixedUpdate since it modifies physics forces.
pub fn current_zone_system(
    zone_query: Query<(&CurrentZone, &Transform)>,
    mut body_query: Query<(&Transform, &mut ExternalForce), With<RigidBody>>,
) {
    for (zone, zone_transform) in &zone_query {
        let zone_center = zone_transform.translation.truncate();
        
        for (body_transform, mut force) in &mut body_query {
            let body_pos = body_transform.translation.truncate();
            
            if zone.contains(zone_center, body_pos) {
                // Apply the current's force to this entity
                force.apply_force(zone.velocity);
            }
        }
    }
}

/// Spawns a test current zone for visual debugging.
pub fn spawn_test_current_zone(mut commands: Commands) {
    let zone_pos = Vec2::new(200.0, 0.0);
    let half_extents = Vec2::new(100.0, 150.0);
    // Force needs to be significant relative to ship mass (1000 kg)
    // Ship thrust is 150000, so we use a noticeable fraction of that
    let velocity = Vec2::new(30000.0, 0.0); // Strong rightward push (doubled)
    
    info!("Spawning test current zone at ({}, {}) with force {}", zone_pos.x, zone_pos.y, velocity.x);
    
    commands.spawn((
        Name::new("Test Current Zone"),
        CurrentZone::new(velocity, half_extents),
        Transform::from_xyz(zone_pos.x, zone_pos.y, -1.0), // Below other entities
        Sprite {
            color: Color::srgba(0.2, 0.4, 0.8, 0.3), // Semi-transparent blue
            custom_size: Some(half_extents * 2.0), // Full size, not half-extents
            ..default()
        },
        CombatEntity,
    ));
}

/// System that detects combat victory when all AI ships are destroyed or surrendered.
pub fn combat_victory_system(
    ai_ships: Query<(Entity, Option<&Surrendered>), (With<Ship>, With<AI>)>,
    player_ships: Query<Entity, (With<Ship>, With<Player>)>,
    mut combat_ended_events: EventWriter<crate::events::CombatEndedEvent>,
) {
    // Only check for victory if the player is still alive
    if player_ships.is_empty() {
        return;
    }
    
    // Victory when all AI ships are destroyed OR surrendered
    if ai_ships.is_empty() {
        // Technically this is victory if no enemies existed, but usually we spawn some
        // Let's assume victory if empty to be safe
        info!("No enemies present - Victory!");
        combat_ended_events.send(crate::events::CombatEndedEvent { victory: true });
        return;
    }

    // Check if any active combatants remain
    let active_enemies = ai_ships.iter().filter(|(_, surrendered)| surrendered.is_none()).count();
    
    if active_enemies == 0 {
        info!("All enemies destroyed or surrendered!");
        combat_ended_events.send(crate::events::CombatEndedEvent { victory: true });
    }
}

/// System that handles combat victory by capturing surrendered ships and transitioning state.
pub fn handle_combat_victory_system(
    mut combat_ended_events: EventReader<crate::events::CombatEndedEvent>,
    mut next_state: ResMut<NextState<crate::plugins::core::GameState>>,
    surrendered_ships: Query<(&Health, &Name, Option<&Cargo>), (With<Ship>, With<Surrendered>)>,
    mut player_fleet: ResMut<PlayerFleet>,
) {
    for event in combat_ended_events.read() {
        if event.victory {
            // Process surrendered ships
            for (health, name, cargo) in &surrendered_ships {
                let ship_data = ShipData {
                    sprite_path: "sprites/ships/round_ship_small.png".to_string(), // Todo: preserve actual sprite
                    hull_health: health.hull,
                    max_hull_health: 100.0, // Hardcoded for now, should read from component
                    cargo: cargo.cloned(),
                    name: name.as_str().to_string(),
                };
                
                info!("Captured ship: {}", ship_data.name);
                player_fleet.ships.push(ship_data);
            }

            info!("Combat victory! Transitioning to HighSeas state.");
            next_state.set(crate::plugins::core::GameState::HighSeas);
        }
    }
}
