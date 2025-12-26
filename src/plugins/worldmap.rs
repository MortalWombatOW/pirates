use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy_ecs_tilemap::prelude::*;
use bevy_prototype_lyon::prelude::*;
use crate::plugins::core::GameState;
use crate::plugins::port::{spawn_port, generate_port_name};
use crate::plugins::debug_ui::DebugToggles;
use crate::resources::{MapData, FogOfWar, RouteCache};
use crate::components::{Player, Ship, Health, Vision, AI, Faction, FactionId, Order, OrderQueue, HighSeasEntity};
use crate::components::ship::ShipType;
use crate::systems::{
    fog_of_war_update_system, FogTile,
    click_to_navigate_system,
    path_visualization_system, port_arrival_system, order_execution_system,
    contract_delegation_system,
    intel_visualization_system,
    // Landmass velocity-based movement systems
    landmass_player_movement_system, landmass_ai_movement_system,
    arrival_detection_system, sync_destination_to_agent_target,
    coastline_avoidance_system,
};
use crate::utils::pathfinding::{tile_to_world, world_to_tile};
use crate::utils::spatial_hash::SpatialHash;
use crate::utils::geometry::{extract_contours, CoastlinePolygon, offset_polygon, build_landmass_navmeshes};
use crate::resources::{NavMeshResource, PendingNavMeshes, LandmassArchipelagos, ShoreBufferTier};
use bevy_landmass::prelude::*;
use bevy_landmass::NavMeshHandle;
use bevy_landmass::debug::{Landmass2dDebugPlugin, EnableLandmassDebug};
use std::sync::Arc;
use crate::events::CombatTriggeredEvent;
use crate::resources::stippling_material::StipplingMaterial;

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
            // Vector graphics for coastlines
            .add_plugins(ShapePlugin)
            // Landmass navigation plugin
            .add_plugins(Landmass2dPlugin::default())
            // Debug visualization (press F3 to toggle)
            .add_plugins(Landmass2dDebugPlugin { draw_on_start: false, ..Default::default() })
            // Stippling material for water depth visualization
            .add_plugins(bevy::sprite::Material2dPlugin::<StipplingMaterial>::default())
            // Initialize resources
            .init_resource::<MapData>()
            .init_resource::<FogOfWar>()
            .init_resource::<RouteCache>()
            .init_resource::<CoastlineData>()
            .init_resource::<NavMeshResource>()
            .init_resource::<EncounterSpatialHash>()
            .init_resource::<EncounterCooldown>()
            .init_resource::<EncounteredEnemy>()
            .init_resource::<crate::resources::PlayerFleet>()
            .init_resource::<crate::resources::FleetEntities>()
            .add_event::<CombatTriggeredEvent>()
            .add_systems(Startup, (
                generate_procedural_map,
                create_tileset_texture,
                extract_coastlines_system.after(generate_procedural_map),
                initialize_archipelagos.after(extract_coastlines_system),
                spawn_navigation_islands.after(initialize_archipelagos),
            ))
            .add_systems(OnEnter(GameState::HighSeas), (
                spawn_tilemap_from_map_data,
                spawn_coastline_shapes,
                spawn_elevation_markers.after(spawn_coastline_shapes),
                spawn_high_seas_player,
                spawn_high_seas_ai_ships,
                spawn_player_fleet,
                spawn_port_entities,
                spawn_location_labels.after(spawn_port_entities),
                spawn_legacy_wrecks,
                reset_encounter_cooldown,
                show_tilemap,
            ))
            // Fog of war and visibility systems
            .add_systems(Update, (
                fog_of_war_update_system,
                crate::systems::ink_reveal::spawn_ink_reveals.after(fog_of_war_update_system),
                crate::systems::ink_reveal::animate_ink_reveals.after(crate::systems::ink_reveal::spawn_ink_reveals),
                fog_of_war_ai_visibility_system,
                coastline_visibility_system,
            ).run_if(in_state(GameState::HighSeas)))
            // Encounter and combat systems
            .add_systems(Update, (
                rebuild_encounter_spatial_hash,
                encounter_detection_system.after(rebuild_encounter_spatial_hash),
                handle_combat_trigger_system.after(encounter_detection_system),
            ).run_if(in_state(GameState::HighSeas)))
            // Navigation systems (landmass-only, no grid fallback)
            .add_systems(Update, (
                click_to_navigate_system,
                order_execution_system,
                sync_destination_to_agent_target.after(click_to_navigate_system).after(order_execution_system),
            ).run_if(in_state(GameState::HighSeas)))
            // Movement systems (landmass velocity-based)
            .add_systems(Update, (
                landmass_player_movement_system,
                landmass_ai_movement_system,
                arrival_detection_system
                    .after(landmass_player_movement_system)
                    .after(landmass_ai_movement_system),
                coastline_avoidance_system
                    .after(landmass_player_movement_system)
                    .after(landmass_ai_movement_system),
            ).run_if(in_state(GameState::HighSeas)))
            // Visualization and other systems
            .add_systems(Update, (
                path_visualization_system,
                intel_visualization_system,
                port_arrival_system,
                contract_delegation_system,
                wreck_exploration_system,
                toggle_navmesh_debug,
            ).run_if(in_state(GameState::HighSeas)))
            .add_systems(OnEnter(GameState::Combat), hide_tilemap)
            .add_systems(OnExit(GameState::HighSeas), clear_fleet_entities);
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

/// Marker component for legacy wreck entities from previous runs.
#[derive(Component)]
pub struct LegacyWreckMarker {
    /// Index into MetaProfile.legacy_wrecks for this wreck's data.
    pub wreck_index: usize,
}

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

/// Resource storing extracted coastline polygons for rendering.
/// Populated on startup after map generation.
#[derive(Resource, Default)]
pub struct CoastlineData {
    /// CCW-ordered coastline polygons (land on left)
    pub polygons: Vec<CoastlinePolygon>,
}

/// Marker component for coastline shape entities.
/// Used to toggle visibility and clean up on state exit.
#[derive(Component)]
pub struct CoastlineShape;

/// Marker component for location label Text2d entities.
/// Used to clean up labels on state exit.
#[derive(Component)]
pub struct LocationLabelMarker;

/// Creates a procedural tileset texture with colors for each tile type.
/// 
/// Layout: 5 tiles in a row (64x64 each), total 320x64 pixels
/// Index 0: Deep Water (dark blue)
/// Index 1: Shallow Water (light blue/teal)
/// Index 2: Sand (tan/beige)
/// Index 3: Land (green)
/// Index 4: Port (brown/wood)
/// Index 5: Hills (darker green with hachures)
/// Index 6: Mountains (dark gray with peaks)
/// Index 7: Fog/Parchment (cream)
fn create_tileset_texture(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    const TILE_SIZE: u32 = 64;
    const NUM_TILES: u32 = 8;
    const TEXTURE_WIDTH: u32 = TILE_SIZE * NUM_TILES;
    const TEXTURE_HEIGHT: u32 = TILE_SIZE;

    // Create RGBA pixel data
    let mut data = vec![0u8; (TEXTURE_WIDTH * TEXTURE_HEIGHT * 4) as usize];

    // Define colors for each tile type (RGBA)
    let colors: [(u8, u8, u8, u8); 8] = [
        (30, 60, 120, 255),    // Index 0: Deep Water - dark blue
        (60, 130, 170, 255),   // Index 1: Shallow Water - teal
        (220, 190, 140, 255),  // Index 2: Sand - tan
        (80, 140, 80, 255),    // Index 3: Land - green
        (120, 80, 50, 255),    // Index 4: Port - brown
        (60, 110, 60, 255),    // Index 5: Hills - darker green
        (80, 80, 90, 255),     // Index 6: Mountains - dark gray
        (240, 230, 200, 255),  // Index 7: Fog/Parchment - cream
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
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StipplingMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Skip if tilemap already exists
    if !existing_tilemap.is_empty() {
        return;
    }

    // Spawn Stippling Overlay
    spawn_stipple_overlay(&mut commands, &map_data, &mut images, &mut materials, &mut meshes);

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
    for (x, y, tile) in map_data.iter() {
        let tile_pos = TilePos { x, y };
        let texture_index = tile.tile_type.texture_index();

        let tile_entity = commands
            .spawn((
                TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(texture_index),
                    ..Default::default()
                },
                WorldMapTile,
                HighSeasEntity,
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
        HighSeasEntity,
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
                        texture_index: TileTextureIndex(7), // Fog/Parchment tile
                        ..Default::default()
                    },
                    FogTile,
                    HighSeasEntity,
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
        HighSeasEntity,
    ));

    info!("Fog tilemap spawned: {}x{} tiles", map_size.x, map_size.y);
}

/// Spawns the player ship in the High Seas view.
/// Applies archetype bonuses from the selected starting character.
fn spawn_high_seas_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    _map_data: Res<MapData>,
    selected_archetype: Res<crate::plugins::main_menu::SelectedArchetype>,
    registry: Res<crate::resources::ArchetypeRegistry>,
    mut faction_registry: ResMut<crate::resources::FactionRegistry>,
    archipelagos: Option<Res<LandmassArchipelagos>>,
) {
    use crate::components::{Cargo, Gold};

    // Get archetype configuration
    let archetype_config = registry.get(selected_archetype.0);
    let (starting_gold, ship_type) = archetype_config
        .map(|c| (c.starting_gold, c.ship_type))
        .unwrap_or((500, ShipType::Sloop)); // Fallback to defaults

    info!(
        "Spawning player for High Seas with archetype {:?}: {} gold, {:?}",
        selected_archetype.0, starting_gold, ship_type
    );

    // Apply faction reputation bonuses from archetype
    if let Some(config) = archetype_config {
        for (faction_id, rep_modifier) in &config.faction_reputation {
            if let Some(faction_state) = faction_registry.get_mut(*faction_id) {
                faction_state.player_reputation += rep_modifier;
                info!(
                    "Applied {:+} reputation to {:?} (now {})",
                    rep_modifier, faction_id, faction_state.player_reputation
                );
            }
        }
    }

    // Spawn at map center
    let center_x = 0.0;
    let center_y = 0.0;

    // Select sprite based on ship type
    let sprite_path = match ship_type {
        ShipType::Sloop => "sprites/ships/player.png",
        ShipType::Frigate => "sprites/ships/frigate.png",
        ShipType::Schooner => "sprites/ships/schooner.png",
        ShipType::Raft => "sprites/ships/raft.png",
    };
    let texture_handle: Handle<Image> = asset_server.load(sprite_path);

    // Adjust cargo capacity based on ship type
    let cargo_capacity = match ship_type {
        ShipType::Sloop => 100,
        ShipType::Frigate => 200,
        ShipType::Schooner => 150,
        ShipType::Raft => 30,
    };

    // Get appropriate archipelago for ship type
    let tier = ShoreBufferTier::from_ship_type(ship_type);
    let archipelago_entity = archipelagos.as_ref().map(|a| a.get(tier));

    let mut entity_commands = commands.spawn((
        Name::new("High Seas Player"),
        Player,
        Ship,
        ship_type, // ShipType component for turn rate calculations
        HighSeasPlayer,
        Vision { radius: 10.0 }, // Sight radius in tiles
        Health::default(),       // Required by camera follow
        Cargo::new(cargo_capacity),
        Gold(starting_gold),
        Sprite {
            image: texture_handle,
            custom_size: Some(Vec2::splat(64.0)),
            flip_y: true,
            ..default()
        },
        Transform::from_xyz(center_x, center_y, 2.0), // Above fog
        HighSeasEntity,
    ));

    // Add landmass agent components if archipelago is available
    if let Some(arch_entity) = archipelago_entity {
        entity_commands.insert((
            Agent2dBundle {
                agent: Default::default(),
                settings: AgentSettings {
                    radius: tier.agent_radius(),
                    desired_speed: ship_type.base_speed(),
                    max_speed: ship_type.base_speed() * 1.3,
                },
                archipelago_ref: ArchipelagoRef2d::new(arch_entity),
            },
        ));
        info!("Added landmass Agent2dBundle to player (tier: {:?})", tier);
    }
}

/// Spawns legacy wreck entities from previous deaths.
/// Wrecks are interactable markers on the map containing loot from past runs.
fn spawn_legacy_wrecks(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    profile: Res<crate::resources::MetaProfile>,
    existing_wrecks: Query<Entity, With<LegacyWreckMarker>>,
) {
    // Don't spawn duplicates if wrecks already exist
    if !existing_wrecks.is_empty() {
        return;
    }

    const TILE_SIZE: f32 = 16.0;

    for (index, wreck) in profile.legacy_wrecks.iter().enumerate() {
        // Convert tile position back to world coordinates
        let world_pos = Vec2::new(
            wreck.position.x as f32 * TILE_SIZE,
            wreck.position.y as f32 * TILE_SIZE,
        );

        info!(
            "Spawning legacy wreck '{}' at {:?} (tile {:?}) with {} gold",
            wreck.ship_name, world_pos, wreck.position, wreck.gold
        );

        // Spawn wreck entity with visual indicator
        commands.spawn((
            Name::new(format!("Wreck: {}", wreck.ship_name)),
            LegacyWreckMarker { wreck_index: index },
            Sprite {
                image: asset_server.load("sprites/loot/wreck.png"),
                custom_size: Some(Vec2::splat(48.0)),
                color: Color::srgba(0.8, 0.6, 0.4, 0.9), // Weathered brown tint
                ..default()
            },
            Transform::from_xyz(world_pos.x, world_pos.y, 1.5), // Between fog and ships
            HighSeasEntity,
        ));
    }

    if !profile.legacy_wrecks.is_empty() {
        info!("Spawned {} legacy wrecks from previous runs", profile.legacy_wrecks.len());
    }
}

/// Proximity radius for wreck exploration (in world units).
const WRECK_EXPLORE_RADIUS: f32 = 48.0;

/// System that handles wreck exploration when the player gets close.
/// Transfers gold and cargo from the wreck to the player, then removes the wreck.
fn wreck_exploration_system(
    mut commands: Commands,
    player_query: Query<&Transform, With<HighSeasPlayer>>,
    wreck_query: Query<(Entity, &Transform, &LegacyWreckMarker)>,
    mut player_gold_query: Query<&mut crate::components::Gold, With<Player>>,
    mut player_cargo_query: Query<&mut crate::components::Cargo, With<Player>>,
    mut profile: ResMut<crate::resources::MetaProfile>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (wreck_entity, wreck_transform, wreck_marker) in &wreck_query {
        let wreck_pos = wreck_transform.translation.truncate();
        let distance = player_pos.distance(wreck_pos);

        if distance <= WRECK_EXPLORE_RADIUS {
            // Get wreck data from profile
            let Some(wreck_data) = profile.legacy_wrecks.get(wreck_marker.wreck_index) else {
                // Wreck data not found, just despawn the entity
                commands.entity(wreck_entity).despawn_recursive();
                continue;
            };

            info!(
                "Exploring wreck '{}': {} gold, {} cargo items",
                wreck_data.ship_name,
                wreck_data.gold,
                wreck_data.cargo.len()
            );

            // Transfer gold to player
            if let Ok(mut gold) = player_gold_query.get_single_mut() {
                gold.0 += wreck_data.gold;
                info!("Recovered {} gold from wreck (total: {})", wreck_data.gold, gold.0);
            }

            // Transfer cargo to player
            if let Ok(mut cargo) = player_cargo_query.get_single_mut() {
                for (good_type_str, quantity) in &wreck_data.cargo {
                    // Try to parse the good type
                    if let Some(good_type) = parse_good_type(good_type_str) {
                        let space_available = cargo.capacity.saturating_sub(cargo.total_units());
                        let to_add = (*quantity).min(space_available);
                        if to_add > 0 {
                            *cargo.goods.entry(good_type).or_insert(0) += to_add;
                            info!("Recovered {} {:?} from wreck", to_add, good_type);
                        }
                    }
                }
            }

            // Remove wreck from profile
            if wreck_marker.wreck_index < profile.legacy_wrecks.len() {
                profile.legacy_wrecks.remove(wreck_marker.wreck_index);
                info!("Removed wreck from profile, {} wrecks remaining", profile.legacy_wrecks.len());

                // Save profile to persist wreck removal
                if let Err(e) = profile.save_to_file() {
                    error!("Failed to save profile after wreck exploration: {}", e);
                }
            }

            // Despawn wreck entity
            commands.entity(wreck_entity).despawn_recursive();
        }
    }
}

/// Helper to parse good type strings back to GoodType enum.
fn parse_good_type(s: &str) -> Option<crate::components::cargo::GoodType> {
    use crate::components::cargo::GoodType;
    match s {
        "Sugar" => Some(GoodType::Sugar),
        "Rum" => Some(GoodType::Rum),
        "Spices" => Some(GoodType::Spices),
        "Timber" => Some(GoodType::Timber),
        "Cloth" => Some(GoodType::Cloth),
        "Weapons" => Some(GoodType::Weapons),
        _ => None,
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

/// Hides the world map and fog tilemaps (used when entering Combat).
fn hide_tilemap(
    mut tilemap_query: Query<&mut Visibility, Or<(With<WorldMap>, With<FogMap>)>>,
) {
    for mut visibility in &mut tilemap_query {
        *visibility = Visibility::Hidden;
    }
}

/// Shows the world map and fog tilemaps (used when entering HighSeas).
fn show_tilemap(
    mut tilemap_query: Query<&mut Visibility, Or<(With<WorldMap>, With<FogMap>)>>,
) {
    for mut visibility in &mut tilemap_query {
        *visibility = Visibility::Inherited;
    }
}

/// Spawns AI ships on the High Seas map at random navigable locations.
fn spawn_high_seas_ai_ships(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    map_data: Res<MapData>,
    archipelagos: Option<Res<LandmassArchipelagos>>,
) {
    use rand::prelude::*;

    let mut rng = rand::thread_rng();
    let num_ships = 50;

    let texture_handle: Handle<Image> = asset_server.load("sprites/ships/enemy.png");

    // Collect navigable tiles (deep water only for AI ships)
    let navigable_tiles: Vec<(u32, u32)> = map_data.iter()
        .filter(|(_, _, tile)| tile.tile_type.is_navigable())
        .map(|(x, y, _)| (x, y))
        .collect();

    if navigable_tiles.is_empty() {
        warn!("No navigable tiles found for AI ship spawning!");
        return;
    }

    // AI ships use random ship types with weighted distribution
    let ship_types = [
        (ShipType::Sloop, 0.4),     // 40% small pirates
        (ShipType::Schooner, 0.35), // 35% merchants
        (ShipType::Frigate, 0.15),  // 15% heavy ships
        (ShipType::Raft, 0.1),      // 10% rafts
    ];

    for i in 0..num_ships {
        let (tile_x, tile_y) = navigable_tiles[rng.gen_range(0..navigable_tiles.len())];
        let world_pos = tile_to_world(IVec2::new(tile_x as i32, tile_y as i32), map_data.width, map_data.height);

        // Select ship type based on weighted distribution
        let roll: f32 = rng.gen();
        let mut cumulative = 0.0;
        let ship_type = ship_types.iter()
            .find(|(_, weight)| {
                cumulative += weight;
                roll < cumulative
            })
            .map(|(t, _)| *t)
            .unwrap_or(ShipType::Sloop);

        let tier = ShoreBufferTier::from_ship_type(ship_type);
        let archipelago_entity = archipelagos.as_ref().map(|a| a.get(tier));

        // All ships are Pirates for now (until reputation system in Phase 5)
        let faction = FactionId::Pirates;

        let mut entity_commands = commands.spawn((
            Name::new(format!("High Seas AI Ship {}", i)),
            Ship,
            ship_type, // ShipType component for turn rate calculations
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
            HighSeasEntity,
        ));

        // Add landmass agent components if archipelago is available
        if let Some(arch_entity) = archipelago_entity {
            entity_commands.insert((
                Agent2dBundle {
                    agent: Default::default(),
                    settings: AgentSettings {
                        radius: tier.agent_radius(),
                        desired_speed: ship_type.base_speed() * 0.5, // AI slower than player
                        max_speed: ship_type.base_speed() * 0.65,
                    },
                    archipelago_ref: ArchipelagoRef2d::new(arch_entity),
                },
            ));
        }
    }

    info!("Spawned {} AI ships on High Seas map", num_ships);
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
    for (x, y, tile) in map_data.iter() {
        if tile.tile_type.is_port() {
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
            commands.entity(entity).insert((HighSeasPort, HighSeasEntity));
            
            port_count += 1;
        }
    }
    
    info!("Spawned {} port entities on the map", port_count);
}

/// Spawns location labels for all ports, positioned perpendicular to nearby coastlines.
/// Uses the Quintessential font for authentic 18th-century nautical chart styling.
fn spawn_location_labels(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    coastline_data: Res<CoastlineData>,
    port_query: Query<(&Transform, &crate::components::port::PortName), With<crate::components::port::Port>>,
) {
    use crate::components::location_label::{LocationLabel, LabelImportance};
    use rand::Rng;

    let font = asset_server.load("fonts/Quintessential-Regular.ttf");
    let mut label_count = 0;
    let mut rng = rand::thread_rng();

    // Ink color matching other cartographic elements
    let ink_color = Color::srgb(0.25, 0.18, 0.12);

    for (port_transform, port_name) in &port_query {
        let port_pos = port_transform.translation.truncate();

        // Calculate perpendicular angle to nearest coastline
        let angle = calculate_coastline_perpendicular(port_pos, &coastline_data);

        // Randomly assign importance (mostly standard, some major/minor)
        let roll: f32 = rng.gen();
        let importance = if roll < 0.15 {
            LabelImportance::Major
        } else if roll < 0.85 {
            LabelImportance::Standard
        } else {
            LabelImportance::Minor
        };

        let font_size = importance.font_size();
        let label = LocationLabel::new(port_name.0.clone(), importance, angle);

        // Offset label position inland (perpendicular to coast)
        let offset_distance = 40.0 + font_size; // Move label away from port icon
        let offset = Vec2::from_angle(angle) * offset_distance;
        let label_pos = port_pos + offset;

        commands.spawn((
            Text2d::new(label.name.clone()),
            TextFont {
                font: font.clone(),
                font_size,
                ..default()
            },
            TextColor(ink_color),
            Transform::from_xyz(label_pos.x, label_pos.y, 5.0)
                .with_rotation(Quat::from_rotation_z(angle - std::f32::consts::FRAC_PI_2)),
            LocationLabelMarker,
            label,
            HighSeasEntity,
        ));

        label_count += 1;
    }

    info!("Spawned {} location labels", label_count);
}

/// Calculates the angle perpendicular to the nearest coastline edge, pointing inland.
/// Returns the angle in radians.
fn calculate_coastline_perpendicular(pos: Vec2, coastline_data: &CoastlineData) -> f32 {
    let mut nearest_dist = f32::MAX;
    let mut nearest_normal = Vec2::Y; // Default pointing up

    for polygon in &coastline_data.polygons {
        let points = &polygon.points;
        if points.len() < 2 {
            continue;
        }

        // Check each edge of the polygon
        for i in 0..points.len() {
            let a = points[i];
            let b = points[(i + 1) % points.len()];

            // Find closest point on edge to pos
            let ab = b - a;
            let ap = pos - a;
            let t = (ap.dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
            let closest = a + ab * t;
            let dist = pos.distance(closest);

            if dist < nearest_dist {
                nearest_dist = dist;
                // Normal perpendicular to edge (CCW winding means land is on left)
                // So rightward perpendicular points toward water, leftward toward land
                let edge_dir = ab.normalize_or_zero();
                // Leftward perpendicular (toward land for CCW polygon)
                nearest_normal = Vec2::new(-edge_dir.y, edge_dir.x);
            }
        }
    }

    nearest_normal.to_angle()
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
            HighSeasEntity,
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

/// Tile size in pixels for coastline extraction
const COASTLINE_TILE_SIZE: f32 = 64.0;

/// Extracts coastline polygons from the generated map data.
/// Runs on startup after procedural map generation.
fn extract_coastlines_system(
    mut commands: Commands,
    map_data: Res<MapData>,
    mut coastline_data: ResMut<CoastlineData>,
) {
    let raw_polygons = extract_contours(&map_data, COASTLINE_TILE_SIZE);
    
    // Apply smoothing
    use crate::utils::geometry::smooth_coastline;
    let polygons: Vec<CoastlinePolygon> = raw_polygons
        .into_iter()
        .filter(|poly| poly.points.len() >= 3)
        .map(|poly| CoastlinePolygon {
            points: smooth_coastline(&poly.points), // Uses COASTLINE_SUBDIVISIONS/TENSION constants
        })
        .collect();

    info!(
        "Extracted {} coastline polygons with {} total points (smoothed)",
        polygons.len(),
        polygons.iter().map(|p| p.points.len()).sum::<usize>()
    );
    
    // Build landmass NavigationMesh2d from coastline polygons
    // Calculate world bounds
    let tile_size = COASTLINE_TILE_SIZE;
    let half_width = map_data.width as f32 * tile_size / 2.0;
    let half_height = map_data.height as f32 * tile_size / 2.0;
    let map_bounds = (-half_width, -half_height, half_width, half_height);

    let pending_meshes = build_landmass_navmeshes(&polygons, map_bounds);

    // Check how many tiers were built
    let tier_count = [&pending_meshes.small, &pending_meshes.medium, &pending_meshes.large]
        .iter()
        .filter(|m| m.is_some())
        .count();

    if tier_count > 0 {
        info!("Landmass NavMeshes built successfully with {} tiers", tier_count);
    } else {
        warn!("NavMesh build failed - falling back to grid pathfinding");
    }

    // Insert pending meshes for archipelago initialization
    commands.insert_resource(pending_meshes);

    // Insert legacy NavMeshResource stub for backward compatibility during migration
    commands.insert_resource(NavMeshResource::new());
    coastline_data.polygons = polygons;
}

/// Initializes the three landmass archipelagos for different ship size tiers.
/// Each archipelago has a different agent radius corresponding to shore buffer.
fn initialize_archipelagos(mut commands: Commands) {
    // Create three archipelagos with different agent radii
    // Agent radius determines how close ships can get to obstacles
    // Use custom AgentOptions for better navigation around obstacles

    // Helper to create agent options with larger node_sample_distance
    // to prevent getting stuck in narrow triangles
    let make_options = |radius: f32| {
        AgentOptions {
            // Larger sample distance = fewer waypoints = smoother paths
            node_sample_distance: radius * 0.5,
            // Standard neighborhood for agent avoidance
            neighbourhood: radius * 10.0,
            // Standard avoidance horizons
            avoidance_time_horizon: 1.0,
            obstacle_avoidance_time_horizon: 0.3, // Lower = less sticky obstacles
            reached_destination_avoidance_responsibility: 0.1,
        }
    };

    let small = commands
        .spawn((
            Archipelago2d::new(make_options(ShoreBufferTier::Small.agent_radius())),
            HighSeasEntity,
        ))
        .id();

    let medium = commands
        .spawn((
            Archipelago2d::new(make_options(ShoreBufferTier::Medium.agent_radius())),
            HighSeasEntity,
        ))
        .id();

    let large = commands
        .spawn((
            Archipelago2d::new(make_options(ShoreBufferTier::Large.agent_radius())),
            HighSeasEntity,
        ))
        .id();

    info!(
        "Initialized 3 landmass archipelagos: small={:?}, medium={:?}, large={:?}",
        small, medium, large
    );

    commands.insert_resource(LandmassArchipelagos { small, medium, large });
}

/// Spawns navigation islands from pending nav meshes.
/// Each tier gets its own island attached to its archipelago.
fn spawn_navigation_islands(
    mut commands: Commands,
    pending_meshes: Option<Res<PendingNavMeshes>>,
    archipelagos: Option<Res<LandmassArchipelagos>>,
    mut nav_meshes: ResMut<Assets<NavMesh2d>>,
) {
    let Some(pending) = pending_meshes else {
        warn!("No pending nav meshes available for island creation");
        return;
    };

    let Some(archs) = archipelagos else {
        warn!("No archipelagos available for island creation");
        return;
    };

    // Helper to spawn an island for a tier
    let mut spawn_island = |mesh: &NavigationMesh2d, arch_entity: Entity, tier_name: &str| {
        // Validate and convert to ValidNavigationMesh
        match mesh.clone().validate() {
            Ok(valid_mesh) => {
                let nav_mesh = NavMesh2d {
                    nav_mesh: Arc::new(valid_mesh),
                    // Empty map means all polygon types use default cost of 1.0
                    type_index_to_node_type: std::collections::HashMap::new(),
                };
                let handle = nav_meshes.add(nav_mesh);

                commands.spawn((
                    Island2dBundle {
                        island: Island,
                        archipelago_ref: ArchipelagoRef2d::new(arch_entity),
                        nav_mesh: NavMeshHandle(handle),
                    },
                    Transform::default(),
                    HighSeasEntity,
                ));

                info!("Spawned navigation island for {} tier", tier_name);
            }
            Err(e) => {
                warn!("Failed to validate {} tier nav mesh: {:?}", tier_name, e);
            }
        }
    };

    // Spawn islands for each tier
    if let Some(mesh) = &pending.small {
        spawn_island(mesh, archs.small, "small");
    }

    if let Some(mesh) = &pending.medium {
        spawn_island(mesh, archs.medium, "medium");
    }

    if let Some(mesh) = &pending.large {
        spawn_island(mesh, archs.large, "large");
    }
}

/// Ink color for coastline rendering (dark brown, like old maps)
const COASTLINE_INK_COLOR: Color = Color::srgba(0.2, 0.15, 0.1, 0.8);
/// Stroke width for coastline lines
const COASTLINE_STROKE_WIDTH: f32 = 2.0;

/// Number of decorative waterlines to draw around coastlines
const WATERLINE_COUNT: usize = 3;
/// Distance between waterlines in world units
const WATERLINE_SPACING: f32 = 10.0;

/// Spawns Lyon shape entities for each coastline polygon.
/// Runs when entering HighSeas state.
fn spawn_coastline_shapes(
    mut commands: Commands,
    coastline_data: Res<CoastlineData>,
    existing_shapes: Query<Entity, With<CoastlineShape>>,
) {
    // Don't spawn if already exists
    if !existing_shapes.is_empty() {
        return;
    }

    let mut shapes_spawned = 0;

    for polygon in &coastline_data.polygons {
        if polygon.points.len() < 2 {
            continue;
        }

        // --- 1. Spawn Main Coastline ---
        let mut path_builder = PathBuilder::new();
        let first = polygon.points[0];
        path_builder.move_to(Vec2::new(first.x, first.y));
        for point in polygon.points.iter().skip(1) {
            path_builder.line_to(Vec2::new(point.x, point.y));
        }
        path_builder.close();
        let path = path_builder.build();

        commands.spawn((
            CoastlineShape,
            ShapeBundle {
                path,
                transform: Transform::from_xyz(0.0, 0.0, -8.0), // Between tilemap (-10) and fog (-5)
                ..default()
            },
            Stroke::new(COASTLINE_INK_COLOR, COASTLINE_STROKE_WIDTH),
            HighSeasEntity,
        ));
        shapes_spawned += 1;

        // --- 2. Spawn Waterlines (Ripples) ---
        // Cascade: each waterline offsets from the previous one
        let mut current_points = polygon.points.clone();
        
        for i in 1..=WATERLINE_COUNT {
            // Offset from previous waterline (not original)
            let offset_points = offset_polygon(&current_points, WATERLINE_SPACING);
            
            if offset_points.len() < 3 {
                break; // Polygon collapsed, stop spawning more waterlines
            }

            // Build path for waterline
            let mut wb = PathBuilder::new();
            let p0 = offset_points[0];
            wb.move_to(Vec2::new(p0.x, p0.y));
            for p in offset_points.iter().skip(1) {
                wb.line_to(Vec2::new(p.x, p.y));
            }
            wb.close();
            let water_path = wb.build();

            // Calculate fading color and width
            // Alpha decreases: 0.4 -> 0.27 -> 0.2
            let alpha = 0.5 / (i as f32 + 0.2); 
            let color = Color::srgba(0.2, 0.15, 0.1, alpha);
            // Width decreases slightly
            let width = (COASTLINE_STROKE_WIDTH - (i as f32 * 0.4)).max(0.5);

            commands.spawn((
                CoastlineShape, // Managed by same cleanup/toggle system
                ShapeBundle {
                    path: water_path,
                    // Slightly lower z-index for each successive line
                    transform: Transform::from_xyz(0.0, 0.0, -8.1 - (i as f32 * 0.01)), 
                    ..default()
                },
                Stroke::new(color, width),
                HighSeasEntity,
            ));
            shapes_spawned += 1;
            
            // Update for next iteration
            current_points = offset_points;
        }
    }

    info!("Spawned {} coastline shape entities (including waterlines)", shapes_spawned);
}

/// Marker for elevation decoration entities (hills hachures, mountain peaks).
/// Separate from CoastlineShape but toggles with coastlines.
#[derive(Component)]
pub struct ElevationMarker;

/// Spawns decorative Lyon shapes for hills (hachures) and mountains (peaks).
/// Part of the coastline visibility group.
fn spawn_elevation_markers(
    mut commands: Commands,
    map_data: Res<MapData>,
    existing_markers: Query<Entity, With<ElevationMarker>>,
) {
    use rand::prelude::*;
    use crate::resources::TileType;
    
    // Don't spawn if already exists
    if !existing_markers.is_empty() {
        return;
    }

    const TILE_SIZE: f32 = 64.0; // Correct tile size matching tilemap
    let ink_color = Color::srgba(0.15, 0.12, 0.08, 0.7); // Darker ink with transparency
    
    let mut rng = rand::thread_rng();
    let mut markers_spawned = 0;

    for (x, y, tile) in map_data.iter() {
        let tile_type = tile.tile_type;
        if tile_type != TileType::Hills && tile_type != TileType::Mountains {
            continue;
        }

        let world_x = x as f32 * TILE_SIZE - (map_data.width as f32 * TILE_SIZE / 2.0);
        let world_y = y as f32 * TILE_SIZE - (map_data.height as f32 * TILE_SIZE / 2.0);

        if tile_type == TileType::Hills {
            // SPARSE: Only spawn hachures on 50% of hill tiles
            if !rng.gen_bool(0.5) {
                continue;
            }

            // Draw 1-2 wavy horizontal hachure lines per hills tile
            let num_lines = rng.gen_range(1..=2);
            for line_idx in 0..num_lines {
                let mut path_builder = PathBuilder::new();
                
                // Randomized base y within tile, but keep away from top/bottom edges
                let step = TILE_SIZE / (num_lines as f32 + 1.0);
                let base_y = (line_idx as f32 + 1.0) * step;
                let y_offset: f32 = rng.gen_range(-4.0..4.0); // Scaled jitter
                
                // Add an arc for "high in middle, down on edges" shape
                let arc_height: f32 = rng.gen_range(8.0..12.0); // How pronounced the hill hump is
                
                // Multi-frequency noise for organic look
                let phase: f32 = rng.gen_range(0.0..std::f32::consts::TAU);
                let amp1: f32 = rng.gen_range(2.0..4.0); // Slightly reduced noise amplitude so arc dominates
                let freq1: f32 = rng.gen_range(0.1..0.2); 
                let amp2: f32 = rng.gen_range(1.0..2.0);
                let freq2: f32 = rng.gen_range(0.3..0.6);
                
                // Start point
                let start_val = 16.0; 
                let end_val = 48.0;
                let total_width = end_val - start_val;

                // Move to start with arc + noise
                let t_start = 0.0;
                let arc_start = (t_start * std::f32::consts::PI).sin() * arc_height;
                let noise_start = (start_val * freq1 + phase).sin() * amp1 + (start_val * freq2).cos() * amp2;
                let start_y = base_y + y_offset + noise_start + arc_start;
                path_builder.move_to(Vec2::new(world_x + start_val, world_y + start_y));
                
                // Draw wavy line across tile with arc
                for px in (20..=48).step_by(2) { 
                    let x_f32 = px as f32;
                    let t = (x_f32 - start_val) / total_width; // 0.0 to 1.0
                    
                    let arc = (t * std::f32::consts::PI).sin() * arc_height;
                    let noise = (x_f32 * freq1 + phase).sin() * amp1 + (x_f32 * freq2).cos() * amp2;
                    
                    let local_y = base_y + y_offset + noise + arc;
                    
                    // Clamp y roughly
                    let clamped_y = local_y.clamp(8.0, 56.0);
                    path_builder.line_to(Vec2::new(world_x + x_f32, world_y + clamped_y));
                }
                
                let path = path_builder.build();
                
                // Random position jitter (anywhere within ~2 tiles range)
                let jitter_x = rng.gen_range(-64.0..64.0);
                let jitter_y = rng.gen_range(-64.0..64.0);
                
                commands.spawn((
                    CoastlineShape,
                    ElevationMarker,
                    ShapeBundle {
                        path,
                        // Nudge half-tile down-left + random jitter
                        transform: Transform::from_xyz(-32.0 + jitter_x, -32.0 + jitter_y, -7.9), 
                        ..default()
                    },
                    Stroke::new(ink_color, 2.0), // Thicker stroke for larger resolution
                    HighSeasEntity,
                ));
                markers_spawned += 1;
            }
        } else if tile_type == TileType::Mountains {
            // SPARSE: Only spawn mountains on 70% of mountain tiles
            if !rng.gen_bool(0.7) {
                continue;
            }

            // Draw 1 peak symbol per mountain tile
            let num_peaks = 1; 
            for _ in 0..num_peaks {
                let mut path_builder = PathBuilder::new();
                
                // Center peak within tile, clamped offset
                let peak_x_offset: f32 = TILE_SIZE / 2.0 + rng.gen_range(-4.0..4.0);
                
                let peak_height: f32 = rng.gen_range(24.0..36.0); // Scaled height
                let base_width: f32 = rng.gen_range(20.0..28.0);  // Scaled width
                
                let wave_amp: f32 = rng.gen_range(1.0..2.5);
                let wave_freq: f32 = rng.gen_range(0.1..0.3);
                let phase: f32 = rng.gen_range(0.0..std::f32::consts::TAU);
                
                // Left base point
                let left_x = world_x + peak_x_offset - base_width / 2.0;
                let base_y = world_y + TILE_SIZE / 2.0 - peak_height / 2.0;
                let peak_top_y = base_y + peak_height;
                
                path_builder.move_to(Vec2::new(left_x, base_y));
                
                // Draw left slope
                for i in 1..=8 {
                    let t = i as f32 / 8.0;
                    let px = left_x + (base_width / 2.0) * t + ((i as f32) * wave_freq + phase).sin() * wave_amp;
                    let py = base_y + peak_height * t;
                    path_builder.line_to(Vec2::new(px, py));
                }
                
                // Peak point
                let peak_top = Vec2::new(world_x + peak_x_offset, peak_top_y);
                path_builder.line_to(peak_top);
                
                // Draw right slope
                let right_slope_points: Vec<Vec2> = (0..=8).rev().map(|i| {
                    let t = i as f32 / 8.0;
                    let px = world_x + peak_x_offset + (base_width / 2.0) * (1.0 - t) + ((i as f32) * wave_freq + phase + 1.0).sin() * wave_amp;
                    let py = base_y + peak_height * t;
                    Vec2::new(px, py)
                }).collect();
                
                for p in &right_slope_points {
                    path_builder.line_to(*p);
                }
                
                let path = path_builder.build();

                // Random position jitter for mountains
                let jitter_x = rng.gen_range(-64.0..64.0);
                let jitter_y = rng.gen_range(-64.0..64.0);
                
                commands.spawn((
                    CoastlineShape,
                    ElevationMarker,
                    ShapeBundle {
                        path,
                        transform: Transform::from_xyz(-32.0 + jitter_x, -32.0 + jitter_y, -7.8),
                        ..default()
                    },
                    Stroke::new(ink_color, 2.5), // Thicker stroke
                    HighSeasEntity,
                ));
                markers_spawned += 1;

                // Spawn shading lines on the right slope
                // Simple hatching: draw short horizontal lines from right slope inward
                if rng.gen_bool(0.7) { // 70% chance of shading
                    let mut shade_builder = PathBuilder::new();
                    for i in 1..3 {
                        if i < right_slope_points.len() - 1 {
                            let p = right_slope_points[i];
                            shade_builder.move_to(p);
                            shade_builder.line_to(Vec2::new(p.x - 2.5, p.y - 1.0)); // Hash mark
                        }
                    }
                    commands.spawn((
                        CoastlineShape,
                        ElevationMarker,
                        ShapeBundle {
                            path: shade_builder.build(),
                            transform: Transform::from_xyz(-32.0 + jitter_x, -32.0 + jitter_y, -7.85), // Same jitter as parent
                            ..default()
                        },
                        Stroke::new(Color::srgba(0.15, 0.12, 0.08, 0.5), 1.0), // Fainter ink
                        HighSeasEntity,
                    ));
                    markers_spawned += 1;
                }
            }
        }
    }

    info!("Spawned {} elevation markers (hills/mountains)", markers_spawned);
}

/// Updates coastline visibility based on debug toggle.
fn coastline_visibility_system(
    toggles: Res<DebugToggles>,
    mut query: Query<&mut Visibility, With<CoastlineShape>>,
) {
    if !toggles.is_changed() {
        return;
    }

    let visibility = if toggles.show_coastlines {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    for mut vis in &mut query {
        *vis = visibility;
    }
}

/// Toggles NavMesh debug visualization with F3 key.
fn toggle_navmesh_debug(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut enable_debug: ResMut<EnableLandmassDebug>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        enable_debug.0 = !enable_debug.0;
        info!("NavMesh debug visualization: {}", if enable_debug.0 { "ON" } else { "OFF" });
    }
}

/// Creates a depth/density texture from map data and spawns a stippling overlay.
fn spawn_stipple_overlay(
    commands: &mut Commands,
    map_data: &MapData,
    images: &mut ResMut<Assets<Image>>,
    materials: &mut ResMut<Assets<StipplingMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
) {
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

    let width = map_data.width;
    let height = map_data.height;

    // Create texture data (R8Unorm)
    let mut data = vec![0u8; (width * height) as usize];

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            
            // Calculate density based on tile
            // Land = 0.0 (Clear)
            // Water = density increases near coastline (shallow water)
            let density = if let Some(tile) = map_data.tile(x, y) {
                if tile.tile_type.is_navigable() {
                    // Water - density inversely proportional to depth
                    // Only very shallow water (depth < 0.15) gets dots
                    // Multiply depth by 6 for aggressive falloff
                    // Multiply result by 0.4 for lower base rate
                    ((1.0 - tile.depth * 6.0).clamp(0.0, 1.0) * 0.4 * 255.0) as u8
                } else {
                    // Land
                    0
                }
            } else {
                0
            };
            
            data[idx] = density;
        }
    }

    let image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::R8Unorm,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD | bevy::render::render_asset::RenderAssetUsages::MAIN_WORLD,
    );

    let image_handle = images.add(image);

    let material_handle = materials.add(StipplingMaterial {
        color: LinearRgba::new(0.15, 0.25, 0.4, 0.45), // Subdued nautical blue dots
        dot_spacing: 32.0, // Spacing in world units (half a tile)
        depth_texture: image_handle,
    });

    let mesh_handle = meshes.add(Rectangle::new(width as f32 * 64.0, height as f32 * 64.0));

    commands.spawn((
        Mesh2d(mesh_handle),
        MeshMaterial2d(material_handle),
        // Align with map - offset by half tile down and left
        Transform::from_xyz(-32.0, -32.0, -9.0), // Above map (-10), below ships
        Name::new("StippleOverlay"),
        HighSeasEntity,
    ));
    
    info!("Spawned Stipple Overlay");
}
