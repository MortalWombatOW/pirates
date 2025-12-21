use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::components::{
    health::Health,
    port::{Inventory, Port, PortName},
    ship::{Player, Ship},
};
use crate::plugins::core::GameState;

/// Plugin for the Port View UI.
/// Displays when the player is docked at a port.
pub struct PortUiPlugin;

impl Plugin for PortUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CurrentPort>()
            .init_resource::<PortUiState>()
            .add_systems(Update, port_ui_system.run_if(in_state(GameState::Port)));
    }
}

/// Resource tracking which port the player is currently visiting.
#[derive(Resource, Default)]
pub struct CurrentPort {
    pub entity: Option<Entity>,
}

/// UI state for the port interface.
#[derive(Resource, Default)]
pub struct PortUiState {
    /// Currently selected tab (0=Market, 1=Tavern, 2=Docks, 3=Contracts)
    pub selected_tab: usize,
}

/// The main port UI system.
fn port_ui_system(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    mut ui_state: ResMut<PortUiState>,
    current_port: Res<CurrentPort>,
    port_query: Query<(&PortName, &Inventory), With<Port>>,
    player_query: Query<&Health, (With<Player>, With<Ship>)>,
) {
    // Get port data if available
    let port_data = current_port.entity.and_then(|e| port_query.get(e).ok());
    let port_name = port_data
        .map(|(name, _)| name.0.as_str())
        .unwrap_or("Unknown Port");
    
    // Central panel for the port UI
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        // Header with port name and depart button
        ui.horizontal(|ui| {
            ui.heading(port_name);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("⛵ Depart").clicked() {
                    info!("Departing from port...");
                    next_state.set(GameState::HighSeas);
                }
            });
        });
        
        ui.separator();
        
        // Tab bar
        ui.horizontal(|ui| {
            let tabs = ["🏪 Market", "🍺 Tavern", "⚓ Docks", "📜 Contracts"];
            for (idx, tab_name) in tabs.iter().enumerate() {
                if ui.selectable_label(ui_state.selected_tab == idx, *tab_name).clicked() {
                    ui_state.selected_tab = idx;
                }
            }
        });
        
        ui.separator();
        
        // Tab content
        egui::ScrollArea::vertical().show(ui, |ui| {
            match ui_state.selected_tab {
                0 => render_market_panel(ui, port_data.map(|(_, inv)| inv)),
                1 => render_tavern_panel(ui),
                2 => render_docks_panel(ui, player_query.get_single().ok()),
                3 => render_contracts_panel(ui),
                _ => {}
            }
        });
    });
}

/// Renders the Market panel with goods for buying/selling.
fn render_market_panel(ui: &mut egui::Ui, inventory: Option<&Inventory>) {
    ui.heading("Market");
    ui.label("Buy and sell goods at this port.");
    ui.add_space(10.0);
    
    if let Some(inventory) = inventory {
        if inventory.goods.is_empty() {
            ui.label("No goods available at this market.");
        } else {
            // Table header
            egui::Grid::new("market_grid")
                .num_columns(4)
                .striped(true)
                .min_col_width(80.0)
                .show(ui, |ui| {
                    ui.strong("Good");
                    ui.strong("Stock");
                    ui.strong("Price");
                    ui.strong("Actions");
                    ui.end_row();
                    
                    // Sort goods for consistent display
                    let mut goods: Vec<_> = inventory.goods.iter().collect();
                    goods.sort_by(|a, b| format!("{:?}", a.0).cmp(&format!("{:?}", b.0)));
                    
                    for (good_type, item) in goods {
                        ui.label(format!("{:?}", good_type));
                        ui.label(format!("{}", item.quantity));
                        ui.label(format!("{:.0}g", item.price));
                        ui.horizontal(|ui| {
                            if ui.small_button("Buy").clicked() {
                                info!("Buy {:?} clicked", good_type);
                                // TODO: Implement in Epic 4.3
                            }
                            if ui.small_button("Sell").clicked() {
                                info!("Sell {:?} clicked", good_type);
                                // TODO: Implement in Epic 4.3
                            }
                        });
                        ui.end_row();
                    }
                });
        }
    } else {
        // No current port - show sample inventory for testing
        ui.label("⚠ Debug Mode: No port selected");
        ui.add_space(10.0);
        
        egui::Grid::new("market_grid_sample")
            .num_columns(4)
            .striped(true)
            .min_col_width(80.0)
            .show(ui, |ui| {
                ui.strong("Good");
                ui.strong("Stock");
                ui.strong("Price");
                ui.strong("Actions");
                ui.end_row();
                
                // Sample goods for visual testing
                let sample_goods = [
                    ("Rum", 75, 15),
                    ("Sugar", 120, 8),
                    ("Spices", 30, 25),
                ];
                
                for (name, qty, price) in sample_goods {
                    ui.label(name);
                    ui.label(format!("{}", qty));
                    ui.label(format!("{}g", price));
                    ui.horizontal(|ui| {
                        let _ = ui.small_button("Buy");
                        let _ = ui.small_button("Sell");
                    });
                    ui.end_row();
                }
            });
    }
}

/// Renders the Tavern panel (placeholder for Epic 6.1).
fn render_tavern_panel(ui: &mut egui::Ui) {
    ui.heading("Tavern");
    ui.label("Gather intelligence and recruit crew.");
    ui.add_space(20.0);
    
    ui.vertical_centered(|ui| {
        ui.label("🍺");
        ui.add_space(10.0);
        ui.label("The tavern is quiet today...");
        ui.add_space(5.0);
        ui.weak("(Intel system coming in Epic 6.1)");
    });
}

/// Renders the Docks panel with ship repair options.
fn render_docks_panel(ui: &mut egui::Ui, health: Option<&Health>) {
    ui.heading("Docks");
    ui.label("Repair and upgrade your ship.");
    ui.add_space(10.0);
    
    if let Some(health) = health {
        ui.group(|ui| {
            ui.label("Ship Status:");
            ui.add_space(5.0);
            
            // Sails
            let sails_pct = health.sails / health.sails_max;
            ui.horizontal(|ui| {
                ui.label("Sails:");
                ui.add(egui::ProgressBar::new(sails_pct)
                    .text(format!("{:.0}/{:.0}", health.sails, health.sails_max))
                    .fill(if sails_pct > 0.5 { egui::Color32::from_rgb(100, 180, 100) } else { egui::Color32::from_rgb(200, 150, 50) })
                );
                if sails_pct < 1.0 && ui.small_button("Repair").clicked() {
                    info!("Repair sails clicked");
                    // TODO: Implement in Epic 4.6
                }
            });
            
            // Rudder
            let rudder_pct = health.rudder / health.rudder_max;
            ui.horizontal(|ui| {
                ui.label("Rudder:");
                ui.add(egui::ProgressBar::new(rudder_pct)
                    .text(format!("{:.0}/{:.0}", health.rudder, health.rudder_max))
                    .fill(if rudder_pct > 0.5 { egui::Color32::from_rgb(100, 180, 100) } else { egui::Color32::from_rgb(200, 150, 50) })
                );
                if rudder_pct < 1.0 && ui.small_button("Repair").clicked() {
                    info!("Repair rudder clicked");
                    // TODO: Implement in Epic 4.6
                }
            });
            
            // Hull
            let hull_pct = health.hull / health.hull_max;
            ui.horizontal(|ui| {
                ui.label("Hull:");
                ui.add(egui::ProgressBar::new(hull_pct)
                    .text(format!("{:.0}/{:.0}", health.hull, health.hull_max))
                    .fill(if hull_pct > 0.5 { egui::Color32::from_rgb(100, 180, 100) } else { egui::Color32::from_rgb(180, 80, 80) })
                );
                if hull_pct < 1.0 && ui.small_button("Repair").clicked() {
                    info!("Repair hull clicked");
                    // TODO: Implement in Epic 4.6
                }
            });
        });
    } else {
        ui.label("⚠ No ship data available");
        ui.weak("(Player ship not found)");
    }
}

/// Renders the Contracts panel (placeholder for Epic 4.5).
fn render_contracts_panel(ui: &mut egui::Ui) {
    ui.heading("Contracts");
    ui.label("Accept jobs for gold and reputation.");
    ui.add_space(20.0);
    
    ui.vertical_centered(|ui| {
        ui.label("📜");
        ui.add_space(10.0);
        ui.label("No contracts available at this port.");
        ui.add_space(5.0);
        ui.weak("(Contract system coming in Epic 4.5)");
    });
}
