// Hit flash component for damage visual feedback.

use bevy::prelude::*;

/// Component for temporary hit flash effect on ships.
/// When a ship takes damage, its sprite briefly flashes white then fades back.
#[derive(Component)]
pub struct HitFlash {
    /// Timer controlling flash duration.
    pub timer: Timer,
    /// Original sprite color to restore after flash.
    pub original_color: Color,
}

impl HitFlash {
    /// Creates a new hit flash with the specified duration.
    pub fn new(duration: f32, original_color: Color) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            original_color,
        }
    }

    /// Default flash duration (0.3 seconds for good visibility).
    pub const DEFAULT_DURATION: f32 = 0.3;
}
