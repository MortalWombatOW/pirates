use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::resources::ui_assets::UiAssets;

use crate::components::{
    cargo::{Cargo, Gold},
    contract::{AcceptedContract, Contract, ContractDetails, ContractProgress},
    health::Health,
    intel::{Intel, IntelData, IntelType, IntelExpiry, TavernIntel, AcquiredIntel},
    port::{Inventory, Port, PortName},
    ship::{Player, Ship},
};
use crate::events::{ContractAcceptedEvent, ContractCompletedEvent, TradeExecutedEvent, RepairRequestEvent, RepairType, IntelAcquiredEvent};
use crate::plugins::core::GameState;
use crate::systems::repair::{repair_execution_system, calculate_repair_cost};

/// Plugin for the Port View UI.
/// Displays when the player is docked at a port.
pub struct PortUiPlugin;

impl Plugin for PortUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CurrentPort>()
            .init_resource::<PortUiState>()
            .init_resource::<PlayerContracts>()
            .add_event::<TradeExecutedEvent>()
            .add_event::<ContractAcceptedEvent>()
            .add_event::<ContractCompletedEvent>()
            .add_event::<RepairRequestEvent>()
            .add_event::<IntelAcquiredEvent>()
            .add_systems(OnEnter(GameState::Port), (generate_port_contracts, generate_tavern_intel))
            .add_systems(Update, (
                port_ui_system,
                trade_execution_system,
                contract_acceptance_system,
                repair_execution_system,
                intel_purchase_system,
                crate::systems::intel_acquisition_system,
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

/// Resource tracking player's active contracts.
#[derive(Resource, Default)]
pub struct PlayerContracts {
    pub active: Vec<Entity>,
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct PortUiEvents<'w> {
    pub trade: EventWriter<'w, TradeExecutedEvent>,
    pub contract: EventWriter<'w, ContractAcceptedEvent>,
    pub repair: EventWriter<'w, RepairRequestEvent>,
    pub intel: EventWriter<'w, IntelAcquiredEvent>,
    pub companion: EventWriter<'w, crate::plugins::companion::CompanionRecruitedEvent>,
    pub auto_trade: EventWriter<'w, crate::plugins::companion::AutoTradeEvent>,
}

/// Main system to render the Port UI.
fn port_ui_system(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    mut ui_state: ResMut<PortUiState>,
    current_port: Res<CurrentPort>,
    mut events: PortUiEvents,
    // Queries
    port_query: Query<(Entity, &PortName, &Inventory), With<Port>>,
    player_query: Query<(&Health, Option<&Cargo>, Option<&Gold>), (With<Player>, With<Ship>)>,
    contract_query: Query<(Entity, &ContractDetails), (With<Contract>, Without<AcceptedContract>)>,
    active_contract_query: Query<(Entity, &ContractDetails), (With<Contract>, With<AcceptedContract>)>,
    intel_query: Query<(Entity, &IntelData), (With<Intel>, With<TavernIntel>, Without<AcquiredIntel>)>,
    player_contracts: Res<PlayerContracts>,
    tavern_companions: Res<crate::plugins::companion::TavernCompanions>,
    companion_query: Query<&crate::components::companion::CompanionRole, With<crate::components::companion::Companion>>,
    ui_assets: Res<UiAssets>,
) {
    // Check key input to close port view
    if contexts.ctx_mut().input(|i| i.key_pressed(egui::Key::Escape)) {
        next_state.set(GameState::HighSeas);
        return;
    }

    // Get player data
    let player_data = player_query.get_single().ok();
    let player_gold = player_data.and_then(|(_, _, g)| g.map(|g| g.0)).unwrap_or(0);
    let player_cargo = player_data.and_then(|(_, c, _)| c);
    
    // Check for Quartermaster
    let has_quartermaster = companion_query.iter().any(|r| matches!(r, crate::components::companion::CompanionRole::Quartermaster));

    let texture_id = contexts.add_image(ui_assets.parchment_texture.clone());

    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        // Draw parchment background
        crate::plugins::ui_theme::draw_parchment_bg(ui, texture_id);

        let port_name = current_port.entity
            .and_then(|e| port_query.get(e).ok())
            .map(|(_, name, _)| name.0.as_str())
            .unwrap_or("Unknown Port");

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
        
        ui.horizontal(|ui| {
            if ui.selectable_label(ui_state.selected_tab == 0, "Market").clicked() { ui_state.selected_tab = 0; }
            if ui.selectable_label(ui_state.selected_tab == 1, "Tavern").clicked() { ui_state.selected_tab = 1; }
            if ui.selectable_label(ui_state.selected_tab == 2, "Docks").clicked() { ui_state.selected_tab = 2; }
            if ui.selectable_label(ui_state.selected_tab == 3, "Contracts").clicked() { ui_state.selected_tab = 3; }
        });
        
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            match ui_state.selected_tab {
                0 => render_market_panel(
                    ui, 
                    current_port.entity, 
                    current_port.entity.and_then(|e| port_query.get(e).ok()).map(|p| p.2), // p.2 is &Inventory
                    player_gold, 
                    player_cargo, 
                    &mut events.trade,
                    has_quartermaster,
                    &mut events.auto_trade,
                ),
                1 => render_tavern_panel(
                    ui,
                    current_port.entity,
                    player_gold,
                    &intel_query,
                    &mut events.intel,
                    &tavern_companions,
                    &mut events.companion,
                ),
                2 => render_docks_panel(ui, player_data.map(|(h, _, _)| h), player_gold, &mut events.repair),
                3 => render_contracts_panel(
                    ui,
                    current_port.entity,
                    &contract_query,
                    &active_contract_query,
                    &player_contracts,
                    &mut events.contract,
                ),
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
    has_quartermaster: bool,
    auto_trade_events: &mut EventWriter<crate::plugins::companion::AutoTradeEvent>,
) {
    ui.horizontal(|ui| {
        ui.heading("Market");
        if has_quartermaster {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("⚡ Auto-Trade").on_hover_text("Quartermaster: Automatically buy low and sell high.").clicked() {
                     if let Some(port) = port_entity {
                         auto_trade_events.send(crate::plugins::companion::AutoTradeEvent { port_entity: port });
                     }
                }
            });
        }
    });
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

/// Generates contracts for ports when entering port state.
/// Each contract receives an expiry time based on the current WorldClock.
fn generate_port_contracts(
    mut commands: Commands,
    port_query: Query<Entity, With<Port>>,
    existing_contracts: Query<Entity, With<Contract>>,
    world_clock: Res<crate::resources::WorldClock>,
) {
    use crate::components::cargo::GoodType;
    use rand::Rng;
    
    // Don't regenerate if contracts exist
    if existing_contracts.iter().count() > 0 {
        return;
    }
    
    let current_tick = world_clock.total_ticks();
    let mut rng = rand::thread_rng();
    let ports: Vec<Entity> = port_query.iter().collect();
    
    if ports.len() < 2 {
        warn!("Not enough ports to generate contracts");
        return;
    }
    
    // Generate 2-4 contracts per port
    for &origin_port in &ports {
        let num_contracts = rng.gen_range(2..=4);
        
        for _ in 0..num_contracts {
            // Pick a random destination different from origin
            let dest_port = loop {
                let idx = rng.gen_range(0..ports.len());
                if ports[idx] != origin_port {
                    break ports[idx];
                }
            };
            
            // Random good type
            let good = match rng.gen_range(0..6) {
                0 => GoodType::Rum,
                1 => GoodType::Sugar,
                2 => GoodType::Spices,
                3 => GoodType::Timber,
                4 => GoodType::Cloth,
                _ => GoodType::Weapons,
            };
            
            let quantity = rng.gen_range(5..=20);
            let reward = quantity * rng.gen_range(15..=30);
            
            commands.spawn((
                Contract,
                ContractDetails::transport_with_expiry(
                    origin_port, dest_port, good, quantity, reward, current_tick
                ),
            ));
        }
    }
    
    info!("Generated {} contracts with expiry for {} ports", 
        ports.len() * 3, // approximate count
        ports.len()
    );
}

/// Renders the Contracts panel.
fn render_contracts_panel(
    ui: &mut egui::Ui,
    current_port: Option<Entity>,
    available_query: &Query<(Entity, &ContractDetails), (With<Contract>, Without<AcceptedContract>)>,
    active_query: &Query<(Entity, &ContractDetails), (With<Contract>, With<AcceptedContract>)>,
    player_contracts: &PlayerContracts,
    contract_events: &mut EventWriter<ContractAcceptedEvent>,
) {
    ui.heading("Contracts");
    ui.label("Accept jobs for gold and reputation.");
    ui.add_space(10.0);
    
    // Show active contracts first
    if !player_contracts.active.is_empty() {
        ui.group(|ui| {
            ui.strong("📋 Active Contracts");
            ui.add_space(5.0);
            
            for (entity, details) in active_query.iter() {
                if player_contracts.active.contains(&entity) {
                    ui.horizontal(|ui| {
                        ui.label(format!("• {} - 💰{}", details.description, details.reward_gold));
                    });
                }
            }
        });
        ui.add_space(10.0);
    }
    
    // Show available contracts at this port
    ui.strong("📜 Available Contracts");
    ui.add_space(5.0);
    
    let Some(port_entity) = current_port else {
        ui.label("No port selected.");
        return;
    };
    
    let mut contracts_at_port = 0;
    egui::Grid::new("contracts_grid")
        .num_columns(3)
        .striped(true)
        .min_col_width(100.0)
        .show(ui, |ui| {
            ui.strong("Description");
            ui.strong("Reward");
            ui.strong("Action");
            ui.end_row();
            
            for (entity, details) in available_query.iter() {
                if details.origin_port == port_entity {
                    contracts_at_port += 1;
                    ui.label(&details.description);
                    ui.label(format!("💰{}", details.reward_gold));
                    if ui.button("Accept").clicked() {
                        contract_events.send(ContractAcceptedEvent {
                            contract_entity: entity,
                        });
                    }
                    ui.end_row();
                }
            }
        });
    
    if contracts_at_port == 0 {
        ui.label("No contracts available at this port.");
    }
}

/// System that handles contract acceptance.
fn contract_acceptance_system(
    mut commands: Commands,
    mut events: EventReader<ContractAcceptedEvent>,
    mut player_contracts: ResMut<PlayerContracts>,
) {
    for event in events.read() {
        // Add AcceptedContract marker and progress tracking
        commands.entity(event.contract_entity).insert((
            AcceptedContract,
            ContractProgress::default(),
        ));
        
        player_contracts.active.push(event.contract_entity);
        
        info!("Contract {:?} accepted!", event.contract_entity);
    }
}

/// Renders the Tavern panel with intel for purchase.
fn render_tavern_panel(
    ui: &mut egui::Ui,
    current_port: Option<Entity>,
    player_gold: u32,
    intel_query: &Query<(Entity, &IntelData), (With<Intel>, With<TavernIntel>, Without<AcquiredIntel>)>,
    intel_events: &mut EventWriter<IntelAcquiredEvent>,
    tavern_companions: &crate::plugins::companion::TavernCompanions,
    recruit_events: &mut EventWriter<crate::plugins::companion::CompanionRecruitedEvent>,
) {
    ui.heading("Tavern");
    ui.label("Gather intelligence and recruit crew.");
    ui.add_space(10.0);
    
    let Some(port_entity) = current_port else {
        ui.label("No port selected.");
        return;
    };
    
    ui.group(|ui| {
        ui.strong("🗣️ Available Intel");
        ui.add_space(5.0);
        
        let mut intel_count = 0;
        egui::Grid::new("intel_grid")
            .num_columns(3)
            .striped(true)
            .min_col_width(100.0)
            .show(ui, |ui| {
                ui.strong("Information");
                ui.strong("Cost");
                ui.strong("Action");
                ui.end_row();
                
                for (entity, intel_data) in intel_query.iter() {
                    // Only show intel from this port
                    if intel_data.source_port != Some(port_entity) {
                        continue;
                    }
                    intel_count += 1;
                    
                    // Intel type icon and description
                    let icon = match intel_data.intel_type {
                        IntelType::ShipRoute => "🚢",
                        IntelType::PortInventory => "📦",
                        IntelType::TreasureLocation => "💎",
                        IntelType::Rumor => "💬",
                        IntelType::FleetPosition => "⚓",
                        IntelType::MapReveal => "🗺️",
                    };
                    ui.label(format!("{} {}", icon, intel_data.description));
                    ui.label(format!("💰{}", intel_data.purchase_cost));
                    
                    let can_afford = player_gold >= intel_data.purchase_cost;
                    if ui.add_enabled(can_afford, egui::Button::new("Buy")).clicked() {
                        intel_events.send(IntelAcquiredEvent {
                            intel_entity: entity,
                            intel_type: intel_data.intel_type,
                            source_port: Some(port_entity),
                        });
                    }
                    ui.end_row();
                }
            });
        
        if intel_count == 0 {
            ui.add_space(5.0);
            ui.label("No rumors today...");
            ui.weak("(Try another port)");
        }
    });
    
    render_recruitment_section(ui, player_gold, tavern_companions, recruit_events);
}

/// Renders the Recruitment section within the Tavern panel.
fn render_recruitment_section(
    ui: &mut egui::Ui,
    player_gold: u32,
    tavern_companions: &crate::plugins::companion::TavernCompanions,
    recruit_events: &mut EventWriter<crate::plugins::companion::CompanionRecruitedEvent>,
) {
    ui.add_space(20.0);
    ui.group(|ui| {
        ui.strong("👥 Crew Recruitment");
        ui.add_space(5.0);
        
        if tavern_companions.available.is_empty() {
             ui.label("No willing souls found in this tavern.");
             return;
        }

        egui::Grid::new("recruitment_grid")
            .num_columns(4)
            .striped(true)
            .min_col_width(80.0)
            .show(ui, |ui| {
                ui.strong("Name");
                ui.strong("Role");
                ui.strong("Cost");
                ui.strong("Action");
                ui.end_row();

                for companion in &tavern_companions.available {
                    ui.label(&companion.name);
                    
                    // Show role with description tooltip
                    let role_name = companion.role.name();
                    let role_desc = companion.role.description();
                    ui.label(role_name).on_hover_text(role_desc);
                    
                    ui.label(format!("💰{}", companion.cost));
                    
                    let can_afford = player_gold >= companion.cost;
                    if ui.add_enabled(can_afford, egui::Button::new("Recruit")).clicked() {
                        recruit_events.send(crate::plugins::companion::CompanionRecruitedEvent {
                            companion_id: companion.id,
                        });
                    }
                    ui.end_row();
                }
            });
    });
}

/// Renders the Docks panel with ship repair options.
fn render_docks_panel(
    ui: &mut egui::Ui,
    health: Option<&Health>,
    player_gold: u32,
    repair_events: &mut EventWriter<RepairRequestEvent>,
) {
    ui.heading("Docks");
    ui.label("Repair and upgrade your ship.");
    ui.add_space(10.0);
    
    if let Some(health) = health {
        ui.group(|ui| {
            ui.label("Ship Status:");
            ui.add_space(5.0);
            
            // Sails
            let sails_damage = health.sails_max - health.sails;
            let sails_cost = calculate_repair_cost(RepairType::Sails, sails_damage);
            let sails_pct = health.sails / health.sails_max;
            ui.horizontal(|ui| {
                ui.label("Sails:");
                ui.add(egui::ProgressBar::new(sails_pct)
                    .text(format!("{:.0}/{:.0}", health.sails, health.sails_max))
                    .fill(if sails_pct > 0.5 { egui::Color32::from_rgb(100, 180, 100) } else { egui::Color32::from_rgb(200, 150, 50) })
                );
                if sails_pct < 1.0 {
                    let can_afford = player_gold >= sails_cost;
                    let button_text = format!("Repair ({}g)", sails_cost);
                    if ui.add_enabled(can_afford, egui::Button::new(button_text).small()).clicked() {
                        repair_events.send(RepairRequestEvent { repair_type: RepairType::Sails });
                    }
                }
            });
            
            // Rudder
            let rudder_damage = health.rudder_max - health.rudder;
            let rudder_cost = calculate_repair_cost(RepairType::Rudder, rudder_damage);
            let rudder_pct = health.rudder / health.rudder_max;
            ui.horizontal(|ui| {
                ui.label("Rudder:");
                ui.add(egui::ProgressBar::new(rudder_pct)
                    .text(format!("{:.0}/{:.0}", health.rudder, health.rudder_max))
                    .fill(if rudder_pct > 0.5 { egui::Color32::from_rgb(100, 180, 100) } else { egui::Color32::from_rgb(200, 150, 50) })
                );
                if rudder_pct < 1.0 {
                    let can_afford = player_gold >= rudder_cost;
                    let button_text = format!("Repair ({}g)", rudder_cost);
                    if ui.add_enabled(can_afford, egui::Button::new(button_text).small()).clicked() {
                        repair_events.send(RepairRequestEvent { repair_type: RepairType::Rudder });
                    }
                }
            });
            
            // Hull
            let hull_damage = health.hull_max - health.hull;
            let hull_cost = calculate_repair_cost(RepairType::Hull, hull_damage);
            let hull_pct = health.hull / health.hull_max;
            ui.horizontal(|ui| {
                ui.label("Hull:");
                ui.add(egui::ProgressBar::new(hull_pct)
                    .text(format!("{:.0}/{:.0}", health.hull, health.hull_max))
                    .fill(if hull_pct > 0.5 { egui::Color32::from_rgb(100, 180, 100) } else { egui::Color32::from_rgb(180, 80, 80) })
                );
                if hull_pct < 1.0 {
                    let can_afford = player_gold >= hull_cost;
                    let button_text = format!("Repair ({}g)", hull_cost);
                    if ui.add_enabled(can_afford, egui::Button::new(button_text).small()).clicked() {
                        repair_events.send(RepairRequestEvent { repair_type: RepairType::Hull });
                    }
                }
            });
        });
    } else {
        ui.label("⚠ No ship data available");
        ui.weak("(Player ship not found)");
    }
}

/// Generates intel available for purchase at taverns when entering port state.
fn generate_tavern_intel(
    mut commands: Commands,
    port_query: Query<Entity, With<Port>>,
    existing_intel: Query<Entity, With<TavernIntel>>,
    world_clock: Res<crate::resources::WorldClock>,
) {
    use rand::Rng;
    
    // Don't regenerate if intel exists
    if existing_intel.iter().count() > 0 {
        return;
    }
    
    let current_tick = world_clock.total_ticks();
    let mut rng = rand::thread_rng();
    let ports: Vec<Entity> = port_query.iter().collect();
    
    if ports.is_empty() {
        warn!("No ports to generate intel for");
        return;
    }
    
    // Generate 2-4 intel items per port
    for &port_entity in &ports {
        let num_intel = rng.gen_range(2..=4);
        
        for _ in 0..num_intel {
            // Random intel type with weighted distribution
            let intel_type = match rng.gen_range(0..10) {
                0..=3 => IntelType::Rumor,           // 40% rumors
                4..=5 => IntelType::MapReveal,       // 20% map reveals
                6..=7 => IntelType::ShipRoute,       // 20% ship routes
                8 => IntelType::TreasureLocation,    // 10% treasure
                _ => IntelType::FleetPosition,       // 10% fleet positions
            };
            
            // Generate description and cost based on type
            let (description, cost, positions) = match intel_type {
                IntelType::Rumor => {
                    let rumors = [
                        "A merchant fleet was spotted heading north",
                        "Pirates have been raiding the eastern waters",
                        "A storm sank a treasure ship last week",
                        "The navy is patrolling near the southern islands",
                    ];
                    let desc = rumors[rng.gen_range(0..rumors.len())].to_string();
                    (desc, rng.gen_range(10..=30), Vec::new())
                }
                IntelType::MapReveal => {
                    // Reveal a random area of the map
                    let center_x = rng.gen_range(50..450);
                    let center_y = rng.gen_range(50..450);
                    let radius = rng.gen_range(5..=15);
                    let mut positions = Vec::new();
                    for dx in -radius..=radius {
                        for dy in -radius..=radius {
                            if dx * dx + dy * dy <= radius * radius {
                                positions.push(IVec2::new(center_x + dx, center_y + dy));
                            }
                        }
                    }
                    let desc = format!("Map of a region ({} tiles)", positions.len());
                    (desc, rng.gen_range(30..=80), positions)
                }
                IntelType::ShipRoute => {
                    let desc = "Trade route between nearby ports".to_string();
                    (desc, rng.gen_range(40..=100), Vec::new())
                }
                IntelType::TreasureLocation => {
                    let x = rng.gen_range(50..450);
                    let y = rng.gen_range(50..450);
                    let desc = "Location of hidden treasure".to_string();
                    (desc, rng.gen_range(80..=200), vec![IVec2::new(x, y)])
                }
                IntelType::FleetPosition => {
                    let x = rng.gen_range(50..450);
                    let y = rng.gen_range(50..450);
                    let desc = "Last known position of a fleet".to_string();
                    (desc, rng.gen_range(50..=120), vec![IVec2::new(x, y)])
                }
                IntelType::PortInventory => {
                    let desc = "Port market prices".to_string();
                    (desc, rng.gen_range(20..=50), Vec::new())
                }
            };
            
            let intel_data = IntelData {
                intel_type,
                source_port: Some(port_entity),
                target_entity: None,
                revealed_positions: positions,
                route_waypoints: Vec::new(),
                description,
                purchase_cost: cost,
            };
            
            commands.spawn((
                Intel,
                intel_data,
                TavernIntel,
                IntelExpiry::new(current_tick),
            ));
        }
    }
    
    info!("Generated tavern intel for {} ports", ports.len());
}

/// System that processes intel purchases from the tavern.
fn intel_purchase_system(
    mut events: EventReader<IntelAcquiredEvent>,
    mut player_query: Query<&mut Gold, (With<Player>, With<Ship>)>,
    intel_query: Query<&IntelData, With<Intel>>,
) {
    for event in events.read() {
        // Get intel data to check cost
        let Ok(intel_data) = intel_query.get(event.intel_entity) else {
            continue;
        };
        
        // Deduct gold from player
        let Ok(mut gold) = player_query.get_single_mut() else {
            warn!("Intel purchase failed: Player not found");
            continue;
        };
        
        if gold.spend(intel_data.purchase_cost) {
            info!(
                "Purchased intel for {} gold: {}",
                intel_data.purchase_cost, intel_data.description
            );
        } else {
            warn!(
                "Intel purchase failed: Insufficient gold ({} < {})",
                gold.0, intel_data.purchase_cost
            );
        }
    }
}
