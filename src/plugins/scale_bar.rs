//! Scale Bar UI component - an authentic 18th-century nautical chart scale.
//!
//! Uses Lyon vector graphics rendered via the Overlay Camera (RenderLayer 1).
//! Positioned in the bottom-left corner, complementing the Compass Rose in the bottom-right.

use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
use bevy_prototype_lyon::prelude::*;

use crate::plugins::core::GameState;

pub struct ScaleBarPlugin;

impl Plugin for ScaleBarPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::HighSeas), spawn_scale_bar)
            .add_systems(Update, update_scale_bar_position.run_if(in_state(GameState::HighSeas)))
            .add_systems(OnExit(GameState::HighSeas), despawn_scale_bar);
    }
}

// --- Constants ---
const UI_LAYER: usize = 1;

// Color Constants (matching compass rose palette)
const COLOR_INK: Color = Color::srgba(0.15, 0.12, 0.08, 1.0);
const COLOR_PARCHMENT: Color = Color::srgba(0.94, 0.90, 0.78, 1.0);

// Geometry Constants
const BAR_WIDTH: f32 = 150.0;
const BAR_HEIGHT: f32 = 8.0;
const SEGMENT_COUNT: u32 = 5;
const END_CAP_HEIGHT: f32 = 14.0;

// Offset from bottom-left corner
const MARGIN: Vec2 = Vec2::new(100.0, 60.0);

#[derive(Component)]
pub struct ScaleBar;

#[derive(Component)]
pub struct ScaleBarRoot;

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
        SpatialBundle {
            transform: Transform::from_translation(initial_pos),
            visibility: Visibility::Inherited,
            ..default()
        },
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

    // "SCALE OF MILES" label
    let font = asset_server.load("fonts/Quintessential-Regular.ttf");
    commands.spawn((
        Text2d::new("SCALE OF MILES"),
        TextFont {
            font,
            font_size: 12.0,
            ..default()
        },
        TextColor(COLOR_INK),
        Transform::from_xyz(0.0, BAR_HEIGHT / 2.0 + 10.0, 0.3),
        ScaleBar,
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

fn despawn_scale_bar(mut commands: Commands, query: Query<Entity, With<ScaleBar>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
