//! Compass Rose UI component - a traditional 32-point wind rose.
//!
//! Uses Lyon vector graphics rendered via the shared Overlay Camera (RenderLayer 1).
//! Positioned in the bottom-right corner.

use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_prototype_lyon::prelude::*;
use bevy::window::PrimaryWindow;

use crate::plugins::core::GameState;
use crate::plugins::overlay_ui::{UI_LAYER, COLOR_INK, COLOR_PARCHMENT, COLOR_GOLD, COLOR_GOLD_DARK, COLOR_GREEN, COLOR_RED};

pub struct CompassRosePlugin;

impl Plugin for CompassRosePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::HighSeas), spawn_compass_rose)
            .add_systems(Update, update_compass_position.run_if(in_state(GameState::HighSeas)))
            .add_systems(OnExit(GameState::HighSeas), despawn_compass_rose);
    }
}

// Geometry Constants
const OUTER_RING_RADIUS: f32 = 55.0;
const CENTER_RADIUS: f32 = 12.0;
const PRINCIPAL_LENGTH: f32 = 50.0;
const HALF_WIND_LENGTH: f32 = 35.0;
const QUARTER_WIND_LENGTH: f32 = 22.0;

const PRINCIPAL_WIDTH: f32 = 12.0;
const HALF_WIND_WIDTH: f32 = 7.0;
const QUARTER_WIND_WIDTH: f32 = 4.0;

// Offset from bottom-right corner
const MARGIN: Vec2 = Vec2::new(90.0, 90.0);

#[derive(Component)]
pub struct CompassRose;

#[derive(Component)]
pub struct CompassRoseRoot;

fn spawn_compass_rose(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    // Calculate initial position (bottom-right)
    let mut initial_pos = Vec3::new(0.0, 0.0, 0.0);
    if let Ok(window) = window_query.get_single() {
        let half_w = window.width() / 2.0;
        let half_h = window.height() / 2.0;
        initial_pos = Vec3::new(
            half_w - MARGIN.x,
            -half_h + MARGIN.y,
            0.0
        );
    }

    // Spawn Compass Root
    let root = commands.spawn((
        Name::new("Compass Rose Root"),
        CompassRose,
        CompassRoseRoot,
        Transform::from_translation(initial_pos).with_scale(Vec3::splat(0.7)),
        Visibility::Inherited,
        RenderLayers::layer(UI_LAYER),
    )).id();

    // --- Draw Compass Components ---

    // Outer Ring
    let ring = shapes::Circle { radius: OUTER_RING_RADIUS, center: Vec2::ZERO };
    commands.spawn((
        ShapeBundle { path: GeometryBuilder::build_as(&ring), transform: Transform::from_xyz(0.0, 0.0, 0.1), ..default() },
        Stroke::new(COLOR_INK, 2.0),
        CompassRose,
        RenderLayers::layer(UI_LAYER),
    )).set_parent(root);

    // Spikes helper
    let mut helper = |count: i32, length: f32, width: f32, color: Color, z: f32, offset_deg: f32| {
        for i in 0..count {
            let angle = (offset_deg + i as f32 * (360.0 / count as f32)).to_radians();
            let tip = Vec2::new(angle.sin(), angle.cos()) * length;
            let perp = Vec2::new(angle.cos(), -angle.sin());
            let base_center = Vec2::new(angle.sin(), angle.cos()) * (CENTER_RADIUS * 0.8);
            let base_left = base_center - perp * (width / 2.0);
            let base_right = base_center + perp * (width / 2.0);

            let mut pb = PathBuilder::new();
            pb.move_to(tip);
            pb.line_to(base_left);
            pb.line_to(base_right);
            pb.close();

            // Handle alternating colors for principal winds
            let final_color = if count == 8 && offset_deg == 0.0 {
                 if i % 2 == 0 { COLOR_GOLD } else { COLOR_GOLD_DARK }
            } else {
                color
            };

            commands.spawn((
                ShapeBundle { path: pb.build(), transform: Transform::from_xyz(0.0, 0.0, z), ..default() },
                Fill::color(final_color),
                Stroke::new(COLOR_INK, 0.5),
                CompassRose,
                RenderLayers::layer(UI_LAYER),
            )).set_parent(root);
        }
    };

    // Quarter-Winds (Red)
    helper(16, QUARTER_WIND_LENGTH, QUARTER_WIND_WIDTH, COLOR_RED, 0.2, 11.25);
    // Half-Winds (Green)
    helper(8, HALF_WIND_LENGTH, HALF_WIND_WIDTH, COLOR_GREEN, 0.3, 22.5);
    // Principal Winds (Gold/Black)
    helper(8, PRINCIPAL_LENGTH, PRINCIPAL_WIDTH, COLOR_GOLD, 0.4, 0.0);

    // Center Circle
    let center_circle = shapes::Circle { radius: CENTER_RADIUS, center: Vec2::ZERO };
    commands.spawn((
        ShapeBundle { path: GeometryBuilder::build_as(&center_circle), transform: Transform::from_xyz(0.0, 0.0, 0.5), ..default() },
        Fill::color(COLOR_PARCHMENT),
        Stroke::new(COLOR_INK, 1.5),
        CompassRose,
        RenderLayers::layer(UI_LAYER),
    )).set_parent(root);

    // Fleur-de-lis
    spawn_fleur_de_lis(&mut commands, root, Vec2::new(0.0, OUTER_RING_RADIUS + 15.0), 0.6);
    // Cross
    spawn_cross(&mut commands, root, Vec2::new(OUTER_RING_RADIUS + 8.0, 0.0), 0.6);

    info!("Spawned Compass Rose");
}

/// Helper for Fleur-de-lis spawning with RenderLayer
fn spawn_fleur_de_lis(commands: &mut Commands, parent: Entity, position: Vec2, z: f32) {
    let scale = 0.8;
    let mut path = PathBuilder::new();
    // Central petal
    path.move_to(Vec2::new(0.0, 0.0) * scale + position);
    path.cubic_bezier_to(Vec2::new(-4.0, 8.0) * scale + position, Vec2::new(-3.0, 14.0) * scale + position, Vec2::new(0.0, 18.0) * scale + position);
    path.cubic_bezier_to(Vec2::new(3.0, 14.0) * scale + position, Vec2::new(4.0, 8.0) * scale + position, Vec2::new(0.0, 0.0) * scale + position);
    // Left petal
    path.move_to(Vec2::new(-2.0, 2.0) * scale + position);
    path.cubic_bezier_to(Vec2::new(-10.0, 4.0) * scale + position, Vec2::new(-14.0, 10.0) * scale + position, Vec2::new(-10.0, 14.0) * scale + position);
    path.cubic_bezier_to(Vec2::new(-8.0, 10.0) * scale + position, Vec2::new(-6.0, 6.0) * scale + position, Vec2::new(-2.0, 2.0) * scale + position);
    // Right petal
    path.move_to(Vec2::new(2.0, 2.0) * scale + position);
    path.cubic_bezier_to(Vec2::new(10.0, 4.0) * scale + position, Vec2::new(14.0, 10.0) * scale + position, Vec2::new(10.0, 14.0) * scale + position);
    path.cubic_bezier_to(Vec2::new(8.0, 10.0) * scale + position, Vec2::new(6.0, 6.0) * scale + position, Vec2::new(2.0, 2.0) * scale + position);
    // Base bar
    path.move_to(Vec2::new(-6.0, -2.0) * scale + position);
    path.line_to(Vec2::new(6.0, -2.0) * scale + position);
    path.line_to(Vec2::new(6.0, 0.0) * scale + position);
    path.line_to(Vec2::new(-6.0, 0.0) * scale + position);
    path.close();

    commands.spawn((
        ShapeBundle { path: path.build(), transform: Transform::from_xyz(0.0, 0.0, z), ..default() },
        Fill::color(COLOR_GOLD),
        Stroke::new(COLOR_INK, 0.8),
        CompassRose,
        RenderLayers::layer(UI_LAYER),
    )).set_parent(parent);
}

/// Helper for Cross spawning with RenderLayer
fn spawn_cross(commands: &mut Commands, parent: Entity, position: Vec2, z: f32) {
    let arm_length = 6.0;
    let arm_width = 2.0;
    let mut path = PathBuilder::new();
    path.move_to(position + Vec2::new(-arm_width/2.0, -arm_length));
    path.line_to(position + Vec2::new(arm_width/2.0, -arm_length));
    path.line_to(position + Vec2::new(arm_width/2.0, arm_length));
    path.line_to(position + Vec2::new(-arm_width/2.0, arm_length));
    path.close();
    path.move_to(position + Vec2::new(-arm_length, -arm_width/2.0));
    path.line_to(position + Vec2::new(arm_length, -arm_width/2.0));
    path.line_to(position + Vec2::new(arm_length, arm_width/2.0));
    path.line_to(position + Vec2::new(-arm_length, arm_width/2.0));
    path.close();

    commands.spawn((
        ShapeBundle { path: path.build(), transform: Transform::from_xyz(0.0, 0.0, z), ..default() },
        Fill::color(COLOR_INK),
        CompassRose,
        RenderLayers::layer(UI_LAYER),
    )).set_parent(parent);
}


/// Keeps the compass in the bottom-right corner when window is resized.
fn update_compass_position(
    window_query: Query<&Window, (With<PrimaryWindow>, Changed<Window>)>,
    mut root_query: Query<&mut Transform, With<CompassRoseRoot>>,
) {
    let Ok(window) = window_query.get_single() else { return; };
    let Ok(mut transform) = root_query.get_single_mut() else { return; };

    let half_w = window.width() / 2.0;
    let half_h = window.height() / 2.0;
    
    transform.translation.x = half_w - MARGIN.x;
    transform.translation.y = -half_h + MARGIN.y;
}

fn despawn_compass_rose(mut commands: Commands, query: Query<Entity, With<CompassRose>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
