//! Compass Rose UI component - a traditional 32-point wind rose.
//!
//! Displays in the top-right corner of the High Seas view.

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use std::f32::consts::PI;

use crate::plugins::core::GameState;

pub struct CompassRosePlugin;

impl Plugin for CompassRosePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::HighSeas), spawn_compass_rose)
            .add_systems(Update, update_compass_position.run_if(in_state(GameState::HighSeas)))
            .add_systems(OnExit(GameState::HighSeas), despawn_compass_rose);
    }
}

/// Marker for the root compass rose entity (used for positioning)
#[derive(Component)]
pub struct CompassRoseRoot;

// --- Color Constants ---
// Traditional compass rose colors (can be refactored to a theme resource later)

/// Gold/Yellow for principal winds (N, NE, E, SE, S, SW, W, NW)
const COLOR_PRINCIPAL: Color = Color::srgba(0.79, 0.64, 0.15, 1.0); // #C9A227
/// Black outline for principal wind spikes
const COLOR_PRINCIPAL_DARK: Color = Color::srgba(0.15, 0.12, 0.08, 1.0);
/// Green for half-winds (NNE, ENE, ESE, etc.)
const COLOR_HALF_WIND: Color = Color::srgba(0.18, 0.35, 0.24, 1.0); // #2D5A3D
/// Red for quarter-winds (finest divisions)
const COLOR_QUARTER_WIND: Color = Color::srgba(0.69, 0.19, 0.19, 1.0); // #B03030
/// Parchment/cream for background elements
const COLOR_PARCHMENT: Color = Color::srgba(0.94, 0.90, 0.78, 1.0);
/// Dark ink for outlines and text
const COLOR_INK: Color = Color::srgba(0.15, 0.12, 0.08, 1.0);

// --- Geometry Constants ---
const COMPASS_SIZE: f32 = 120.0; // Overall diameter
const OUTER_RING_RADIUS: f32 = 55.0;
const CENTER_RADIUS: f32 = 12.0;

// Spike lengths (as fraction of radius to outer ring)
const PRINCIPAL_LENGTH: f32 = 50.0;
const HALF_WIND_LENGTH: f32 = 35.0;
const QUARTER_WIND_LENGTH: f32 = 22.0;

// Spike widths at base
const PRINCIPAL_WIDTH: f32 = 12.0;
const HALF_WIND_WIDTH: f32 = 7.0;
const QUARTER_WIND_WIDTH: f32 = 4.0;

/// Marker component for compass rose entities
#[derive(Component)]
pub struct CompassRose;

/// Spawns the compass rose UI in the top-right corner.
fn spawn_compass_rose(mut commands: Commands) {
    let center = Vec2::ZERO;
    
    // Parent entity for all compass parts (positioned relative to camera)
    let compass_entity = commands.spawn((
        Name::new("Compass Rose"),
        CompassRose,
        CompassRoseRoot,
        SpatialBundle {
            transform: Transform::from_xyz(0.0, 0.0, 100.0), // High Z for UI layer
            visibility: Visibility::Inherited,
            ..default()
        },
    )).id();

    // --- Layer 1: Outer Ring ---
    let ring = shapes::Circle {
        radius: OUTER_RING_RADIUS,
        center,
    };
    commands.spawn((
        CompassRose,
        ShapeBundle {
            path: GeometryBuilder::build_as(&ring),
            transform: Transform::from_xyz(0.0, 0.0, 100.0),
            ..default()
        },
        Stroke::new(COLOR_INK, 2.0),
    )).set_parent(compass_entity);

    // --- Layer 2: Quarter-Winds (16 red spikes) ---
    // Angles: 11.25°, 33.75°, 56.25°, ... (offset by 11.25° from cardinals)
    for i in 0..16 {
        let angle = (11.25 + i as f32 * 22.5).to_radians();
        spawn_spike(&mut commands, compass_entity, center, angle, QUARTER_WIND_LENGTH, QUARTER_WIND_WIDTH, COLOR_QUARTER_WIND, 100.1);
    }

    // --- Layer 3: Half-Winds (8 green spikes) ---
    // Angles: 22.5°, 67.5°, 112.5°, ... (between principals)
    for i in 0..8 {
        let angle = (22.5 + i as f32 * 45.0).to_radians();
        spawn_spike(&mut commands, compass_entity, center, angle, HALF_WIND_LENGTH, HALF_WIND_WIDTH, COLOR_HALF_WIND, 100.2);
    }

    // --- Layer 4: Principal Winds (8 gold spikes) ---
    // Angles: 0° (N), 45° (NE), 90° (E), 135° (SE), 180° (S), 225° (SW), 270° (W), 315° (NW)
    for i in 0..8 {
        let angle = (i as f32 * 45.0).to_radians();
        // Alternate gold and dark for visual depth
        let color = if i % 2 == 0 { COLOR_PRINCIPAL } else { COLOR_PRINCIPAL_DARK };
        spawn_spike(&mut commands, compass_entity, center, angle, PRINCIPAL_LENGTH, PRINCIPAL_WIDTH, color, 100.3);
    }

    // --- Layer 5: Center Circle ---
    let center_circle = shapes::Circle {
        radius: CENTER_RADIUS,
        center,
    };
    commands.spawn((
        CompassRose,
        ShapeBundle {
            path: GeometryBuilder::build_as(&center_circle),
            transform: Transform::from_xyz(0.0, 0.0, 100.4),
            ..default()
        },
        Fill::color(COLOR_PARCHMENT),
        Stroke::new(COLOR_INK, 1.5),
    )).set_parent(compass_entity);

    // --- Layer 6: Fleur-de-lis at North ---
    spawn_fleur_de_lis(&mut commands, compass_entity, Vec2::new(0.0, OUTER_RING_RADIUS + 15.0), 100.5);

    // --- Layer 7: Cross at East ---
    spawn_cross(&mut commands, compass_entity, Vec2::new(OUTER_RING_RADIUS + 8.0, 0.0), 100.5);

    info!("Spawned Compass Rose");
}

/// Spawns a triangular spike (isoceles triangle pointing outward).
fn spawn_spike(
    commands: &mut Commands,
    parent: Entity,
    center: Vec2,
    angle: f32,
    length: f32,
    base_width: f32,
    color: Color,
    z: f32,
) {
    // Tip of spike
    let tip = center + Vec2::new(angle.sin(), angle.cos()) * length;
    
    // Base corners (perpendicular to spike direction)
    let perp = Vec2::new(angle.cos(), -angle.sin());
    let base_center = center + Vec2::new(angle.sin(), angle.cos()) * (CENTER_RADIUS * 0.8);
    let base_left = base_center - perp * (base_width / 2.0);
    let base_right = base_center + perp * (base_width / 2.0);

    let mut path_builder = PathBuilder::new();
    path_builder.move_to(tip);
    path_builder.line_to(base_left);
    path_builder.line_to(base_right);
    path_builder.close();

    commands.spawn((
        CompassRose,
        ShapeBundle {
            path: path_builder.build(),
            transform: Transform::from_xyz(0.0, 0.0, z),
            ..default()
        },
        Fill::color(color),
        Stroke::new(COLOR_INK, 0.5),
    )).set_parent(parent);
}

/// Spawns a procedural fleur-de-lis symbol using bezier curves.
fn spawn_fleur_de_lis(commands: &mut Commands, parent: Entity, position: Vec2, z: f32) {
    let scale = 0.8;
    let mut path = PathBuilder::new();

    // Central petal (pointing up)
    path.move_to(Vec2::new(0.0, 0.0) * scale + position);
    path.cubic_bezier_to(
        Vec2::new(-4.0, 8.0) * scale + position,
        Vec2::new(-3.0, 14.0) * scale + position,
        Vec2::new(0.0, 18.0) * scale + position,
    );
    path.cubic_bezier_to(
        Vec2::new(3.0, 14.0) * scale + position,
        Vec2::new(4.0, 8.0) * scale + position,
        Vec2::new(0.0, 0.0) * scale + position,
    );

    // Left petal (curving left and up)
    path.move_to(Vec2::new(-2.0, 2.0) * scale + position);
    path.cubic_bezier_to(
        Vec2::new(-10.0, 4.0) * scale + position,
        Vec2::new(-14.0, 10.0) * scale + position,
        Vec2::new(-10.0, 14.0) * scale + position,
    );
    path.cubic_bezier_to(
        Vec2::new(-8.0, 10.0) * scale + position,
        Vec2::new(-6.0, 6.0) * scale + position,
        Vec2::new(-2.0, 2.0) * scale + position,
    );

    // Right petal (mirror of left)
    path.move_to(Vec2::new(2.0, 2.0) * scale + position);
    path.cubic_bezier_to(
        Vec2::new(10.0, 4.0) * scale + position,
        Vec2::new(14.0, 10.0) * scale + position,
        Vec2::new(10.0, 14.0) * scale + position,
    );
    path.cubic_bezier_to(
        Vec2::new(8.0, 10.0) * scale + position,
        Vec2::new(6.0, 6.0) * scale + position,
        Vec2::new(2.0, 2.0) * scale + position,
    );

    // Base horizontal bar
    path.move_to(Vec2::new(-6.0, -2.0) * scale + position);
    path.line_to(Vec2::new(6.0, -2.0) * scale + position);
    path.line_to(Vec2::new(6.0, 0.0) * scale + position);
    path.line_to(Vec2::new(-6.0, 0.0) * scale + position);
    path.close();

    commands.spawn((
        CompassRose,
        ShapeBundle {
            path: path.build(),
            transform: Transform::from_xyz(0.0, 0.0, z),
            ..default()
        },
        Fill::color(COLOR_PRINCIPAL),
        Stroke::new(COLOR_INK, 0.8),
    )).set_parent(parent);
}

/// Spawns a small decorative cross symbol.
fn spawn_cross(commands: &mut Commands, parent: Entity, position: Vec2, z: f32) {
    let arm_length = 6.0;
    let arm_width = 2.0;

    let mut path = PathBuilder::new();
    
    // Vertical bar
    path.move_to(position + Vec2::new(-arm_width/2.0, -arm_length));
    path.line_to(position + Vec2::new(arm_width/2.0, -arm_length));
    path.line_to(position + Vec2::new(arm_width/2.0, arm_length));
    path.line_to(position + Vec2::new(-arm_width/2.0, arm_length));
    path.close();

    // Horizontal bar
    path.move_to(position + Vec2::new(-arm_length, -arm_width/2.0));
    path.line_to(position + Vec2::new(arm_length, -arm_width/2.0));
    path.line_to(position + Vec2::new(arm_length, arm_width/2.0));
    path.line_to(position + Vec2::new(-arm_length, arm_width/2.0));
    path.close();

    commands.spawn((
        CompassRose,
        ShapeBundle {
            path: path.build(),
            transform: Transform::from_xyz(0.0, 0.0, z),
            ..default()
        },
        Fill::color(COLOR_INK),
    )).set_parent(parent);
}

/// Offset from top-right corner of screen (in world units, scaled by zoom)
const COMPASS_SCREEN_OFFSET: Vec2 = Vec2::new(-100.0, -100.0);

/// Updates compass position to stay in top-right corner relative to camera.
fn update_compass_position(
    camera_query: Query<(&Transform, &OrthographicProjection), With<Camera2d>>,
    mut compass_query: Query<&mut Transform, (With<CompassRoseRoot>, Without<Camera2d>)>,
) {
    let Ok((camera_transform, projection)) = camera_query.get_single() else {
        return;
    };
    let Ok(mut compass_transform) = compass_query.get_single_mut() else {
        return;
    };

    // Calculate screen corners in world space
    let camera_pos = camera_transform.translation.truncate();
    let scale = projection.scale;
    
    // Top-right corner offset (assuming 1920x1080 viewport roughly)
    // We'll approximate based on projection area
    let half_width = projection.area.width() / 2.0;
    let half_height = projection.area.height() / 2.0;
    
    let target_pos = camera_pos + Vec2::new(half_width + COMPASS_SCREEN_OFFSET.x, half_height + COMPASS_SCREEN_OFFSET.y);
    
    compass_transform.translation.x = target_pos.x;
    compass_transform.translation.y = target_pos.y;
}

/// Despawns the compass rose when leaving High Seas.
fn despawn_compass_rose(
    mut commands: Commands,
    query: Query<Entity, With<CompassRose>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
