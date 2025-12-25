use bevy::prelude::*;

/// Represents a tile type in the world map.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TileType {
    /// Deep water - open ocean
    DeepWater,
    /// Shallow water - near coastlines
    ShallowWater,
    /// Land - impassable terrain
    Land,
    /// Sand/Beach - coastal transition
    Sand,
    /// Port - docking location
    Port,
}

impl TileType {
    /// Returns the tileset texture index for this tile type.
    /// Based on the procedural tileset layout in WorldMapPlugin:
    /// Index 0: Deep Water (dark blue)
    /// Index 1: Shallow Water (teal)  
    /// Index 2: Sand (tan)
    /// Index 3: Land (green)
    /// Index 4: Port (brown)
    pub fn texture_index(&self) -> u32 {
        match self {
            TileType::DeepWater => 0,
            TileType::ShallowWater => 1,
            TileType::Sand => 2,
            TileType::Land => 3,
            TileType::Port => 4,
        }
    }

    /// Returns whether ships can pass through this tile.
    pub fn is_navigable(&self) -> bool {
        matches!(self, TileType::DeepWater | TileType::ShallowWater)
    }

    /// Returns whether this tile is a docking location.
    pub fn is_port(&self) -> bool {
        matches!(self, TileType::Port)
    }
}

/// Represents a single tile in the world map.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tile {
    pub tile_type: TileType,
    /// Water depth (0.0 for land).
    /// Used for stippling and future depth-based mechanics.
    pub depth: f32,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            tile_type: TileType::DeepWater,
            depth: 0.0,
        }
    }
}

impl Tile {
    pub fn new(tile_type: TileType, depth: f32) -> Self {
        Self { tile_type, depth }
    }
    
    pub fn from_type(tile_type: TileType) -> Self {
        Self { tile_type, depth: 0.0 }
    }
}

/// Resource containing the world map tile data.
/// 
/// This is the source of truth for tile types and is used by:
/// - WorldMapPlugin to render the tilemap
/// - NavigationSystem for pathfinding
/// - FogOfWar for visibility tracking
#[derive(Resource)]
pub struct MapData {
    /// Width of the map in tiles
    pub width: u32,
    /// Height of the map in tiles
    pub height: u32,
    /// Flat array of tiles, stored row-major (y * width + x)
    tiles: Vec<Tile>,
}

impl MapData {
    /// Creates a new MapData with the given dimensions, filled with deep water (depth 0.0).
    pub fn new(width: u32, height: u32) -> Self {
        let tiles = vec![Tile::default(); (width * height) as usize];
        Self { width, height, tiles }
    }

    /// Creates a new MapData with the given dimensions and default tile.
    pub fn new_filled(width: u32, height: u32, default_tile: Tile) -> Self {
        let tiles = vec![default_tile; (width * height) as usize];
        Self { width, height, tiles }
    }

    /// Gets the tile at the given coordinates.
    /// Returns None if coordinates are out of bounds.
    pub fn tile(&self, x: u32, y: u32) -> Option<&Tile> {
        if x < self.width && y < self.height {
            Some(&self.tiles[(y * self.width + x) as usize])
        } else {
            None
        }
    }

    /// Gets a mutable reference to the tile at the given coordinates.
    pub fn tile_mut(&mut self, x: u32, y: u32) -> Option<&mut Tile> {
        if x < self.width && y < self.height {
            Some(&mut self.tiles[(y * self.width + x) as usize])
        } else {
            None
        }
    }

    /// Sets the tile at the given coordinates.
    /// Returns true if successful, false if out of bounds.
    pub fn set_tile(&mut self, x: u32, y: u32, tile: Tile) -> bool {
        if x < self.width && y < self.height {
            self.tiles[(y * self.width + x) as usize] = tile;
            true
        } else {
            false
        }
    }

    /// Helper to set just the tile type, preserving existing depth (or using 0.0 if not yet set?)
    /// Actually, just overwriting depth to 0.0 if we fundamentally change type might differ on context,
    /// but usually we set type during gen. We'll preserve depth for safety or reset?
    /// Let's make it simple: updates type, keeps depth.
    pub fn set_type(&mut self, x: u32, y: u32, tile_type: TileType) -> bool {
        if let Some(tile) = self.tile_mut(x, y) {
            tile.tile_type = tile_type;
            true
        } else {
            false
        }
    }

    /// Returns an iterator over all tiles with their coordinates.
    pub fn iter(&self) -> impl Iterator<Item = (u32, u32, &Tile)> + '_ {
        (0..self.height).flat_map(move |y| {
            (0..self.width).map(move |x| (x, y, &self.tiles[(y * self.width + x) as usize]))
        })
    }

    /// Returns whether the given coordinates are within the map bounds.
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height
    }

    /// Returns whether the tile at the given coordinates is navigable.
    pub fn is_navigable(&self, x: u32, y: u32) -> bool {
        self.tile(x, y).map(|t| t.tile_type.is_navigable()).unwrap_or(false)
    }
}

impl Default for MapData {
    fn default() -> Self {
        // Default to a 64x64 map for testing
        Self::new(64, 64)
    }
}
