use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy_ecs_tilemap::prelude::*;
use crate::plugins::core::GameState;
use crate::plugins::port::{spawn_port, generate_port_name};
use crate::resources::{MapData, FogOfWar, RouteCache};
use crate::components::{Player, Ship, Health, Vision, AI, Faction, FactionId, Order, OrderQueue};
use crate::systems::{
    fog_of_war_update_system, update_fog_tilemap_system, FogTile,
    click_to_navigate_system, pathfinding_system, navigation_movement_system,
    path_visualization_system, port_arrival_system, order_execution_system,
    ai_pathfinding_system, ai_movement_system, contract_delegation_system,
    intel_visualization_system,
};
use crate::utils::pathfinding::{tile_to_world, world_to_tile};
use crate::utils::spatial_hash::SpatialHash;
use crate::events::CombatTriggeredEvent;

/// Plugin managing the world map tilemap for the High Seas view.
/// 
/// This plugin handles:
/// - Creating a procedural tileset texture
/// - Spawning the tilemap entity and tiles from MapData
/// - Managing map visibility based on game state
pub struct WorldMapPlugin;

impl Plugin for WorldMapPlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize resources
            .init_resource::<MapData>()
            .init_resource::<FogOfWar>()
            .init_resource::<RouteCache>()
            .init_resource::<EncounterSpatialHash>()
            .init_resource::<EncounterCooldown>()
            .init_resource::<EncounteredEnemy>()
            .init_resource::<crate::resources::PlayerFleet>()
            .init_resource::<crate::resources::FleetEntities>()
            .add_event::<CombatTriggeredEvent>()
            .add_systems(Startup, (generate_procedural_map, create_tileset_texture))
            .add_systems(OnEnter(GameState::HighSeas), (
                spawn_tilemap_from_map_data, 
                spawn_high_seas_player, 
                spawn_high_seas_ai_ships,
                spawn_player_fleet,
                spawn_port_entities,
                reset_encounter_cooldown,
            ))
            .add_systems(Update, (
                fog_of_war_update_system,
                update_fog_tilemap_system,
                fog_of_war_ai_visibility_system,
                rebuild_encounter_spatial_hash,
                encounter_detection_system.after(rebuild_encounter_spatial_hash),
                handle_combat_trigger_system.after(encounter_detection_system),
                click_to_navigate_system,
                order_execution_system,
                pathfinding_system.after(click_to_navigate_system).after(order_execution_system),
                ai_pathfinding_system.after(order_execution_system),
                navigation_movement_system.after(pathfinding_system),
                ai_movement_system.after(ai_pathfinding_system),
                path_visualization_system,
                intel_visualization_system,
                port_arrival_system,
                contract_delegation_system,
            ).run_if(in_state(GameState::HighSeas)))
            .add_systems(OnExit(GameState::HighSeas), (despawn_high_seas_player, despawn_high_seas_ai_ships, despawn_port_entities, clear_fleet_entities));
    }
}

/// Marker component for the world map tilemap
#[derive(Component)]
pub struct WorldMap;

/// Marker component for world map tiles
#[derive(Component)]
pub struct WorldMapTile;

/// Marker component for the fog tilemap
#[derive(Component)]
pub struct FogMap;

/// Marker component for the player in High Seas view
#[derive(Component)]
pub struct HighSeasPlayer;

/// Marker component for AI ships in High Seas view (no physics, just visual)
#[derive(Component)]
pub struct HighSeasAI;

/// Resource holding the procedurally generated tileset image handle
#[derive(Resource)]
pub struct TilesetHandle(pub Handle<Image>);

/// Resource holding a spatial hash of AI ship positions for encounter detection.
/// Rebuilt each frame to reflect current positions.
#[derive(Resource, Default)]
pub struct EncounterSpatialHash {
    pub hash: SpatialHash<Entity>,
}

/// Encounter detection radius in world units (4 tiles = 256 units)
const ENCOUNTER_RADIUS: f32 = 256.0;

/// Cooldown to prevent rapid encounter re-triggering.
#[derive(Resource, Default)]
pub struct EncounterCooldown {
    /// If true, an encounter is being processed and no new ones should trigger.
    pub active: bool,
}

/// Resource storing data about the last encountered enemy for combat spawning.
#[derive(Resource, Default)]
pub struct EncounteredEnemy {
    /// Faction of the encountered enemy.
    pub faction: Option<FactionId>,
}

/// Creates a procedural tileset texture with colors for each tile type.
/// 
/// Layout: 5 tiles in a row (64x64 each), total 320x64 pixels
/// Index 0: Deep Water (dark blue)
/// Index 1: Shallow Water (light blue/teal)
/// Index 2: Sand (tan/beige)
/// Index 3: Land (green)
/// Index 4: Port (brown/wood)
fn create_tileset_texture(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    const TILE_SIZE: u32 = 64;
    const NUM_TILES: u32 = 6; // Added 1 for Fog/Parchment
    const TEXTURE_WIDTH: u32 = TILE_SIZE * NUM_TILES;
    const TEXTURE_HEIGHT: u32 = TILE_SIZE;

    // Create RGBA pixel data
    let mut data = vec![0u8; (TEXTURE_WIDTH * TEXTURE_HEIGHT * 4) as usize];

    // Define colors for each tile type (RGBA)
    let colors: [(u8, u8, u8, u8); 6] = [
        (30, 60, 120, 255),    // Index 0: Deep Water - dark blue
        (60, 130, 170, 255),   // Index 1: Shallow Water - teal
        (220, 190, 140, 255),  // Index 2: Sand - tan
        (80, 140, 80, 255),    // Index 3: Land - green
        (120, 80, 50, 255),    // Index 4: Port - brown
        (240, 230, 200, 255),  // Index 5: Fog/Parchment - cream/dark tan
    ];

    // Fill each tile with its color
    for tile_idx in 0..NUM_TILES {
        let (r, g, b, a) = colors[tile_idx as usize];
        let tile_start_x = tile_idx * TILE_SIZE;

        for y in 0..TILE_SIZE {
            for x in 0..TILE_SIZE {
                let px = tile_start_x + x;
                let py = y;
                let pixel_idx = ((py * TEXTURE_WIDTH + px) * 4) as usize;

                // Add subtle variation for visual interest
                let variation = ((x + y) % 8) as i16 - 4;
                let r_var = (r as i16 + variation).clamp(0, 255) as u8;
                let g_var = (g as i16 + variation).clamp(0, 255) as u8;
                let b_var = (b as i16 + variation).clamp(0, 255) as u8;

                data[pixel_idx] = r_var;
                data[pixel_idx + 1] = g_var;
                data[pixel_idx + 2] = b_var;
                data[pixel_idx + 3] = a;
            }
        }
    }

    // Create the image
    // Use both MAIN_WORLD and RENDER_WORLD to ensure the texture is properly retained
    let image = Image::new(
        bevy::render::render_resource::Extent3d {
            width: TEXTURE_WIDTH,
            height: TEXTURE_HEIGHT,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    let handle = images.add(image);
    commands.insert_resource(TilesetHandle(handle));

    info!("Procedural tileset created: {}x{} pixels, {} tiles", TEXTURE_WIDTH, TEXTURE_HEIGHT, NUM_TILES);
}

/// Generates the world map using procedural noise.
/// Uses a random seed for variety between game sessions.
fn generate_procedural_map(mut map_data: ResMut<MapData>) {
    use crate::utils::procgen::{generate_world_map, MapGenConfig};
    use rand::Rng;
    
    // Generate random seed for this game session
    let seed: u32 = rand::thread_rng().gen();
    
    let config = MapGenConfig {
        seed,
        width: 512,
        height: 512,
        ..Default::default()
    };
    
    *map_data = generate_world_map(config);
}

/// Spawns the tilemap from MapData resource.
/// Skips if tilemap already exists (persists across state transitions).
fn spawn_tilemap_from_map_data(
    mut commands: Commands,
    map_data: Res<MapData>,
    tileset: Option<Res<TilesetHandle>>,
    existing_tilemap: Query<Entity, With<WorldMap>>,
) {
    // Skip if tilemap already exists
    if !existing_tilemap.is_empty() {
        return;
    }

    let Some(tileset) = tileset else {
        error!("TilesetHandle resource not found! Tilemap cannot be spawned.");
        return;
    };

    // Define map dimensions from MapData
    let map_size = TilemapSize { 
        x: map_data.width, 
        y: map_data.height 
    };

    // Create a tilemap entity early to associate tiles with it
    let tilemap_entity = commands.spawn_empty().id();

    // Create tile storage
    let mut tile_storage = TileStorage::empty(map_size);

    // Spawn tiles from MapData
    for (x, y, tile_type) in map_data.iter() {
        let tile_pos = TilePos { x, y };
        let texture_index = tile_type.texture_index();

        let tile_entity = commands
            .spawn((
                TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(texture_index),
                    ..Default::default()
                },
                WorldMapTile,
            ))
            .id();
        tile_storage.set(&tile_pos, tile_entity);
    }

    // Tile size (64x64 pixels)
    let tile_size = TilemapTileSize { x: 64.0, y: 64.0 };
    let grid_size: TilemapGridSize = tile_size.into();
    let map_type = TilemapType::default(); // Square tiles

    commands.entity(tilemap_entity).insert((
        TilemapBundle {
            grid_size,
            map_type,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(tileset.0.clone()),
            tile_size,
            // Center the tilemap at origin by offsetting by half the map size
            transform: Transform::from_xyz(
                -(map_size.x as f32 * tile_size.x) / 2.0,
                -(map_size.y as f32 * tile_size.y) / 2.0,
                -10.0, // Below ships
            ),
            ..Default::default()
        },
        WorldMap,
    ));

    info!("World map tilemap spawned: {}x{} tiles", map_size.x, map_size.y);

    // --- Spawn Fog Tilemap ---
    let fog_tilemap_entity = commands.spawn_empty().id();
    let mut fog_storage = TileStorage::empty(map_size);

    for x in 0..map_data.width {
        for y in 0..map_data.height {
            let tile_pos = TilePos { x, y };
            
            let tile_entity = commands
                .spawn((
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(fog_tilemap_entity),
                        texture_index: TileTextureIndex(5), // Fog/Parchment tile
                        ..Default::default()
                    },
                    FogTile,
                ))
                .id();
            fog_storage.set(&tile_pos, tile_entity);
        }
    }

    commands.entity(fog_tilemap_entity).insert((
        TilemapBundle {
            grid_size,
            map_type,
            size: map_size,
            storage: fog_storage,
            texture: TilemapTexture::Single(tileset.0.clone()),
            tile_size,
            // Positioned slightly above the world map
            transform: Transform::from_xyz(
                -(map_size.x as f32 * tile_size.x) / 2.0,
                -(map_size.y as f32 * tile_size.y) / 2.0,
                -5.0, // Above world map (-10), below ships (1+)
            ),
            ..Default::default()
        },
        FogMap,
    ));

    info!("Fog tilemap spawned: {}x{} tiles", map_size.x, map_size.y);
}

/// Spawns the player ship in the High Seas view.
fn spawn_high_seas_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    _map_data: Res<MapData>,
) {
    use crate::components::{Cargo, Gold};
    
    info!("Spawning player for High Seas...");
    
    // Spawn at map center
    let center_x = 0.0;
    let center_y = 0.0;
    
    let texture_handle: Handle<Image> = asset_server.load("sprites/ships/player.png");
    
    commands.spawn((
        Name::new("High Seas Player"),
        Player,
        Ship,
        HighSeasPlayer,
        Vision { radius: 10.0 }, // Sight radius in tiles
        Health::default(), // Required by camera follow
        Cargo::new(100),   // Trading capacity
        Gold(100),         // Starting gold
        Sprite {
            image: texture_handle,
            custom_size: Some(Vec2::splat(64.0)),
            flip_y: true,
            ..default()
        },
        Transform::from_xyz(center_x, center_y, 2.0), // Above fog
    ));
}

/// Despawn High Seas player when leaving the state.
fn despawn_high_seas_player(
    mut commands: Commands,
    query: Query<Entity, With<HighSeasPlayer>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}


/// Despawns the world map when leaving HighSeas state.
pub fn despawn_tilemap(
    mut commands: Commands,
    tilemap_query: Query<Entity, Or<(With<WorldMap>, With<FogMap>)>>,
    tile_query: Query<Entity, Or<(With<WorldMapTile>, With<FogTile>)>>,
) {
    // Despawn all tiles first
    for entity in tile_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Despawn the tilemap entity
    for entity in tilemap_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    info!("Tilemaps despawned");
}

/// Spawns AI ships on the High Seas map at random navigable locations.
fn spawn_high_seas_ai_ships(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    map_data: Res<MapData>,
) {
    use rand::prelude::*;
    
    let mut rng = rand::thread_rng();
    let num_ships = 50;
    
    let texture_handle: Handle<Image> = asset_server.load("sprites/ships/enemy.png");
    
    // Collect navigable tiles (deep water only for AI ships)
    let navigable_tiles: Vec<(u32, u32)> = map_data.iter()
        .filter(|(_, _, tile)| tile.is_navigable())
        .map(|(x, y, _)| (x, y))
        .collect();
    
    if navigable_tiles.is_empty() {
        warn!("No navigable tiles found for AI ship spawning!");
        return;
    }
    
    for i in 0..num_ships {
        let (tile_x, tile_y) = navigable_tiles[rng.gen_range(0..navigable_tiles.len())];
        let world_pos = tile_to_world(IVec2::new(tile_x as i32, tile_y as i32), map_data.width, map_data.height);
        
        // All ships are Pirates for now (until reputation system in Phase 5)
        let faction = FactionId::Pirates;
        
        commands.spawn((
            Name::new(format!("High Seas AI Ship {}", i)),
            Ship,
            AI,
            Faction(faction),
            HighSeasAI,
            Health::default(),
            Sprite {
                image: texture_handle.clone(),
                custom_size: Some(Vec2::splat(48.0)), // Slightly smaller than player
                flip_y: true,
                ..default()
            },
            Transform::from_xyz(world_pos.x, world_pos.y, 1.0), // Same layer as player
            OrderQueue::with_order(Order::Patrol {
                center: world_pos,
                radius: 1500.0, // Approx 23 tiles
                waypoint_index: 0,
            }),
        ));
    }
    
    info!("Spawned {} AI ships on High Seas map", num_ships);
}

/// Despawn AI ships when leaving the High Seas state.
fn despawn_high_seas_ai_ships(
    mut commands: Commands,
    query: Query<Entity, With<HighSeasAI>>,
) {
    let count = query.iter().count();
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    info!("Despawned {} AI ships", count);
}

/// Updates AI ship visibility based on fog of war.
/// Ships in unexplored tiles are hidden, ships in explored tiles are visible.
fn fog_of_war_ai_visibility_system(
    fog_of_war: Res<FogOfWar>,
    map_data: Res<MapData>,
    mut query: Query<(&Transform, &mut Visibility), With<HighSeasAI>>,
) {
    for (transform, mut visibility) in &mut query {
        let world_pos = transform.translation.truncate();
        let tile_pos = world_to_tile(world_pos, map_data.width, map_data.height);
        
        if fog_of_war.is_explored(tile_pos) {
            *visibility = Visibility::Inherited;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

/// Rebuilds the encounter spatial hash from current AI ship positions.
/// Runs each frame to keep positions current.
fn rebuild_encounter_spatial_hash(
    mut encounter_hash: ResMut<EncounterSpatialHash>,
    ai_query: Query<(Entity, &Transform), With<HighSeasAI>>,
) {
    encounter_hash.hash.clear();
    
    for (entity, transform) in &ai_query {
        let pos = transform.translation.truncate();
        encounter_hash.hash.insert(pos, entity);
    }
}

/// Detects when the player is near hostile AI ships and triggers combat.
fn encounter_detection_system(
    encounter_hash: Res<EncounterSpatialHash>,
    encounter_cooldown: Res<EncounterCooldown>,
    player_query: Query<&Transform, (With<Player>, With<HighSeasPlayer>)>,
    ai_query: Query<(Entity, &Transform, &Faction, Option<&Name>), With<HighSeasAI>>,
    mut combat_events: EventWriter<CombatTriggeredEvent>,
) {
    // Don't trigger new encounters while one is being processed
    if encounter_cooldown.active {
        return;
    }
    
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    
    let player_pos = player_transform.translation.truncate();
    let nearby_ships = encounter_hash.hash.query(player_pos, ENCOUNTER_RADIUS);
    
    for &entity_ref in &nearby_ships {
        let entity = *entity_ref;
        if let Ok((_, ai_transform, faction, name)) = ai_query.get(entity) {
            let ai_pos = ai_transform.translation.truncate();
            let distance = player_pos.distance(ai_pos);
            
            // Double-check distance (spatial hash is approximate)
            if distance <= ENCOUNTER_RADIUS {
                // Hostility check (3.6.4): Pirates are always hostile
                let is_hostile = matches!(faction.0, FactionId::Pirates);
                
                if is_hostile {
                    let ship_name = name.map(|n| n.as_str()).unwrap_or("Unknown Ship");
                    info!(
                        "Hostile encounter! {} ({:?}) at distance {:.0} - triggering combat!",
                        ship_name, faction.0, distance
                    );
                    
                    // Emit combat triggered event (3.6.5)
                    combat_events.send(CombatTriggeredEvent {
                        enemy_entity: entity,
                        enemy_faction: faction.0,
                    });
                    
                    // Only trigger one encounter at a time
                    return;
                }
            }
        }
    }
}

/// Handles combat trigger events by transitioning to Combat state.
fn handle_combat_trigger_system(
    mut combat_events: EventReader<CombatTriggeredEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    mut encounter_cooldown: ResMut<EncounterCooldown>,
    mut encountered_enemy: ResMut<EncounteredEnemy>,
) {
    for event in combat_events.read() {
        info!(
            "Combat triggered by {:?} faction ship! Transitioning to Combat state.",
            event.enemy_faction
        );
        
        // Store encounter data for combat spawning (3.6.7)
        encountered_enemy.faction = Some(event.enemy_faction);
        
        // Set cooldown to prevent re-triggering
        encounter_cooldown.active = true;
        
        // Transition to combat state (3.6.6)
        next_state.set(GameState::Combat);
        
        // Only handle one event per frame
        break;
    }
}

/// Resets the encounter cooldown when entering HighSeas state.
fn reset_encounter_cooldown(mut cooldown: ResMut<EncounterCooldown>) {
    cooldown.active = false;
}

/// Marker component for port entities spawned on the world map.
#[derive(Component)]
pub struct HighSeasPort;

/// Spawns port entities at port tile locations on the map.
/// Each port gets an Inventory with random goods, a generated name, and a faction.
fn spawn_port_entities(
    mut commands: Commands,
    map_data: Res<MapData>,
) {
    use rand::Rng;
    
    let mut rng = rand::thread_rng();
    let mut port_count = 0;
    
    // Find all port tiles and spawn port entities
    for (x, y, tile_type) in map_data.iter() {
        if tile_type.is_port() {
            // Convert tile coordinates to world position
            let world_pos = tile_to_world(
                IVec2::new(x as i32, y as i32), 
                map_data.width, 
                map_data.height
            );
            
            // Generate port name and assign random faction (not Pirates)
            let name = generate_port_name();
            let faction = match rng.gen_range(0..3) {
                0 => FactionId::NationA,
                1 => FactionId::NationB,
                _ => FactionId::NationC,
            };
            
            // Spawn the port entity using the port plugin function
            let entity = spawn_port(&mut commands, world_pos, name.clone(), Faction(faction));
            
            // Add the HighSeasPort marker for cleanup
            commands.entity(entity).insert(HighSeasPort);
            
            port_count += 1;
        }
    }
    
    info!("Spawned {} port entities on the map", port_count);
}

/// Despawns all port entities when leaving the High Seas state.
fn despawn_port_entities(
    mut commands: Commands,
    query: Query<Entity, With<HighSeasPort>>,
) {
    let count = query.iter().count();
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    info!("Despawned {} port entities", count);
}

/// System to spawn the player's fleet when entering HighSeas.
/// Populates FleetEntities with spawned entity IDs for UI access.
pub fn spawn_player_fleet(
    mut commands: Commands,
    player_fleet: Res<crate::resources::PlayerFleet>,
    mut fleet_entities: ResMut<crate::resources::FleetEntities>,
    player_query: Query<(Entity, &Transform), With<crate::components::Player>>,
    asset_server: Res<AssetServer>,
) {
    // Clear any stale entity references
    fleet_entities.entities.clear();

    let Ok((player_entity, player_transform)) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (i, ship_data) in player_fleet.ships.iter().enumerate() {
        // Spawn fleet ships in a formation behind the player
        let offset = Vec2::new(30.0 * (i as f32 + 1.0), -30.0 * (i as f32 + 1.0));
        let spawn_pos = player_pos + offset;
        
        let texture_handle = asset_server.load(&ship_data.sprite_path);

        let entity = commands.spawn((
            Name::new(format!("Fleet Ship: {}", ship_data.name)),
            crate::components::Ship,
            crate::components::AI,
            crate::components::PlayerOwned,
            HighSeasAI,
            crate::components::Health {
                hull: ship_data.hull_health,
                hull_max: ship_data.max_hull_health,
                ..default()
            },
            Sprite {
                image: texture_handle,
                custom_size: Some(Vec2::splat(48.0)),
                flip_y: true,
                ..default()
            },
            Transform::from_xyz(spawn_pos.x, spawn_pos.y, 1.0),
            // Default order: Escort the player
            crate::components::OrderQueue::with_order(crate::components::Order::Escort {
                target: player_entity,
                follow_distance: 60.0 + (i as f32 * 20.0),
            }),
        )).id();

        // Track entity ID for UI access
        fleet_entities.entities.push(entity);
    }

    if !fleet_entities.entities.is_empty() {
        info!("Spawned {} fleet ships, tracked in FleetEntities", fleet_entities.entities.len());
    }
}

/// Clears FleetEntities when leaving HighSeas state.
fn clear_fleet_entities(mut fleet_entities: ResMut<crate::resources::FleetEntities>) {
    fleet_entities.entities.clear();
}

