use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::components::{
    cargo::{Cargo, Gold},
    health::Health,
    port::{Inventory, Port, PortName},
    ship::{Player, Ship},
};
use crate::events::TradeExecutedEvent;
use crate::plugins::core::GameState;

/// Plugin for the Port View UI.
/// Displays when the player is docked at a port.
pub struct PortUiPlugin;

impl Plugin for PortUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CurrentPort>()
            .init_resource::<PortUiState>()
            .add_event::<TradeExecutedEvent>()
            .add_systems(Update, (
                port_ui_system,
                trade_execution_system,
            ).run_if(in_state(GameState::Port)));
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
    mut current_port: ResMut<CurrentPort>,
    mut trade_events: EventWriter<TradeExecutedEvent>,
    port_query: Query<(Entity, &PortName, &Inventory), With<Port>>,
    player_query: Query<(&Health, Option<&Cargo>, Option<&Gold>), (With<Player>, With<Ship>)>,
) {
    // Auto-select first port if none selected (for debug/testing)
    if current_port.entity.is_none() {
        if let Some((entity, _, _)) = port_query.iter().next() {
            current_port.entity = Some(entity);
        }
    }
    
    // Get port data if available
    let port_data = current_port.entity.and_then(|e| port_query.get(e).ok());
    let port_name = port_data
        .map(|(_, name, _)| name.0.as_str())
        .unwrap_or("Unknown Port");
    let port_entity = port_data.map(|(e, _, _)| e);
    
    // Get player data
    let player_data = player_query.get_single().ok();
    let player_gold = player_data.and_then(|(_, _, g)| g.map(|g| g.0)).unwrap_or(0);
    let player_cargo = player_data.and_then(|(_, c, _)| c);
    
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
                ui.label(format!("💰 {} gold", player_gold));
                if let Some(cargo) = player_cargo {
                    ui.label(format!("📦 {}/{}", cargo.total_units(), cargo.capacity));
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
                0 => render_market_panel(
                    ui, 
                    port_entity,
                    port_data.map(|(_, _, inv)| inv),
                    player_gold,
                    player_cargo,
                    &mut trade_events,
                ),
                1 => render_tavern_panel(ui),
                2 => render_docks_panel(ui, player_data.map(|(h, _, _)| h)),
                3 => render_contracts_panel(ui),
                _ => {}
            }
        });
    });
}

/// Renders the Market panel with goods for buying/selling.
fn render_market_panel(
    ui: &mut egui::Ui, 
    port_entity: Option<Entity>,
    inventory: Option<&Inventory>,
    player_gold: u32,
    player_cargo: Option<&Cargo>,
    trade_events: &mut EventWriter<TradeExecutedEvent>,
) {
    ui.heading("Market");
    ui.label("Buy and sell goods at this port.");
    ui.add_space(10.0);
    
    if let (Some(port_entity), Some(inventory)) = (port_entity, inventory) {
        if inventory.goods.is_empty() {
            ui.label("No goods available at this market.");
        } else {
            // Table header
            egui::Grid::new("market_grid")
                .num_columns(5)
                .striped(true)
                .min_col_width(60.0)
                .show(ui, |ui| {
                    ui.strong("Good");
                    ui.strong("Stock");
                    ui.strong("Price");
                    ui.strong("You Have");
                    ui.strong("Actions");
                    ui.end_row();
                    
                    // Sort goods for consistent display
                    let mut goods: Vec<_> = inventory.goods.iter().collect();
                    goods.sort_by(|a, b| format!("{:?}", a.0).cmp(&format!("{:?}", b.0)));
                    
                    for (good_type, item) in goods {
                        ui.label(format!("{:?}", good_type));
                        ui.label(format!("{}", item.quantity));
                        ui.label(format!("{:.0}g", item.price));
                        
                        // Show player's quantity of this good
                        let player_qty = player_cargo
                            .map(|c| c.get(*good_type))
                            .unwrap_or(0);
                        ui.label(format!("{}", player_qty));
                        
                        // Buy/Sell buttons
                        ui.horizontal(|ui| {
                            let price = item.price as u32;
                            let can_buy = item.quantity > 0 
                                && player_gold >= price
                                && player_cargo.map(|c| !c.is_full()).unwrap_or(false);
                            
                            if ui.add_enabled(can_buy, egui::Button::new("Buy")).clicked() {
                                trade_events.send(TradeExecutedEvent {
                                    port_entity,
                                    good_type: *good_type,
                                    quantity: 1,
                                    is_buy: true,
                                });
                            }
                            
                            let can_sell = player_qty > 0;
                            if ui.add_enabled(can_sell, egui::Button::new("Sell")).clicked() {
                                trade_events.send(TradeExecutedEvent {
                                    port_entity,
                                    good_type: *good_type,
                                    quantity: 1,
                                    is_buy: false,
                                });
                            }
                        });
                        ui.end_row();
                    }
                });
        }
    } else {
        ui.label("⚠ No port data available");
        ui.weak("(Enter port from High Seas to trade)");
    }
}

/// System that executes trades based on TradeExecutedEvent.
fn trade_execution_system(
    mut trade_events: EventReader<TradeExecutedEvent>,
    mut port_query: Query<&mut Inventory, With<Port>>,
    mut player_query: Query<(&mut Cargo, &mut Gold), (With<Player>, With<Ship>)>,
) {
    for event in trade_events.read() {
        let Ok(mut inventory) = port_query.get_mut(event.port_entity) else {
            warn!("Trade failed: Port entity {:?} not found", event.port_entity);
            continue;
        };
        
        let Ok((mut cargo, mut gold)) = player_query.get_single_mut() else {
            warn!("Trade failed: Player not found");
            continue;
        };
        
        if event.is_buy {
            // Buying: Player pays gold, receives goods
            let Some(item) = inventory.get_good(&event.good_type) else {
                warn!("Trade failed: Good {:?} not in port inventory", event.good_type);
                continue;
            };
            
            let price = item.price as u32;
            let available = item.quantity;
            let qty = event.quantity.min(available);
            
            if qty == 0 {
                warn!("Trade failed: No stock available");
                continue;
            }
            
            let total_cost = price * qty;
            if !gold.spend(total_cost) {
                info!("Trade failed: Insufficient gold ({} < {})", gold.0, total_cost);
                continue;
            }
            
            // Check cargo capacity
            let added = cargo.add(event.good_type, qty);
            if added < qty {
                // Refund for goods that didn't fit
                let refund = (qty - added) * price;
                gold.add(refund);
            }
            
            // Remove from port inventory
            let _ = inventory.buy(&event.good_type, added);
            
            info!("Bought {} {:?} for {} gold", added, event.good_type, added * price);
        } else {
            // Selling: Player gives goods, receives gold
            let removed = cargo.remove(event.good_type, event.quantity);
            if removed == 0 {
                warn!("Trade failed: No {:?} in cargo", event.good_type);
                continue;
            }
            
            // Sell at 80% of port's buy price
            let sell_modifier = 0.8;
            let revenue = inventory.sell(event.good_type, removed, sell_modifier) as u32;
            gold.add(revenue);
            
            info!("Sold {} {:?} for {} gold", removed, event.good_type, revenue);
        }
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
