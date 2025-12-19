use bevy::prelude::*;
use crate::components::TargetComponent;

/// Resource tracking the global status of cannons for the player.
#[derive(Resource, Debug, Reflect)]
pub struct CannonState {
    /// Time remaining until next shot can be fired (seconds).
    pub cooldown_remaining: f32,
    /// Default cooldown duration for cannons.
    pub base_cooldown: f32,
    /// Currently selected component to target.
    pub current_target: TargetComponent,
}

impl Default for CannonState {
    fn default() -> Self {
        Self {
            cooldown_remaining: 0.0,
            base_cooldown: 2.0, // Slower reload: 1 shot every 2 seconds
            current_target: TargetComponent::Hull,
        }
    }
}
