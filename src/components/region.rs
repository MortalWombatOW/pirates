//! Region-responsive components and resources for location-based UI triggers.
//!
//! Provides `RegionResponsive` component to link UI elements to named regions,
//! and `CurrentRegion` resource to track player location.

use bevy::prelude::*;

/// Links an entity to a named region for automatic fade triggers.
/// When the player enters/exits the specified region, the entity's
/// `FadeController` will be updated accordingly.
#[derive(Component, Debug, Clone)]
pub struct RegionResponsive {
    /// Name of the region this entity responds to.
    pub region_name: String,
    /// If true, fade in when entering region; if false, fade out.
    pub fade_in_on_enter: bool,
    /// Fade duration in seconds.
    pub fade_duration: f32,
}

impl RegionResponsive {
    /// Creates a component that shows the entity when entering the named region.
    pub fn show_on_enter(region_name: impl Into<String>) -> Self {
        Self {
            region_name: region_name.into(),
            fade_in_on_enter: true,
            fade_duration: 0.5,
        }
    }

    /// Creates a component that hides the entity when entering the named region.
    pub fn hide_on_enter(region_name: impl Into<String>) -> Self {
        Self {
            region_name: region_name.into(),
            fade_in_on_enter: false,
            fade_duration: 0.5,
        }
    }

    /// Sets custom fade duration.
    pub fn with_duration(mut self, duration: f32) -> Self {
        self.fade_duration = duration;
        self
    }
}

/// Resource tracking the player's current region.
/// Updated by `update_current_region` system based on player position.
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct CurrentRegion {
    /// Name of the current region, or None if in open sea.
    pub name: Option<String>,
}

impl CurrentRegion {
    /// Creates a new CurrentRegion with the specified name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
        }
    }

    /// Creates an empty CurrentRegion (open sea).
    pub fn none() -> Self {
        Self { name: None }
    }

    /// Returns true if the player is in the specified region.
    pub fn is_in(&self, region_name: &str) -> bool {
        self.name.as_deref() == Some(region_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_on_enter() {
        let resp = RegionResponsive::show_on_enter("Caribbean Sea");
        assert_eq!(resp.region_name, "Caribbean Sea");
        assert!(resp.fade_in_on_enter);
    }

    #[test]
    fn test_hide_on_enter() {
        let resp = RegionResponsive::hide_on_enter("port_area");
        assert!(!resp.fade_in_on_enter);
    }

    #[test]
    fn test_current_region_is_in() {
        let region = CurrentRegion::new("Caribbean Sea");
        assert!(region.is_in("Caribbean Sea"));
        assert!(!region.is_in("Atlantic"));
    }
}
