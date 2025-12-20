use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use crate::plugins::core::GameState;
use crate::resources::MapData;

/// Plugin managing the world map tilemap for the High Seas view.
/// 
/// This plugin handles:
/// - Loading the tileset texture
/// - Spawning the tilemap entity and tiles from MapData
/// - Managing map visibility based on game state
pub struct WorldMapPlugin;

impl Plugin for WorldMapPlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize MapData resource with default test map
            .init_resource::<MapData>()
            .add_systems(Startup, initialize_test_map)
            .add_systems(OnEnter(GameState::HighSeas), spawn_tilemap_from_map_data)
            .add_systems(OnExit(GameState::HighSeas), despawn_tilemap);
    }
}

/// Marker component for the world map tilemap
#[derive(Component)]
pub struct WorldMap;

/// Marker component for world map tiles
#[derive(Component)]
pub struct WorldMapTile;

/// Initializes the MapData resource with a test map.
/// This will be replaced by procedural generation in Epic 3.2.
fn initialize_test_map(mut map_data: ResMut<MapData>) {
    use crate::resources::TileType;

    // Reset to a clean 64x64 map
    *map_data = MapData::new(64, 64);

    // Create a simple island in the center
    for x in 25..40 {
        for y in 25..40 {
            // Create circular island shape
            let center_x = 32.0;
            let center_y = 32.0;
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance < 5.0 {
                // Inner land
                map_data.set(x, y, TileType::Land);
            } else if distance < 7.0 {
                // Transition to sand/beach
                map_data.set(x, y, TileType::Sand);
            } else if distance < 9.0 {
                // Shallow water around island
                map_data.set(x, y, TileType::ShallowWater);
            }
        }
    }

    // Add a second smaller island
    for x in 10..20 {
        for y in 45..55 {
            let center_x = 15.0;
            let center_y = 50.0;
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance < 3.0 {
                map_data.set(x, y, TileType::Land);
            } else if distance < 4.5 {
                map_data.set(x, y, TileType::Sand);
            } else if distance < 6.0 {
                map_data.set(x, y, TileType::ShallowWater);
            }
        }
    }

    // Add a port on the main island
    map_data.set(37, 32, TileType::Port);

    info!("Test map initialized: {}x{} tiles", map_data.width, map_data.height);
}

/// Spawns the tilemap from MapData resource.
fn spawn_tilemap_from_map_data(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    map_data: Res<MapData>,
) {
    // Load the tileset texture
    let texture_handle: Handle<Image> = asset_server.load("tilemaps/tileset.png");

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

    // Tile size from Kenney tilesheet (64x64 pixels, no margin)
    let tile_size = TilemapTileSize { x: 64.0, y: 64.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default(); // Square tiles

    commands.entity(tilemap_entity).insert((
        TilemapBundle {
            grid_size,
            map_type,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle),
            tile_size,
            transform: Transform::from_xyz(0.0, 0.0, -10.0), // Below ships
            ..Default::default()
        },
        WorldMap,
    ));

    info!("World map tilemap spawned from MapData: {}x{} tiles", map_size.x, map_size.y);
}

/// Despawns the world map when leaving HighSeas state.
fn despawn_tilemap(
    mut commands: Commands,
    tilemap_query: Query<Entity, With<WorldMap>>,
    tile_query: Query<Entity, With<WorldMapTile>>,
) {
    // Despawn all tiles first
    for entity in tile_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Despawn the tilemap entity
    for entity in tilemap_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    info!("World map tilemap despawned");
}
