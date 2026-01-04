use bevy::prelude::*;
use bevy::utils::HashMap;

/// Represents a single cell in the water simulation grid.
#[derive(Clone, Copy, Debug, Default)]
pub struct WaterCell {
    /// Surface height relative to sea level (0.0).
    /// Positive = wave crest, Negative = trough.
    pub height: f32,
    
    /// Horizontal flow velocity.
    // pub velocity: Vec2, // Legacy colocated
    
    /// Staggered Grid: Flow velocity across the East face (+X).
    pub flow_right: f32,
    
    /// Staggered Grid: Flow velocity across the South face (+Y).
    pub flow_down: f32,
    
    /// Bathymetry (terrain height). 
    /// If height < bottom, cell is dry (should handle gracefully).
    pub bottom: f32,
}

impl WaterCell {
    pub fn new(bottom: f32) -> Self {
        Self {
            height: 0.0,
            flow_right: 0.0,
            flow_down: 0.0,
            bottom,
        }
    }

    /// Total water column height (surface - bottom).
    pub fn depth(&self) -> f32 {
        (self.height - self.bottom).max(0.0)
    }
}

/// The global quadtree resource managing water cells.
/// Uses a Linear Quadtree structure with Morton Codes.
/// 
/// Key: (Depth, MortonCode)
/// allowing for overlapping nodes during refinement transitions (though solver should enforce unique leaves).
#[derive(Resource)]
pub struct OceanQuadtree {
    pub nodes: HashMap<(u8, u32), WaterCell>,
    pub max_depth: u8,
    /// Physical size of the entire grid domain (Depth 0).
    pub domain_size: f32,
}

impl Default for OceanQuadtree {
    fn default() -> Self {
        Self {
            nodes: HashMap::default(),
            max_depth: 12, // Increased for extreme resolution (was 10)
            domain_size: 1024.0, 
        }
    }
}

impl OceanQuadtree {
    /// Calculate the physical size of a cell at a given depth.
    pub fn cell_size(&self, depth: u8) -> f32 {
        self.domain_size / (1u32 << depth) as f32
    }
}

pub struct OceanGridPlugin;

impl Plugin for OceanGridPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OceanQuadtree>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_size() {
        let qt = OceanQuadtree {
            domain_size: 100.0,
            ..default()
        };

        assert_eq!(qt.cell_size(0), 100.0);
        assert_eq!(qt.cell_size(1), 50.0);
        assert_eq!(qt.cell_size(2), 25.0);
    }
}
