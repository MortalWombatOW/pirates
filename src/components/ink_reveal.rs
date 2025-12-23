//! InkReveal Component
//!
//! Tracks ink spread animation progress for fog reveal effects.

use bevy::prelude::*;

/// Component that tracks the progress of an ink reveal animation.
/// Attached to tile entities when they are first explored.
#[derive(Component, Debug, Clone)]
pub struct InkReveal {
    /// The tile position being revealed.
    pub tile_pos: IVec2,
    /// Time when the reveal animation started.
    pub start_time: f32,
    /// Animation duration in seconds.
    pub duration: f32,
}

impl InkReveal {
    /// Creates a new ink reveal animation for a tile.
    pub fn new(tile_pos: IVec2, start_time: f32) -> Self {
        Self {
            tile_pos,
            start_time,
            duration: 0.5, // 0.5 second animation
        }
    }

    /// Returns the animation progress (0.0 = start, 1.0 = complete).
    pub fn progress(&self, current_time: f32) -> f32 {
        let elapsed = current_time - self.start_time;
        (elapsed / self.duration).clamp(0.0, 1.0)
    }

    /// Returns true if the animation is complete.
    pub fn is_complete(&self, current_time: f32) -> bool {
        self.progress(current_time) >= 1.0
    }

    /// Returns an eased progress value (ease-out for fast start, slow finish).
    pub fn eased_progress(&self, current_time: f32) -> f32 {
        let t = self.progress(current_time);
        // Ease-out cubic: 1 - (1-t)^3
        1.0 - (1.0 - t).powi(3)
    }
}
