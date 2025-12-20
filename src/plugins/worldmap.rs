use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use crate::plugins::core::GameState;

/// Plugin managing the world map tilemap for the High Seas view.
/// 
/// This plugin handles:
/// - Loading the tileset texture
/// - Spawning the tilemap entity and tiles
/// - Managing map visibility based on game state
pub struct WorldMapPlugin;

impl Plugin for WorldMapPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::HighSeas), spawn_test_tilemap)
            .add_systems(OnExit(GameState::HighSeas), despawn_tilemap);
    }
}

/// Marker component for the world map tilemap
#[derive(Component)]
pub struct WorldMap;

/// Marker component for world map tiles
#[derive(Component)]
pub struct WorldMapTile;

/// Spawns a test tilemap for development purposes.
/// This will later be replaced by procedural generation in task 3.2.
fn spawn_test_tilemap(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Load the tileset texture
    let texture_handle: Handle<Image> = asset_server.load("tilemaps/tileset.png");

    // Define map dimensions
    let map_size = TilemapSize { x: 32, y: 32 };

    // Create a tilemap entity early to associate tiles with it
    let tilemap_entity = commands.spawn_empty().id();

    // Create tile storage
    let mut tile_storage = TileStorage::empty(map_size);

    // Spawn tiles - for now, use tile index 0 (water) for all tiles
    // The Kenney tilesheet has water tiles in the first few indices
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            
            // Determine tile texture index based on position
            // For testing: mostly water (index 0), with some land
            let texture_index = if (x > 10 && x < 20) && (y > 10 && y < 20) {
                // Land in the center - use various land/island tiles
                // Kenney tileset has various tiles, index 50+ are often land variants
                50u32
            } else {
                // Water tile - index 0 in the Kenney tileset is water
                0u32
            };

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

    info!("World map tilemap spawned with {} tiles", map_size.x * map_size.y);
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
