use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::plugins::core::GameState;

pub struct UiThemePlugin;

impl Plugin for UiThemePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::resources::ui_assets::UiAssets>()
           .add_systems(Update, configure_ui_theme.run_if(in_state(GameState::MainMenu).or(in_state(GameState::Port))));
    }
}

fn configure_ui_theme(mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut();
    
    // Define Ink & Parchment colors
    let ink_color = egui::Color32::from_rgb(60, 42, 26); // Dark Brown
    let _parchment_color = egui::Color32::from_rgb(235, 213, 179); // Light Beige (Reference)
    let highlight_color = egui::Color32::from_rgb(100, 60, 30); // Lighter Brown for interaction
    
    let mut style = (*ctx.style()).clone();
    
    // Text styles
    style.visuals.override_text_color = Some(ink_color);
    style.visuals.hyperlink_color = highlight_color;
    
    // Window/Panel styles
    // We want transparent panels so we can draw the texture behind them manually
    style.visuals.window_fill = egui::Color32::TRANSPARENT;
    style.visuals.panel_fill = egui::Color32::TRANSPARENT;
    style.visuals.window_stroke = egui::Stroke::new(1.0, ink_color);
    
    // Widgets
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgba_premultiplied(60, 42, 26, 20); // Faint background
    style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, ink_color);
    
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgba_premultiplied(60, 42, 26, 40);
    style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, ink_color);
    
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgba_premultiplied(100, 60, 30, 60);
    style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, ink_color);
    
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgba_premultiplied(100, 60, 30, 100);
    style.visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, ink_color);
    
    ctx.set_style(style);
}

/// Helper to draw parchment background
pub fn draw_parchment_bg(ui: &mut egui::Ui, texture_id: egui::TextureId) {
    let rect = ui.max_rect();
    let tile_size = 512.0;

    let mut y = rect.min.y;
    while y < rect.max.y {
        let mut x = rect.min.x;
        while x < rect.max.x {
            let tile_rect = egui::Rect::from_min_size(
                egui::pos2(x, y),
                egui::vec2(tile_size, tile_size)
            );
            
            // Clip the tile rect to the window rect
            let visible_rect = tile_rect.intersect(rect);
            
            // Calculate UVs for the clipped portion
            let uv_min = egui::pos2(
                (visible_rect.min.x - x) / tile_size,
                (visible_rect.min.y - y) / tile_size
            );
            let uv_max = egui::pos2(
                (visible_rect.max.x - x) / tile_size,
                (visible_rect.max.y - y) / tile_size
            );
            let uv = egui::Rect::from_min_max(uv_min, uv_max);

            ui.painter().image(
                texture_id,
                visible_rect,
                uv,
                egui::Color32::WHITE
            );

            x += tile_size;
        }
        y += tile_size;
    }
}

/// Ink color constant for decorative elements.
pub const INK_COLOR: egui::Color32 = egui::Color32::from_rgb(60, 42, 26);

/// Draws decorative corner flourishes at all four corners of a rectangle.
/// Creates scroll-like curved patterns typical of old nautical documents.
pub fn draw_corner_flourishes(ui: &mut egui::Ui, rect: egui::Rect, size: f32) {
    let painter = ui.painter();
    
    // Inset corners slightly from the edge
    let inset = 8.0;
    let corners = [
        (rect.left_top() + egui::vec2(inset, inset), false, false),      // Top-left
        (rect.right_top() + egui::vec2(-inset, inset), true, false),     // Top-right
        (rect.left_bottom() + egui::vec2(inset, -inset), false, true),   // Bottom-left
        (rect.right_bottom() + egui::vec2(-inset, -inset), true, true),  // Bottom-right
    ];
    
    for (corner_pos, flip_x, flip_y) in corners {
        draw_single_flourish(painter, corner_pos, size, flip_x, flip_y);
    }
}

/// Draws a single corner flourish with bezier curves.
fn draw_single_flourish(
    painter: &egui::Painter,
    origin: egui::Pos2,
    size: f32,
    flip_x: bool,
    flip_y: bool,
) {
    let stroke = egui::Stroke::new(1.5, INK_COLOR);
    let x_mult = if flip_x { -1.0 } else { 1.0 };
    let y_mult = if flip_y { -1.0 } else { 1.0 };
    
    // Main L-shaped line
    let h_end = origin + egui::vec2(size * x_mult, 0.0);
    let v_end = origin + egui::vec2(0.0, size * y_mult);
    painter.line_segment([origin, h_end], stroke);
    painter.line_segment([origin, v_end], stroke);
    
    // Curled end on horizontal line
    let curl_size = size * 0.3;
    let h_curl_start = h_end;
    let h_curl_ctrl = h_end + egui::vec2(curl_size * 0.3 * x_mult, -curl_size * 0.5 * y_mult);
    let h_curl_end = h_end + egui::vec2(-curl_size * 0.2 * x_mult, -curl_size * 0.7 * y_mult);
    draw_bezier_curve(painter, h_curl_start, h_curl_ctrl, h_curl_end, stroke);
    
    // Curled end on vertical line
    let v_curl_start = v_end;
    let v_curl_ctrl = v_end + egui::vec2(-curl_size * 0.5 * x_mult, curl_size * 0.3 * y_mult);
    let v_curl_end = v_end + egui::vec2(-curl_size * 0.7 * x_mult, -curl_size * 0.2 * y_mult);
    draw_bezier_curve(painter, v_curl_start, v_curl_ctrl, v_curl_end, stroke);
    
    // Small decorative dot at corner
    painter.circle_filled(origin, 2.5, INK_COLOR);
}

/// Draws a quadratic bezier curve using line segments.
fn draw_bezier_curve(
    painter: &egui::Painter,
    p0: egui::Pos2,
    p1: egui::Pos2,
    p2: egui::Pos2,
    stroke: egui::Stroke,
) {
    const SEGMENTS: usize = 8;
    let mut prev = p0;
    
    for i in 1..=SEGMENTS {
        let t = i as f32 / SEGMENTS as f32;
        let inv_t = 1.0 - t;
        
        // Quadratic bezier: B(t) = (1-t)²P0 + 2(1-t)tP1 + t²P2
        let x = inv_t * inv_t * p0.x + 2.0 * inv_t * t * p1.x + t * t * p2.x;
        let y = inv_t * inv_t * p0.y + 2.0 * inv_t * t * p1.y + t * t * p2.y;
        let curr = egui::pos2(x, y);
        
        painter.line_segment([prev, curr], stroke);
        prev = curr;
    }
}

/// Draws an ornamental horizontal divider with anchor centerpiece.
/// Use this to separate sections within UI panels.
pub fn draw_ornamental_divider(ui: &mut egui::Ui, width: f32) {
    let (response, painter) = ui.allocate_painter(egui::vec2(width, 20.0), egui::Sense::hover());
    let rect = response.rect;
    let center = rect.center();
    let stroke = egui::Stroke::new(1.0, INK_COLOR);
    let thick_stroke = egui::Stroke::new(1.5, INK_COLOR);
    
    // Horizontal lines on either side
    let line_y = center.y;
    let gap = 12.0; // Gap for center decoration
    
    // Left line with decorative ends
    let left_start = egui::pos2(rect.left() + 10.0, line_y);
    let left_end = egui::pos2(center.x - gap, line_y);
    painter.line_segment([left_start, left_end], stroke);
    
    // Right line with decorative ends
    let right_start = egui::pos2(center.x + gap, line_y);
    let right_end = egui::pos2(rect.right() - 10.0, line_y);
    painter.line_segment([right_start, right_end], stroke);
    
    // Decorative diamond ends
    draw_small_diamond(&painter, left_start, 3.0);
    draw_small_diamond(&painter, right_end, 3.0);
    
    // Center anchor symbol
    draw_anchor_symbol(&painter, center, 8.0, thick_stroke);
}

/// Draws a small decorative diamond shape.
fn draw_small_diamond(painter: &egui::Painter, center: egui::Pos2, size: f32) {
    let points = [
        egui::pos2(center.x, center.y - size),
        egui::pos2(center.x + size, center.y),
        egui::pos2(center.x, center.y + size),
        egui::pos2(center.x - size, center.y),
    ];
    painter.add(egui::Shape::convex_polygon(
        points.to_vec(),
        INK_COLOR,
        egui::Stroke::NONE,
    ));
}

/// Draws a simplified anchor symbol.
fn draw_anchor_symbol(painter: &egui::Painter, center: egui::Pos2, size: f32, stroke: egui::Stroke) {
    // Vertical shaft
    let top = center + egui::vec2(0.0, -size);
    let bottom = center + egui::vec2(0.0, size);
    painter.line_segment([top, bottom], stroke);
    
    // Top ring (small circle)
    painter.circle_stroke(top + egui::vec2(0.0, -2.0), 2.5, stroke);
    
    // Crossbar
    let cross_y = center.y - size * 0.3;
    let cross_left = egui::pos2(center.x - size * 0.5, cross_y);
    let cross_right = egui::pos2(center.x + size * 0.5, cross_y);
    painter.line_segment([cross_left, cross_right], stroke);
    
    // Flukes (curved arms at bottom)
    let fluke_y = center.y + size * 0.7;
    let fluke_left = egui::pos2(center.x - size * 0.7, fluke_y);
    let fluke_right = egui::pos2(center.x + size * 0.7, fluke_y);
    painter.line_segment([bottom, fluke_left], stroke);
    painter.line_segment([bottom, fluke_right], stroke);
    
    // Small arrowheads on flukes
    let arrow_size = 2.5;
    painter.line_segment([fluke_left, fluke_left + egui::vec2(arrow_size, -arrow_size)], stroke);
    painter.line_segment([fluke_left, fluke_left + egui::vec2(arrow_size, arrow_size)], stroke);
    painter.line_segment([fluke_right, fluke_right + egui::vec2(-arrow_size, -arrow_size)], stroke);
    painter.line_segment([fluke_right, fluke_right + egui::vec2(-arrow_size, arrow_size)], stroke);
}

/// Draws a decorative border around the given rectangle with corner flourishes.
/// Combines a subtle ink border with flourishes at each corner.
pub fn draw_panel_border(ui: &mut egui::Ui, rect: egui::Rect, flourish_size: f32) {
    let painter = ui.painter();
    let border_stroke = egui::Stroke::new(1.0, INK_COLOR);
    
    // Draw main border rectangle
    painter.rect_stroke(rect, 0.0, border_stroke);
    
    // Add corner flourishes
    draw_corner_flourishes(ui, rect, flourish_size);
}

/// Draws a simple rope-style horizontal divider.
/// A lighter alternative to the anchor divider for less prominent separations.
pub fn draw_rope_divider(ui: &mut egui::Ui, width: f32) {
    let (response, painter) = ui.allocate_painter(egui::vec2(width, 12.0), egui::Sense::hover());
    let rect = response.rect;
    let center_y = rect.center().y;
    let stroke = egui::Stroke::new(1.0, INK_COLOR);
    
    // Draw wavy rope pattern
    let wave_amplitude = 2.0;
    let wave_frequency = 0.15;
    let start_x = rect.left() + 20.0;
    let end_x = rect.right() - 20.0;
    
    let mut prev_pos = egui::pos2(start_x, center_y);
    let step = 4.0;
    let mut x = start_x + step;
    
    while x <= end_x {
        let wave_offset = (x * wave_frequency).sin() * wave_amplitude;
        let curr_pos = egui::pos2(x, center_y + wave_offset);
        painter.line_segment([prev_pos, curr_pos], stroke);
        prev_pos = curr_pos;
        x += step;
    }
    
    // End knots
    painter.circle_filled(egui::pos2(start_x - 3.0, center_y), 2.5, INK_COLOR);
    painter.circle_filled(egui::pos2(end_x + 3.0, center_y), 2.5, INK_COLOR);
}
