use bevy::prelude::*;

/// Enum representing targetable ship components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum TargetComponent {
    Sails,
    Rudder,
    Hull,
}

impl Default for TargetComponent {
    fn default() -> Self {
        Self::Hull
    }
}

/// Component applied to cannonball projectiles.
#[derive(Component, Debug, Clone, Reflect)]
pub struct Projectile {
    pub damage: f32,
    pub target: TargetComponent,
}
