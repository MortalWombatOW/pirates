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
    /// Based on the Kenney pirate-pack tilesheet layout.
    pub fn texture_index(&self) -> u32 {
        match self {
            TileType::DeepWater => 0,      // First water tile
            TileType::ShallowWater => 1,   // Lighter water variant
            TileType::Land => 50,          // Land/grass tile
            TileType::Sand => 17,          // Sand/beach tile
            TileType::Port => 50,          // Use land tile for ports (will add marker later)
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
    /// Flat array of tile types, stored row-major (y * width + x)
    tiles: Vec<TileType>,
}

impl MapData {
    /// Creates a new MapData with the given dimensions, filled with deep water.
    pub fn new(width: u32, height: u32) -> Self {
        let tiles = vec![TileType::DeepWater; (width * height) as usize];
        Self { width, height, tiles }
    }

    /// Creates a new MapData with the given dimensions and default tile type.
    pub fn new_filled(width: u32, height: u32, default_tile: TileType) -> Self {
        let tiles = vec![default_tile; (width * height) as usize];
        Self { width, height, tiles }
    }

    /// Gets the tile at the given coordinates.
    /// Returns None if coordinates are out of bounds.
    pub fn get(&self, x: u32, y: u32) -> Option<TileType> {
        if x < self.width && y < self.height {
            Some(self.tiles[(y * self.width + x) as usize])
        } else {
            None
        }
    }

    /// Sets the tile at the given coordinates.
    /// Returns true if successful, false if out of bounds.
    pub fn set(&mut self, x: u32, y: u32, tile: TileType) -> bool {
        if x < self.width && y < self.height {
            self.tiles[(y * self.width + x) as usize] = tile;
            true
        } else {
            false
        }
    }

    /// Returns an iterator over all tiles with their coordinates.
    pub fn iter(&self) -> impl Iterator<Item = (u32, u32, TileType)> + '_ {
        (0..self.height).flat_map(move |y| {
            (0..self.width).map(move |x| (x, y, self.tiles[(y * self.width + x) as usize]))
        })
    }

    /// Returns whether the given coordinates are within the map bounds.
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height
    }

    /// Returns whether the tile at the given coordinates is navigable.
    pub fn is_navigable(&self, x: u32, y: u32) -> bool {
        self.get(x, y).map(|t| t.is_navigable()).unwrap_or(false)
    }
}

impl Default for MapData {
    fn default() -> Self {
        // Default to a 64x64 map for testing
        Self::new(64, 64)
    }
}
