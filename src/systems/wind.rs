use bevy::prelude::*;
use crate::resources::Wind;

/// System that simulates gradual wind changes over time.
/// 
/// Wind direction and strength slowly oscillate to create a dynamic weather feel.
pub fn wind_system(
    mut wind: ResMut<Wind>,
    time: Res<Time>,
) {
    let elapsed = time.elapsed_secs();
    
    // Slowly oscillate wind direction (full rotation every ~10 minutes)
    let direction_base = elapsed * 0.01; // Very slow rotation
    let direction_wobble = (elapsed * 0.1).sin() * 0.3; // Small oscillations
    wind.direction = direction_base + direction_wobble;
    
    // Oscillate wind strength between 0.3 and 0.8
    let strength_base = 0.55;
    let strength_variation = (elapsed * 0.05).sin() * 0.25;
    wind.strength = (strength_base + strength_variation).clamp(0.2, 0.9);
}
