use bevy::prelude::*;

/// Resource representing the current wind conditions in the game world.
/// 
/// Wind affects:
/// - Navigation speed in High Seas (travel with wind = faster)
/// - Ship movement in Combat (downwind = faster, upwind = slower)
#[derive(Resource, Debug, Clone, Copy)]
pub struct Wind {
    /// Wind direction in radians (0 = East, PI/2 = North, etc.)
    pub direction: f32,
    /// Wind strength from 0.0 (calm) to 1.0 (gale)
    pub strength: f32,
}

impl Default for Wind {
    fn default() -> Self {
        Self {
            direction: 0.0,       // Default: blowing East
            strength: 0.5,        // Default: moderate wind
        }
    }
}

impl Wind {
    /// Returns the wind direction as a unit vector.
    pub fn direction_vec(&self) -> Vec2 {
        Vec2::new(self.direction.cos(), self.direction.sin())
    }
    
    /// Returns the wind velocity (direction * strength).
    pub fn velocity(&self) -> Vec2 {
        self.direction_vec() * self.strength
    }
    
    /// Returns a human-readable cardinal direction (N, NE, E, etc.)
    pub fn cardinal_direction(&self) -> &'static str {
        let deg = self.direction.to_degrees().rem_euclid(360.0);
        match deg as u32 {
            0..=22 | 338..=360 => "E",
            23..=67 => "NE",
            68..=112 => "N",
            113..=157 => "NW",
            158..=202 => "W",
            203..=247 => "SW",
            248..=292 => "S",
            293..=337 => "SE",
            _ => "?",
        }
    }
}
