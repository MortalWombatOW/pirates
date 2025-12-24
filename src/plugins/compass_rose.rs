//! Compass Rose UI component - a traditional 32-point wind rose.
//!
//! Uses a procedurally generated texture displayed as true screen-space UI
//! to avoid jitter during camera movement.

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;

use crate::plugins::core::GameState;

pub struct CompassRosePlugin;

impl Plugin for CompassRosePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::HighSeas), spawn_compass_rose)
            .add_systems(OnExit(GameState::HighSeas), despawn_compass_rose);
    }
}

// --- Color Constants ---
// Traditional compass rose colors (can be refactored to a theme resource later)

/// Gold/Yellow for principal winds (N, NE, E, SE, S, SW, W, NW)
const COLOR_PRINCIPAL: [u8; 4] = [201, 162, 39, 255]; // #C9A227
/// Black outline for principal wind spikes
const COLOR_PRINCIPAL_DARK: [u8; 4] = [38, 31, 20, 255];
/// Green for half-winds (NNE, ENE, ESE, etc.)
const COLOR_HALF_WIND: [u8; 4] = [45, 90, 61, 255]; // #2D5A3D
/// Red for quarter-winds (finest divisions)
const COLOR_QUARTER_WIND: [u8; 4] = [176, 48, 48, 255]; // #B03030
/// Parchment/cream for background elements
const COLOR_PARCHMENT: [u8; 4] = [240, 230, 200, 255];
/// Dark ink for outlines
const COLOR_INK: [u8; 4] = [38, 31, 20, 255];
/// Transparent
const COLOR_TRANSPARENT: [u8; 4] = [0, 0, 0, 0];

// --- Geometry Constants ---
const TEXTURE_SIZE: u32 = 180;
const CENTER: f32 = TEXTURE_SIZE as f32 / 2.0;
const OUTER_RING_RADIUS: f32 = 70.0;
const CENTER_RADIUS: f32 = 12.0;

// Spike lengths
const PRINCIPAL_LENGTH: f32 = 55.0;
const HALF_WIND_LENGTH: f32 = 40.0;
const QUARTER_WIND_LENGTH: f32 = 28.0;

// Spike widths at base
const PRINCIPAL_WIDTH: f32 = 14.0;
const HALF_WIND_WIDTH: f32 = 9.0;
const QUARTER_WIND_WIDTH: f32 = 5.0;

/// Marker component for compass rose UI entities
#[derive(Component)]
pub struct CompassRose;

/// Spawns the compass rose as true screen-space UI.
fn spawn_compass_rose(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let texture = create_compass_texture();
    let texture_handle = images.add(texture);

    commands.spawn((
        Name::new("Compass Rose"),
        CompassRose,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            right: Val::Px(20.0),
            width: Val::Px(TEXTURE_SIZE as f32),
            height: Val::Px(TEXTURE_SIZE as f32),
            ..default()
        },
    )).with_children(|parent| {
        parent.spawn((
            ImageNode::new(texture_handle),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ));
    });

    info!("Spawned Compass Rose UI");
}

/// Creates a procedural compass rose texture.
fn create_compass_texture() -> Image {
    let size = TEXTURE_SIZE;
    let mut data = vec![0u8; (size * size * 4) as usize];

    // Helper to set a pixel
    let set_pixel = |data: &mut [u8], x: i32, y: i32, color: [u8; 4]| {
        if x >= 0 && x < size as i32 && y >= 0 && y < size as i32 {
            let idx = ((y as u32 * size + x as u32) * 4) as usize;
            // Alpha blending for antialiasing
            if color[3] > 0 {
                data[idx] = color[0];
                data[idx + 1] = color[1];
                data[idx + 2] = color[2];
                data[idx + 3] = color[3];
            }
        }
    };

    // Draw filled triangle (spike)
    let draw_spike = |data: &mut [u8], angle: f32, length: f32, width: f32, color: [u8; 4]| {
        // Tip of spike (angle: 0 = up/North)
        let tip_x = CENTER + angle.sin() * length;
        let tip_y = CENTER - angle.cos() * length; // Y flipped for texture coords
        
        // Base corners
        let base_dist = CENTER_RADIUS * 0.8;
        let base_x = CENTER + angle.sin() * base_dist;
        let base_y = CENTER - angle.cos() * base_dist;
        
        let perp_x = angle.cos();
        let perp_y = angle.sin();
        
        let left_x = base_x - perp_x * (width / 2.0);
        let left_y = base_y - perp_y * (width / 2.0);
        let right_x = base_x + perp_x * (width / 2.0);
        let right_y = base_y + perp_y * (width / 2.0);

        // Rasterize triangle using scanline
        let min_y = tip_y.min(left_y).min(right_y).floor() as i32;
        let max_y = tip_y.max(left_y).max(right_y).ceil() as i32;
        
        for y in min_y..=max_y {
            let mut min_x = size as i32;
            let mut max_x = 0i32;
            
            // Check intersection with each edge
            let edges = [
                (tip_x, tip_y, left_x, left_y),
                (left_x, left_y, right_x, right_y),
                (right_x, right_y, tip_x, tip_y),
            ];
            
            for (x1, y1, x2, y2) in edges {
                if (y1 <= y as f32 && y2 > y as f32) || (y2 <= y as f32 && y1 > y as f32) {
                    let t = (y as f32 - y1) / (y2 - y1);
                    let x = x1 + t * (x2 - x1);
                    min_x = min_x.min(x.floor() as i32);
                    max_x = max_x.max(x.ceil() as i32);
                }
            }
            
            for x in min_x..=max_x {
                set_pixel(data, x, y, color);
            }
        }
    };

    // Draw filled circle
    let draw_circle = |data: &mut [u8], cx: f32, cy: f32, radius: f32, fill: [u8; 4], stroke: [u8; 4]| {
        let r2 = radius * radius;
        let inner_r2 = (radius - 2.0) * (radius - 2.0);
        
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let d2 = dx * dx + dy * dy;
                
                if d2 <= r2 {
                    if d2 > inner_r2 {
                        set_pixel(data, x as i32, y as i32, stroke);
                    } else {
                        set_pixel(data, x as i32, y as i32, fill);
                    }
                }
            }
        }
    };

    // Draw ring (stroke only)
    let draw_ring = |data: &mut [u8], cx: f32, cy: f32, radius: f32, thickness: f32, color: [u8; 4]| {
        let outer_r2 = radius * radius;
        let inner_r2 = (radius - thickness) * (radius - thickness);
        
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let d2 = dx * dx + dy * dy;
                
                if d2 <= outer_r2 && d2 >= inner_r2 {
                    set_pixel(data, x as i32, y as i32, color);
                }
            }
        }
    };

    // --- Layer 1: Outer Ring ---
    draw_ring(&mut data, CENTER, CENTER, OUTER_RING_RADIUS, 2.0, COLOR_INK);

    // --- Layer 2: Quarter-Winds (16 red spikes) ---
    for i in 0..16 {
        let angle = (11.25_f32 + i as f32 * 22.5).to_radians();
        draw_spike(&mut data, angle, QUARTER_WIND_LENGTH, QUARTER_WIND_WIDTH, COLOR_QUARTER_WIND);
    }

    // --- Layer 3: Half-Winds (8 green spikes) ---
    for i in 0..8 {
        let angle = (22.5_f32 + i as f32 * 45.0).to_radians();
        draw_spike(&mut data, angle, HALF_WIND_LENGTH, HALF_WIND_WIDTH, COLOR_HALF_WIND);
    }

    // --- Layer 4: Principal Winds (8 gold/dark spikes) ---
    for i in 0..8 {
        let angle = (i as f32 * 45.0).to_radians();
        let color = if i % 2 == 0 { COLOR_PRINCIPAL } else { COLOR_PRINCIPAL_DARK };
        draw_spike(&mut data, angle, PRINCIPAL_LENGTH, PRINCIPAL_WIDTH, color);
    }

    // --- Layer 5: Center Circle ---
    draw_circle(&mut data, CENTER, CENTER, CENTER_RADIUS, COLOR_PARCHMENT, COLOR_INK);

    // --- Layer 6: Fleur-de-lis indicator at North (simple arrow for now) ---
    // Draw a small triangle pointing up above the compass
    let fleur_y = CENTER - OUTER_RING_RADIUS - 12.0;
    draw_spike(&mut data, 0.0_f32.to_radians(), OUTER_RING_RADIUS + 20.0, 8.0, COLOR_PRINCIPAL);

    Image::new(
        bevy::render::render_resource::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
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
