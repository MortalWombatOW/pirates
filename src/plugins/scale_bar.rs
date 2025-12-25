//! Scale Bar UI component - an authentic 18th-century nautical chart scale.
//!
//! Uses Lyon vector graphics rendered via the shared Overlay Camera (RenderLayer 1).
//! Positioned in the bottom-left corner, complementing the Compass Rose in the bottom-right.
//! 
//! The bar dynamically adjusts width so each segment represents an exact round distance.
//! Uses transform scaling for smooth transitions without geometry rebuild.

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
            .init_resource::<ScaleBarConfig>()
            .add_systems(OnEnter(GameState::HighSeas), spawn_scale_bar)
            .add_systems(Update, (
                update_scale_bar_position,
                update_scale_bar_scale,
            ).run_if(in_state(GameState::HighSeas)))
            .add_systems(OnExit(GameState::HighSeas), despawn_scale_bar);
    }
}

// Geometry Constants (base values at scale 1.0)
const BASE_BAR_WIDTH: f32 = 150.0;
const BAR_HEIGHT: f32 = 8.0;
const SEGMENT_COUNT: u32 = 5;
const END_CAP_HEIGHT: f32 = 14.0;

// Width bounds (screen pixels)
const MIN_BAR_WIDTH: f32 = 80.0;
const MAX_BAR_WIDTH: f32 = 220.0;

// Offset from bottom-left corner
const MARGIN: Vec2 = Vec2::new(100.0, 60.0);

// Scale conversion: 1 tile = 16px, assume 1 tile ≈ 1 nautical mile
const PIXELS_PER_MILE: f32 = 16.0;

// Nice segment distances to choose from (in miles)
const NICE_DISTANCES: [f32; 9] = [0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0];

#[derive(Resource)]
pub struct ScaleBarConfig {
    pub segment_miles: f32,
    pub total_miles: f32,
    pub width_scale: f32,
}

impl Default for ScaleBarConfig {
    fn default() -> Self {
        Self {
            segment_miles: 2.0,
            total_miles: 10.0,
            width_scale: 1.0,
        }
    }
}

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
            -half_w + MARGIN.x + BASE_BAR_WIDTH / 2.0,
            -half_h + MARGIN.y,
            0.0
        );
    }

    // Spawn Scale Bar Root (this is what we scale)
    let root = commands.spawn((
        Name::new("Scale Bar Root"),
        ScaleBar,
        ScaleBarRoot,
        Transform::from_translation(initial_pos),
        Visibility::Inherited,
        RenderLayers::layer(UI_LAYER),
    )).id();

    // --- Draw Scale Bar Components ---
    let segment_width = BASE_BAR_WIDTH / SEGMENT_COUNT as f32;
    let start_x = -BASE_BAR_WIDTH / 2.0;

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
        extents: Vec2::new(BASE_BAR_WIDTH, BAR_HEIGHT),
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
    for x_offset in [-BASE_BAR_WIDTH / 2.0, BASE_BAR_WIDTH / 2.0] {
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

    // Dynamic label
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
    window_query: Query<&Window, With<PrimaryWindow>>,
    config: Res<ScaleBarConfig>,
    mut root_query: Query<&mut Transform, With<ScaleBarRoot>>,
) {
    let Ok(window) = window_query.get_single() else { return; };
    let Ok(mut transform) = root_query.get_single_mut() else { return; };

    let half_w = window.width() / 2.0;
    let half_h = window.height() / 2.0;
    
    // Adjust X position based on current bar width
    let current_width = BASE_BAR_WIDTH * config.width_scale;
    transform.translation.x = -half_w + MARGIN.x + current_width / 2.0;
    transform.translation.y = -half_h + MARGIN.y;
}

/// Updates the scale bar width and label based on camera zoom.
fn update_scale_bar_scale(
    camera_query: Query<&OrthographicProjection, (With<MainCamera>, Changed<OrthographicProjection>)>,
    mut config: ResMut<ScaleBarConfig>,
    mut root_query: Query<&mut Transform, With<ScaleBarRoot>>,
    mut label_query: Query<(&mut Text2d, &mut Transform), (With<ScaleBarLabel>, Without<ScaleBarRoot>)>,
) {
    let Ok(projection) = camera_query.get_single() else { return; };
    
    // Calculate optimal configuration
    let (width_scale, segment_miles, total_miles) = calculate_bar_config(projection.scale);
    
    // Update config
    config.width_scale = width_scale;
    config.segment_miles = segment_miles;
    config.total_miles = total_miles;
    
    // Scale the root transform (X axis only to change width)
    if let Ok(mut transform) = root_query.get_single_mut() {
        transform.scale.x = width_scale;
    }
    
    // Update label text and counter-scale to keep text size consistent
    if let Ok((mut text, mut label_transform)) = label_query.get_single_mut() {
        // Update text
        if total_miles >= 1.0 {
            text.0 = format!("{} MILES", total_miles as i32);
        } else {
            text.0 = format!("{:.1} MILES", total_miles);
        }
        
        // Counter-scale label so text doesn't stretch
        label_transform.scale.x = 1.0 / width_scale;
    }
}

/// Calculates optimal bar configuration for the given projection scale.
/// Returns (width_scale, segment_miles, total_miles).
fn calculate_bar_config(projection_scale: f32) -> (f32, f32, f32) {
    // Try each nice distance and find one that gives a reasonable bar width
    for &segment_miles in &NICE_DISTANCES {
        let total_miles = segment_miles * SEGMENT_COUNT as f32;
        let world_width = total_miles * PIXELS_PER_MILE;
        let screen_width = world_width / projection_scale;
        
        if screen_width >= MIN_BAR_WIDTH && screen_width <= MAX_BAR_WIDTH {
            let width_scale = screen_width / BASE_BAR_WIDTH;
            return (width_scale, segment_miles, total_miles);
        }
    }
    
    // Fallback: clamp to bounds and calculate segment distance
    let base_world_width = BASE_BAR_WIDTH * projection_scale;
    let base_miles = base_world_width / PIXELS_PER_MILE;
    let segment_miles = base_miles / SEGMENT_COUNT as f32;
    
    (1.0, segment_miles, base_miles)
}

fn despawn_scale_bar(mut commands: Commands, query: Query<Entity, With<ScaleBar>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
