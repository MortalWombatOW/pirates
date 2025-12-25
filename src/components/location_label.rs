//! Location label components for authentic nautical chart styling.
//!
//! Port names are rendered perpendicular to coastlines with importance-based sizing,
//! mimicking 18th-century cartographic conventions.

use bevy::prelude::*;

/// Importance rank for label sizing. Higher importance = larger text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LabelImportance {
    /// Capital ports - largest text (24pt)
    Major,
    /// Regular ports - standard text (18pt)
    #[default]
    Standard,
    /// Small settlements - smallest text (14pt)
    Minor,
}

impl LabelImportance {
    /// Returns the font size for this importance level.
    pub fn font_size(&self) -> f32 {
        match self {
            LabelImportance::Major => 24.0,
            LabelImportance::Standard => 18.0,
            LabelImportance::Minor => 14.0,
        }
    }
}

/// Marks an entity as having a location label rendered on the nautical chart.
/// Labels extend perpendicular to the nearest coastline, in authentic cartographic style.
#[derive(Component, Debug, Clone)]
pub struct LocationLabel {
    /// Display name for the location.
    pub name: String,
    /// Importance level controlling text size.
    pub importance: LabelImportance,
    /// Rotation angle in radians (perpendicular to coastline, pointing inland).
    pub angle: f32,
}

impl LocationLabel {
    /// Creates a new location label with computed angle.
    pub fn new(name: impl Into<String>, importance: LabelImportance, angle: f32) -> Self {
        Self {
            name: name.into(),
            importance,
            angle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_sizes() {
        assert_eq!(LabelImportance::Major.font_size(), 24.0);
        assert_eq!(LabelImportance::Standard.font_size(), 18.0);
        assert_eq!(LabelImportance::Minor.font_size(), 14.0);
    }

    #[test]
    fn test_label_creation() {
        let label = LocationLabel::new("Port Royal", LabelImportance::Major, 1.57);
        assert_eq!(label.name, "Port Royal");
        assert_eq!(label.importance, LabelImportance::Major);
        assert!((label.angle - 1.57).abs() < 0.001);
    }
}
