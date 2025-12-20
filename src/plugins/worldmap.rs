use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy_ecs_tilemap::prelude::*;
use crate::plugins::core::GameState;
use crate::resources::{MapData, TileType};

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
            // Initialize MapData resource with default test map
            .init_resource::<MapData>()
            .add_systems(Startup, (initialize_test_map, create_tileset_texture))
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

/// Resource holding the procedurally generated tileset image handle
#[derive(Resource)]
pub struct TilesetHandle(pub Handle<Image>);

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
    const NUM_TILES: u32 = 5;
    const TEXTURE_WIDTH: u32 = TILE_SIZE * NUM_TILES;
    const TEXTURE_HEIGHT: u32 = TILE_SIZE;

    // Create RGBA pixel data
    let mut data = vec![0u8; (TEXTURE_WIDTH * TEXTURE_HEIGHT * 4) as usize];

    // Define colors for each tile type (RGBA)
    let colors: [(u8, u8, u8, u8); 5] = [
        (30, 60, 120, 255),    // Index 0: Deep Water - dark blue
        (60, 130, 170, 255),   // Index 1: Shallow Water - teal
        (220, 190, 140, 255),  // Index 2: Sand - tan
        (80, 140, 80, 255),    // Index 3: Land - green
        (120, 80, 50, 255),    // Index 4: Port - brown
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

/// Initializes the MapData resource with a test map.
/// This will be replaced by procedural generation in Epic 3.2.
fn initialize_test_map(mut map_data: ResMut<MapData>) {
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
    map_data: Res<MapData>,
    tileset: Option<Res<TilesetHandle>>,
) {
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
            transform: Transform::from_xyz(0.0, 0.0, -10.0), // Below ships
            ..Default::default()
        },
        WorldMap,
    ));

    info!("World map tilemap spawned: {}x{} tiles", map_size.x, map_size.y);
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
