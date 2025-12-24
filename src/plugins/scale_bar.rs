//! Scale Bar UI component - an authentic 18th-century nautical chart scale.
//!
//! Uses Lyon vector graphics rendered via the shared Overlay Camera (RenderLayer 1).
//! Positioned in the bottom-left corner, complementing the Compass Rose in the bottom-right.
//! The label dynamically updates based on camera zoom level.

use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
use bevy_prototype_lyon::prelude::*;

use crate::plugins::core::{GameState, MainCamera};
use crate::plugins::overlay_ui::{UI_LAYER, COLOR_INK, COLOR_PARCHMENT};

pub struct ScaleBarPlugin;

impl Plugin for ScaleBarPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::HighSeas), spawn_scale_bar)
            .add_systems(Update, (
                update_scale_bar_position,
                update_scale_bar_label,
            ).run_if(in_state(GameState::HighSeas)))
            .add_systems(OnExit(GameState::HighSeas), despawn_scale_bar);
    }
}

// Geometry Constants
const BAR_WIDTH: f32 = 150.0;
const BAR_HEIGHT: f32 = 8.0;
const SEGMENT_COUNT: u32 = 5;
const END_CAP_HEIGHT: f32 = 14.0;

// Offset from bottom-left corner
const MARGIN: Vec2 = Vec2::new(100.0, 60.0);

// Scale conversion: 1 tile = 16px, assume 1 tile ≈ 1 nautical mile
const PIXELS_PER_MILE: f32 = 16.0;

#[derive(Component)]
pub struct ScaleBar;

#[derive(Component)]
pub struct ScaleBarRoot;

#[derive(Component)]
pub struct ScaleBarLabel;

fn spawn_scale_bar(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    // Calculate initial position (bottom-left)
    let mut initial_pos = Vec3::new(0.0, 0.0, 0.0);
    if let Ok(window) = window_query.get_single() {
        let half_w = window.width() / 2.0;
        let half_h = window.height() / 2.0;
        initial_pos = Vec3::new(
            -half_w + MARGIN.x + BAR_WIDTH / 2.0,
            -half_h + MARGIN.y,
            0.0
        );
    }

    // Spawn Scale Bar Root
    let root = commands.spawn((
        Name::new("Scale Bar Root"),
        ScaleBar,
        ScaleBarRoot,
        Transform::from_translation(initial_pos),
        Visibility::Inherited,
        RenderLayers::layer(UI_LAYER),
    )).id();

    // --- Draw Scale Bar Components ---

    let segment_width = BAR_WIDTH / SEGMENT_COUNT as f32;
    let start_x = -BAR_WIDTH / 2.0;

    // Alternating segments
    for i in 0..SEGMENT_COUNT {
        let color = if i % 2 == 0 { COLOR_INK } else { COLOR_PARCHMENT };
        let x = start_x + i as f32 * segment_width + segment_width / 2.0;
        
        let segment = shapes::Rectangle {
            extents: Vec2::new(segment_width, BAR_HEIGHT),
            origin: RectangleOrigin::Center,
            ..default()
        };
        
        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&segment),
                transform: Transform::from_xyz(x, 0.0, 0.1),
                ..default()
            },
            Fill::color(color),
            Stroke::new(COLOR_INK, 1.0),
            ScaleBar,
            RenderLayers::layer(UI_LAYER),
        )).set_parent(root);
    }

    // Outer border
    let border = shapes::Rectangle {
        extents: Vec2::new(BAR_WIDTH, BAR_HEIGHT),
        origin: RectangleOrigin::Center,
        ..default()
    };
    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&border),
            transform: Transform::from_xyz(0.0, 0.0, 0.2),
            ..default()
        },
        Stroke::new(COLOR_INK, 1.5),
        ScaleBar,
        RenderLayers::layer(UI_LAYER),
    )).set_parent(root);

    // End caps (decorative vertical lines)
    for x_offset in [-BAR_WIDTH / 2.0, BAR_WIDTH / 2.0] {
        let mut path = PathBuilder::new();
        path.move_to(Vec2::new(x_offset, -END_CAP_HEIGHT / 2.0));
        path.line_to(Vec2::new(x_offset, END_CAP_HEIGHT / 2.0));
        
        commands.spawn((
            ShapeBundle {
                path: path.build(),
                transform: Transform::from_xyz(0.0, 0.0, 0.3),
                ..default()
            },
            Stroke::new(COLOR_INK, 2.0),
            ScaleBar,
            RenderLayers::layer(UI_LAYER),
        )).set_parent(root);
    }

    // Dynamic label (will be updated by update_scale_bar_label system)
    let font = asset_server.load("fonts/Quintessential-Regular.ttf");
    commands.spawn((
        Text2d::new("10 MILES"),
        TextFont {
            font,
            font_size: 12.0,
            ..default()
        },
        TextColor(COLOR_INK),
        Transform::from_xyz(0.0, BAR_HEIGHT / 2.0 + 10.0, 0.3),
        ScaleBar,
        ScaleBarLabel,
        RenderLayers::layer(UI_LAYER),
    )).set_parent(root);

    info!("Spawned Scale Bar");
}

/// Keeps the scale bar in the bottom-left corner when window is resized.
fn update_scale_bar_position(
    window_query: Query<&Window, (With<PrimaryWindow>, Changed<Window>)>,
    mut root_query: Query<&mut Transform, With<ScaleBarRoot>>,
) {
    let Ok(window) = window_query.get_single() else { return; };
    let Ok(mut transform) = root_query.get_single_mut() else { return; };

    let half_w = window.width() / 2.0;
    let half_h = window.height() / 2.0;
    
    transform.translation.x = -half_w + MARGIN.x + BAR_WIDTH / 2.0;
    transform.translation.y = -half_h + MARGIN.y;
}

/// Updates the scale bar label based on the main camera's zoom level.
fn update_scale_bar_label(
    camera_query: Query<&OrthographicProjection, (With<MainCamera>, Changed<OrthographicProjection>)>,
    mut label_query: Query<&mut Text2d, With<ScaleBarLabel>>,
) {
    let Ok(projection) = camera_query.get_single() else { return; };
    let Ok(mut text) = label_query.get_single_mut() else { return; };

    // Calculate world distance the bar represents at current zoom
    let world_width = BAR_WIDTH * projection.scale;
    let miles = world_width / PIXELS_PER_MILE;
    
    // Round to nice values for readability
    let nice_miles = round_to_nice_value(miles);
    
    // Update label text
    if nice_miles >= 1.0 {
        text.0 = format!("{} MILES", nice_miles as i32);
    } else {
        text.0 = format!("{:.1} MILES", nice_miles);
    }
}

/// Rounds a value to a "nice" number for display (1, 2, 5, 10, 20, 50, etc.)
fn round_to_nice_value(value: f32) -> f32 {
    if value <= 0.0 { return 1.0; }
    
    let magnitude = 10_f32.powf(value.log10().floor());
    let normalized = value / magnitude;
    
    let nice = if normalized < 1.5 {
        1.0
    } else if normalized < 3.5 {
        2.0
    } else if normalized < 7.5 {
        5.0
    } else {
        10.0
    };
    
    nice * magnitude
}

fn despawn_scale_bar(mut commands: Commands, query: Query<Entity, With<ScaleBar>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
