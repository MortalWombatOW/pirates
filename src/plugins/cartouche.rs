//! Cartouche UI component - a decorative baroque-style frame for the map title.
//!
//! Uses Lyon vector graphics rendered via the shared Overlay Camera (RenderLayer 1).
//! Positioned at the top-center of the screen.

use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
use bevy_prototype_lyon::prelude::*;

use crate::plugins::core::GameState;
use crate::plugins::overlay_ui::{UI_LAYER, COLOR_INK, COLOR_PARCHMENT, COLOR_GOLD};
use crate::components::fade_controller::FadeController;

pub struct CartouchePlugin;

impl Plugin for CartouchePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::HighSeas), spawn_cartouche)
            .add_systems(Update, (
                update_cartouche_position,
                apply_cartouche_fade,
            ).run_if(in_state(GameState::HighSeas)))
            .add_systems(OnExit(GameState::HighSeas), despawn_cartouche);
    }
}

// Geometry Constants
const FRAME_WIDTH: f32 = 280.0;
const FRAME_HEIGHT: f32 = 90.0;
const CORNER_RADIUS: f32 = 12.0;
const MARGIN_TOP: f32 = 60.0;

/// Marker for all cartouche entities.
#[derive(Component)]
pub struct Cartouche;

/// Marker for the cartouche root entity (for position updates).
#[derive(Component)]
pub struct CartoucheRoot;

fn spawn_cartouche(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    // Calculate initial position (top-center)
    let mut initial_pos = Vec3::new(0.0, 0.0, 0.0);
    if let Ok(window) = window_query.get_single() {
        let half_h = window.height() / 2.0;
        initial_pos = Vec3::new(0.0, half_h - MARGIN_TOP, 0.0);
    }

    // Spawn Cartouche Root
    let root = commands.spawn((
        Name::new("Cartouche Root"),
        Cartouche,
        CartoucheRoot,
        FadeController::visible(),
        Transform::from_translation(initial_pos),
        Visibility::Inherited,
        RenderLayers::layer(UI_LAYER),
    )).id();

    // --- Draw Cartouche Frame ---

    // Outer baroque frame with rounded corners
    spawn_baroque_frame(&mut commands, root);

    // Inner parchment panel
    let inner = shapes::RoundedPolygon {
        points: vec![
            Vec2::new(-FRAME_WIDTH / 2.0 + 8.0, -FRAME_HEIGHT / 2.0 + 8.0),
            Vec2::new(FRAME_WIDTH / 2.0 - 8.0, -FRAME_HEIGHT / 2.0 + 8.0),
            Vec2::new(FRAME_WIDTH / 2.0 - 8.0, FRAME_HEIGHT / 2.0 - 8.0),
            Vec2::new(-FRAME_WIDTH / 2.0 + 8.0, FRAME_HEIGHT / 2.0 - 8.0),
        ],
        radius: CORNER_RADIUS - 4.0,
        closed: true,
    };
    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&inner),
            transform: Transform::from_xyz(0.0, 0.0, 0.2),
            ..default()
        },
        Fill::color(COLOR_PARCHMENT),
        Cartouche,
        RenderLayers::layer(UI_LAYER),
    )).set_parent(root);

    // Decorative corner flourishes
    spawn_corner_flourish(&mut commands, root, Vec2::new(-FRAME_WIDTH / 2.0, FRAME_HEIGHT / 2.0), Corner::TopLeft);
    spawn_corner_flourish(&mut commands, root, Vec2::new(FRAME_WIDTH / 2.0, FRAME_HEIGHT / 2.0), Corner::TopRight);
    spawn_corner_flourish(&mut commands, root, Vec2::new(FRAME_WIDTH / 2.0, -FRAME_HEIGHT / 2.0), Corner::BottomRight);
    spawn_corner_flourish(&mut commands, root, Vec2::new(-FRAME_WIDTH / 2.0, -FRAME_HEIGHT / 2.0), Corner::BottomLeft);

    // Title text
    let font = asset_server.load("fonts/Quintessential-Regular.ttf");
    commands.spawn((
        Text2d::new("THE CARIBBEAN"),
        TextFont {
            font: font.clone(),
            font_size: 28.0,
            ..default()
        },
        TextColor(COLOR_INK),
        Transform::from_xyz(0.0, 8.0, 0.5),
        Cartouche,
        RenderLayers::layer(UI_LAYER),
    )).set_parent(root);

    // Subtitle
    commands.spawn((
        Text2d::new("A Nautical Chart"),
        TextFont {
            font,
            font_size: 14.0,
            ..default()
        },
        TextColor(COLOR_INK),
        Transform::from_xyz(0.0, -15.0, 0.5),
        Cartouche,
        RenderLayers::layer(UI_LAYER),
    )).set_parent(root);

    info!("Spawned Map Title Cartouche");
}

/// Draws the outer baroque-style frame with ornate border.
fn spawn_baroque_frame(commands: &mut Commands, parent: Entity) {
    // Outer frame shape
    let outer = shapes::RoundedPolygon {
        points: vec![
            Vec2::new(-FRAME_WIDTH / 2.0, -FRAME_HEIGHT / 2.0),
            Vec2::new(FRAME_WIDTH / 2.0, -FRAME_HEIGHT / 2.0),
            Vec2::new(FRAME_WIDTH / 2.0, FRAME_HEIGHT / 2.0),
            Vec2::new(-FRAME_WIDTH / 2.0, FRAME_HEIGHT / 2.0),
        ],
        radius: CORNER_RADIUS,
        closed: true,
    };
    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&outer),
            transform: Transform::from_xyz(0.0, 0.0, 0.1),
            ..default()
        },
        Fill::color(COLOR_GOLD),
        Stroke::new(COLOR_INK, 2.5),
        Cartouche,
        RenderLayers::layer(UI_LAYER),
    )).set_parent(parent);

    // Decorative inner border line
    let inner_border = shapes::RoundedPolygon {
        points: vec![
            Vec2::new(-FRAME_WIDTH / 2.0 + 5.0, -FRAME_HEIGHT / 2.0 + 5.0),
            Vec2::new(FRAME_WIDTH / 2.0 - 5.0, -FRAME_HEIGHT / 2.0 + 5.0),
            Vec2::new(FRAME_WIDTH / 2.0 - 5.0, FRAME_HEIGHT / 2.0 - 5.0),
            Vec2::new(-FRAME_WIDTH / 2.0 + 5.0, FRAME_HEIGHT / 2.0 - 5.0),
        ],
        radius: CORNER_RADIUS - 3.0,
        closed: true,
    };
    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&inner_border),
            transform: Transform::from_xyz(0.0, 0.0, 0.15),
            ..default()
        },
        Stroke::new(COLOR_INK, 1.0),
        Cartouche,
        RenderLayers::layer(UI_LAYER),
    )).set_parent(parent);
}

/// Spawns a decorative curl flourish at a corner.
/// The flourish curls outward from the frame corner.
fn spawn_corner_flourish(commands: &mut Commands, parent: Entity, position: Vec2, corner: Corner) {
    let mut path = PathBuilder::new();
    
    // Direction vectors based on corner (pointing outward from frame)
    let (dir_x, dir_y) = match corner {
        Corner::TopLeft => (-1.0, 1.0),
        Corner::TopRight => (1.0, 1.0),
        Corner::BottomRight => (1.0, -1.0),
        Corner::BottomLeft => (-1.0, -1.0),
    };
    
    // Draw curl extending outward from corner
    path.move_to(Vec2::ZERO);
    path.cubic_bezier_to(
        Vec2::new(dir_x * 10.0, dir_y * 2.0),
        Vec2::new(dir_x * 16.0, dir_y * 8.0),
        Vec2::new(dir_x * 12.0, dir_y * 16.0),
    );
    // Curl back inward
    path.cubic_bezier_to(
        Vec2::new(dir_x * 8.0, dir_y * 12.0),
        Vec2::new(dir_x * 6.0, dir_y * 6.0),
        Vec2::new(dir_x * 2.0, dir_y * 4.0),
    );
    
    commands.spawn((
        ShapeBundle {
            path: path.build(),
            transform: Transform::from_translation(position.extend(0.3)),
            ..default()
        },
        Stroke::new(COLOR_INK, 1.5),
        Cartouche,
        RenderLayers::layer(UI_LAYER),
    )).set_parent(parent);
}

/// Corner identifier for flourish orientation.
enum Corner {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

/// Keeps the cartouche at the top-center when window is resized.
fn update_cartouche_position(
    window_query: Query<&Window, (With<PrimaryWindow>, Changed<Window>)>,
    mut root_query: Query<&mut Transform, With<CartoucheRoot>>,
) {
    let Ok(window) = window_query.get_single() else { return; };
    let Ok(mut transform) = root_query.get_single_mut() else { return; };

    let half_h = window.height() / 2.0;
    transform.translation.x = 0.0;
    transform.translation.y = half_h - MARGIN_TOP;
}

fn despawn_cartouche(mut commands: Commands, query: Query<Entity, With<Cartouche>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

/// Applies the root's FadeController alpha to all child cartouche entities.
/// Updates TextColor, Stroke, and Fill components.
fn apply_cartouche_fade(
    root_query: Query<&FadeController, (With<CartoucheRoot>, Changed<FadeController>)>,
    mut text_query: Query<&mut TextColor, With<Cartouche>>,
    mut stroke_query: Query<&mut Stroke, With<Cartouche>>,
    mut fill_query: Query<&mut Fill, With<Cartouche>>,
) {
    let Ok(fade) = root_query.get_single() else { return; };
    let alpha = fade.current_alpha;

    // Update text colors
    for mut color in &mut text_query {
        color.0 = color.0.with_alpha(alpha);
    }

    // Update strokes
    for mut stroke in &mut stroke_query {
        stroke.color = stroke.color.with_alpha(alpha);
    }

    // Update fills
    for mut fill in &mut fill_query {
        fill.color = fill.color.with_alpha(alpha);
    }
}

