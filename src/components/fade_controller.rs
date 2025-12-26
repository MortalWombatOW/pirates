//! FadeController component for smooth alpha transitions.
//!
//! Generic component usable by any UI element (Cartouche, tooltips, etc.)
//! for fade-in/fade-out animations.

use bevy::prelude::*;

/// Controls alpha fade animation for entities with visual elements.
/// Attach to a root entity; child entities will have their alpha updated
/// by a corresponding apply system.
#[derive(Component, Debug, Clone)]
pub struct FadeController {
    /// Desired alpha value (0.0 = invisible, 1.0 = fully visible).
    pub target_alpha: f32,
    /// Interpolated alpha value, updated each frame by animate_fades system.
    pub current_alpha: f32,
    /// Alpha change rate in units per second.
    pub fade_speed: f32,
}

impl Default for FadeController {
    fn default() -> Self {
        Self::visible()
    }
}

impl FadeController {
    /// Creates a fully visible fade controller.
    pub fn visible() -> Self {
        Self {
            target_alpha: 1.0,
            current_alpha: 1.0,
            fade_speed: 2.0, // Default: 0.5s to fade fully
        }
    }

    /// Creates a fully hidden fade controller.
    pub fn hidden() -> Self {
        Self {
            target_alpha: 0.0,
            current_alpha: 0.0,
            fade_speed: 2.0,
        }
    }

    /// Initiates a fade-in animation over the specified duration.
    pub fn fade_in(&mut self, duration: f32) {
        self.target_alpha = 1.0;
        self.fade_speed = if duration > 0.0 { 1.0 / duration } else { 100.0 };
    }

    /// Initiates a fade-out animation over the specified duration.
    pub fn fade_out(&mut self, duration: f32) {
        self.target_alpha = 0.0;
        self.fade_speed = if duration > 0.0 { 1.0 / duration } else { 100.0 };
    }

    /// Returns true if currently fading (target != current).
    pub fn is_fading(&self) -> bool {
        (self.target_alpha - self.current_alpha).abs() > 0.001
    }

    /// Returns true if fully visible (alpha ~= 1.0).
    pub fn is_visible(&self) -> bool {
        self.current_alpha > 0.999
    }

    /// Returns true if fully hidden (alpha ~= 0.0).
    pub fn is_hidden(&self) -> bool {
        self.current_alpha < 0.001
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visible_default() {
        let fc = FadeController::visible();
        assert_eq!(fc.current_alpha, 1.0);
        assert_eq!(fc.target_alpha, 1.0);
        assert!(fc.is_visible());
        assert!(!fc.is_fading());
    }

    #[test]
    fn test_hidden() {
        let fc = FadeController::hidden();
        assert_eq!(fc.current_alpha, 0.0);
        assert!(fc.is_hidden());
    }

    #[test]
    fn test_fade_out() {
        let mut fc = FadeController::visible();
        fc.fade_out(0.5);
        assert_eq!(fc.target_alpha, 0.0);
        assert_eq!(fc.fade_speed, 2.0); // 1.0 / 0.5
        assert!(fc.is_fading());
    }

    #[test]
    fn test_fade_in() {
        let mut fc = FadeController::hidden();
        fc.fade_in(1.0);
        assert_eq!(fc.target_alpha, 1.0);
        assert_eq!(fc.fade_speed, 1.0);
    }
}
