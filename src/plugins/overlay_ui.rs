//! Shared utilities for overlay UI elements rendered on RenderLayer 1.
//!
//! Provides a single overlay camera and common constants for cartography-style UI
//! elements like the CompassRose and ScaleBar.

use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::render::camera::ClearColorConfig;

use crate::plugins::core::GameState;
use crate::components::HighSeasEntity;

pub struct OverlayUiPlugin;

impl Plugin for OverlayUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::HighSeas), spawn_overlay_camera)
            .add_systems(OnExit(GameState::HighSeas), despawn_overlay_camera);
    }
}

// --- Public Constants ---

/// RenderLayer for overlay UI elements.
pub const UI_LAYER: usize = 1;

/// Ink color for strokes and text (dark brown).
pub const COLOR_INK: Color = Color::srgba(0.15, 0.12, 0.08, 1.0);

/// Parchment color for fills and backgrounds.
pub const COLOR_PARCHMENT: Color = Color::srgba(0.94, 0.90, 0.78, 1.0);

/// Gold color for decorative elements.
pub const COLOR_GOLD: Color = Color::srgba(0.79, 0.64, 0.15, 1.0);

/// Dark gold/brown for alternating patterns.
pub const COLOR_GOLD_DARK: Color = Color::srgba(0.15, 0.12, 0.08, 1.0);

/// Green for half-wind markers.
pub const COLOR_GREEN: Color = Color::srgba(0.18, 0.35, 0.24, 1.0);

/// Red for quarter-wind markers.
pub const COLOR_RED: Color = Color::srgba(0.69, 0.19, 0.19, 1.0);

// --- Components ---

/// Marker for the shared overlay camera.
#[derive(Component)]
pub struct OverlayCamera;

// --- Systems ---

fn spawn_overlay_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 1, // Render after main camera (0)
            clear_color: ClearColorConfig::None,
            ..default()
        },
        RenderLayers::layer(UI_LAYER),
        OverlayCamera,
        HighSeasEntity,
    ));
    info!("Spawned Overlay UI Camera");
}

fn despawn_overlay_camera(mut commands: Commands, query: Query<Entity, With<OverlayCamera>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
