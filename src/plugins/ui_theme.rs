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
