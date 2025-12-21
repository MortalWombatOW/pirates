use bevy::prelude::*;
use crate::resources::WorldClock;

/// System that advances the world clock on every FixedUpdate tick.
/// 
/// Runs unconditionally (not gated by GameState) to ensure consistent
/// time progression across all game states.
/// 
/// At 60Hz FixedUpdate:
/// - 1 hour passes every ~1 real second
/// - 1 day passes every ~24 real seconds
pub fn world_tick_system(mut world_clock: ResMut<WorldClock>) {
    world_clock.advance();
}
