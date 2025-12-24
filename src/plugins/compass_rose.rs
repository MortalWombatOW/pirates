//! Compass Rose UI component - a traditional 32-point wind rose.
//!
//! Uses Lyon vector graphics attached as a child of the camera for
//! jitter-free positioning in the bottom-right corner.

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::plugins::core::GameState;

pub struct CompassRosePlugin;

impl Plugin for CompassRosePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::HighSeas), spawn_compass_rose)
            .add_systems(Update, update_compass_scale.run_if(in_state(GameState::HighSeas)))
            .add_systems(OnExit(GameState::HighSeas), despawn_compass_rose);
    }
}

// --- Color Constants ---
/// Gold/Yellow for principal winds (N, NE, E, SE, S, SW, W, NW)
const COLOR_PRINCIPAL: Color = Color::srgba(0.79, 0.64, 0.15, 1.0);
/// Black outline for principal wind spikes
const COLOR_PRINCIPAL_DARK: Color = Color::srgba(0.15, 0.12, 0.08, 1.0);
/// Green for half-winds (NNE, ENE, ESE, etc.)
const COLOR_HALF_WIND: Color = Color::srgba(0.18, 0.35, 0.24, 1.0);
/// Red for quarter-winds (finest divisions)
const COLOR_QUARTER_WIND: Color = Color::srgba(0.69, 0.19, 0.19, 1.0);
/// Parchment/cream for background elements
const COLOR_PARCHMENT: Color = Color::srgba(0.94, 0.90, 0.78, 1.0);
/// Dark ink for outlines
const COLOR_INK: Color = Color::srgba(0.15, 0.12, 0.08, 1.0);

// --- Geometry Constants ---
const OUTER_RING_RADIUS: f32 = 55.0;
const CENTER_RADIUS: f32 = 12.0;

// Spike lengths
const PRINCIPAL_LENGTH: f32 = 50.0;
const HALF_WIND_LENGTH: f32 = 35.0;
const QUARTER_WIND_LENGTH: f32 = 22.0;

// Spike widths at base
const PRINCIPAL_WIDTH: f32 = 12.0;
const HALF_WIND_WIDTH: f32 = 7.0;
const QUARTER_WIND_WIDTH: f32 = 4.0;

// Position offset from camera center (bottom-right)
const COMPASS_OFFSET: Vec3 = Vec3::new(350.0, -250.0, 50.0);

/// Marker component for compass rose entities
#[derive(Component)]
pub struct CompassRose;

/// Marker for the compass rose root entity (for scale updates)
#[derive(Component)]
pub struct CompassRoseRoot;

/// Spawns the compass rose as a child of the camera.
fn spawn_compass_rose(
    mut commands: Commands,
    camera_query: Query<Entity, With<Camera2d>>,
) {
    let Ok(camera_entity) = camera_query.get_single() else {
        warn!("No camera found for compass rose");
        return;
    };

    let center = Vec2::ZERO;
    
    // Parent entity for all compass parts - positioned relative to camera
    let compass_entity = commands.spawn((
        Name::new("Compass Rose"),
        CompassRose,
        CompassRoseRoot,
        Transform::from_translation(COMPASS_OFFSET).with_scale(Vec3::splat(0.7)),
        Visibility::Inherited,
        InheritedVisibility::default(),
        ViewVisibility::default(),
        GlobalTransform::default(),
    )).set_parent(camera_entity).id();

    // --- Layer 1: Outer Ring ---
    let ring = shapes::Circle {
        radius: OUTER_RING_RADIUS,
        center,
    };
    commands.spawn((
        CompassRose,
        ShapeBundle {
            path: GeometryBuilder::build_as(&ring),
            transform: Transform::from_xyz(0.0, 0.0, 0.1),
            ..default()
        },
        Stroke::new(COLOR_INK, 2.0),
    )).set_parent(compass_entity);

    // --- Layer 2: Quarter-Winds (16 red spikes) ---
    for i in 0..16 {
        let angle = (11.25 + i as f32 * 22.5).to_radians();
        spawn_spike(&mut commands, compass_entity, center, angle, QUARTER_WIND_LENGTH, QUARTER_WIND_WIDTH, COLOR_QUARTER_WIND, 0.2);
    }

    // --- Layer 3: Half-Winds (8 green spikes) ---
    for i in 0..8 {
        let angle = (22.5 + i as f32 * 45.0).to_radians();
        spawn_spike(&mut commands, compass_entity, center, angle, HALF_WIND_LENGTH, HALF_WIND_WIDTH, COLOR_HALF_WIND, 0.3);
    }

    // --- Layer 4: Principal Winds (8 gold/dark spikes) ---
    for i in 0..8 {
        let angle = (i as f32 * 45.0).to_radians();
        let color = if i % 2 == 0 { COLOR_PRINCIPAL } else { COLOR_PRINCIPAL_DARK };
        spawn_spike(&mut commands, compass_entity, center, angle, PRINCIPAL_LENGTH, PRINCIPAL_WIDTH, color, 0.4);
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
            transform: Transform::from_xyz(0.0, 0.0, 0.5),
            ..default()
        },
        Fill::color(COLOR_PARCHMENT),
        Stroke::new(COLOR_INK, 1.5),
    )).set_parent(compass_entity);

    // --- Layer 6: Fleur-de-lis at North ---
    spawn_fleur_de_lis(&mut commands, compass_entity, Vec2::new(0.0, OUTER_RING_RADIUS + 15.0), 0.6);

    // --- Layer 7: Cross at East ---
    spawn_cross(&mut commands, compass_entity, Vec2::new(OUTER_RING_RADIUS + 8.0, 0.0), 0.6);

    info!("Spawned Compass Rose (attached to camera)");
}

/// Spawns a triangular spike.
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
    let tip = center + Vec2::new(angle.sin(), angle.cos()) * length;
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

/// Spawns a procedural fleur-de-lis symbol.
fn spawn_fleur_de_lis(commands: &mut Commands, parent: Entity, position: Vec2, z: f32) {
    let scale = 0.8;
    let mut path = PathBuilder::new();

    // Central petal
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

    // Left petal
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

    // Right petal
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

    // Base bar
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

/// Spawns a small cross symbol.
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

/// Base scale for the compass (applied at projection.scale = 1.0)
const COMPASS_BASE_SCALE: f32 = 0.7;

/// Updates compass scale to counteract camera zoom, keeping constant screen size.
/// Also updates position offset to stay in corner regardless of zoom.
fn update_compass_scale(
    camera_query: Query<&OrthographicProjection, With<Camera2d>>,
    mut compass_query: Query<&mut Transform, With<CompassRoseRoot>>,
) {
    let Ok(projection) = camera_query.get_single() else {
        return;
    };
    let Ok(mut compass_transform) = compass_query.get_single_mut() else {
        return;
    };

    // Counter-scale: when camera zooms out (scale increases), compass gets bigger to compensate
    let counter_scale = projection.scale * COMPASS_BASE_SCALE;
    compass_transform.scale = Vec3::splat(counter_scale);
    
    // Also update position to stay in corner (offset scales with projection)
    let offset = Vec3::new(
        350.0 * projection.scale,
        -250.0 * projection.scale,
        50.0,
    );
    compass_transform.translation = offset;
}

/// Despawns the compass rose.
fn despawn_compass_rose(
    mut commands: Commands,
    query: Query<Entity, With<CompassRose>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

