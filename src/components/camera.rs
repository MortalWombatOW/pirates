// Camera-related components for visual effects.

use bevy::prelude::*;

/// Component for camera shake effects.
/// Uses a "trauma" system where trauma accumulates and decays over time.
/// Shake intensity is trauma² for more natural feel.
#[derive(Component, Default)]
pub struct CameraShake {
    /// Current trauma level (0.0 to 1.0, clamped)
    pub trauma: f32,
    /// How fast trauma decays per second
    pub decay_rate: f32,
    /// Maximum position offset in pixels
    pub max_offset: f32,
    /// Maximum rotation offset in radians
    pub max_rotation: f32,
    /// Internal time accumulator for noise sampling
    pub noise_time: f32,
}

impl CameraShake {
    /// Creates a new CameraShake with default settings.
    pub fn new() -> Self {
        Self {
            trauma: 0.0,
            decay_rate: 1.5,
            max_offset: 8.0,
            max_rotation: 0.03, // ~2 degrees
            noise_time: 0.0,
        }
    }

    /// Adds trauma to the camera shake.
    /// Trauma is clamped to 1.0 maximum.
    pub fn add_trauma(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).min(1.0);
    }

    /// Returns the current shake intensity (trauma²).
    pub fn shake_intensity(&self) -> f32 {
        self.trauma * self.trauma
    }

    /// Decays trauma over time.
    pub fn decay(&mut self, delta_seconds: f32) {
        self.trauma = (self.trauma - self.decay_rate * delta_seconds).max(0.0);
        self.noise_time += delta_seconds;
    }
}
