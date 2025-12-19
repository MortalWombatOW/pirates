use bevy::prelude::*;

/// Represents the health state of a ship's three primary components.
/// Damage to each component leads to different debuffs:
/// - **Sails**: Reduce `MaxSpeed` proportionally.
/// - **Rudder**: Reduce `TurnRate` proportionally.
/// - **Hull**: Reduce both slightly, and add `WaterIntake` component.
#[derive(Component, Debug, Clone)]
pub struct Health {
    /// Current sail hitpoints. Damage reduces max speed.
    pub sails: f32,
    /// Maximum sail hitpoints.
    pub sails_max: f32,
    /// Current rudder hitpoints. Damage reduces turn rate.
    pub rudder: f32,
    /// Maximum rudder hitpoints.
    pub rudder_max: f32,
    /// Current hull hitpoints. At 0, ship is destroyed.
    pub hull: f32,
    /// Maximum hull hitpoints.
    pub hull_max: f32,
}

impl Health {
    /// Creates a new Health component with the specified max values.
    /// Current values are initialized to their maximums.
    pub fn new(sails_max: f32, rudder_max: f32, hull_max: f32) -> Self {
        Self {
            sails: sails_max,
            sails_max,
            rudder: rudder_max,
            rudder_max,
            hull: hull_max,
            hull_max,
        }
    }

    /// Returns the ratio of current sails to max sails (0.0 to 1.0).
    pub fn sails_ratio(&self) -> f32 {
        if self.sails_max > 0.0 {
            (self.sails / self.sails_max).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Returns the ratio of current rudder to max rudder (0.0 to 1.0).
    pub fn rudder_ratio(&self) -> f32 {
        if self.rudder_max > 0.0 {
            (self.rudder / self.rudder_max).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Returns the ratio of current hull to max hull (0.0 to 1.0).
    pub fn hull_ratio(&self) -> f32 {
        if self.hull_max > 0.0 {
            (self.hull / self.hull_max).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Returns true if the ship is destroyed (hull HP <= 0).
    pub fn is_destroyed(&self) -> bool {
        self.hull <= 0.0
    }
}

impl Default for Health {
    fn default() -> Self {
        Self::new(100.0, 100.0, 100.0)
    }
}
