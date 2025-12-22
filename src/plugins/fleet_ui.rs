use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::resources::{PlayerFleet, FleetEntities};
use crate::components::{OrderQueue, Order, Player, PlayerOwned};
use crate::components::contract::{Contract, ContractDetails, AcceptedContract, AssignedShip, ContractType};
use crate::plugins::port_ui::PlayerContracts;

/// Plugin for the Fleet Management UI.
pub struct FleetUiPlugin;

impl Plugin for FleetUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<FleetUiState>()
            .add_event::<AssignOrderEvent>()
            .add_event::<AssignContractEvent>()
            .add_systems(Update, (
                toggle_fleet_ui_system,
                fleet_ui_system,
                apply_order_assignments,
                apply_contract_assignments,
            ));
    }
}

/// Resource to track UI state.
#[derive(Resource, Default)]
pub struct FleetUiState {
    pub is_open: bool,
}

/// Event to apply an order assignment to a fleet ship.
#[derive(Event)]
pub struct AssignOrderEvent {
    pub ship_entity: Entity,
    pub order: Order,
}

/// Event to assign a contract to a fleet ship.
#[derive(Event)]
pub struct AssignContractEvent {
    pub contract_entity: Entity,
    pub ship_entity: Entity,
}

/// Order types selectable from the UI.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum OrderType {
    Escort,
    Patrol,
    Idle,
}

impl OrderType {
    fn label(&self) -> &'static str {
        match self {
            OrderType::Escort => "Escort Player",
            OrderType::Patrol => "Patrol Here",
            OrderType::Idle => "Idle",
        }
    }
}

/// System to toggle the UI with 'F' key.
fn toggle_fleet_ui_system(
    mut ui_state: ResMut<FleetUiState>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::KeyF) {
        ui_state.is_open = !ui_state.is_open;
    }
}

/// Main system to render the Fleet UI with order and contract controls.
fn fleet_ui_system(
    mut contexts: EguiContexts,
    ui_state: Res<FleetUiState>,
    player_fleet: Res<PlayerFleet>,
    fleet_entities: Res<FleetEntities>,
    order_query: Query<&OrderQueue, With<PlayerOwned>>,
    ship_transform_query: Query<&Transform, With<PlayerOwned>>,
    player_query: Query<Entity, With<Player>>,
    // Contract queries
    player_contracts: Option<Res<PlayerContracts>>,
    contract_query: Query<(Entity, &ContractDetails, Option<&AssignedShip>), (With<Contract>, With<AcceptedContract>)>,
    mut order_events: EventWriter<AssignOrderEvent>,
    mut contract_events: EventWriter<AssignContractEvent>,
) {
    if !ui_state.is_open {
        return;
    }

    let player_entity = player_query.get_single().ok();

    // Build list of unassigned Transport contracts
    let unassigned_contracts: Vec<(Entity, &ContractDetails)> = contract_query
        .iter()
        .filter(|(_, d, assigned)| {
            assigned.is_none() && d.contract_type == ContractType::Transport
        })
        .map(|(e, d, _)| (e, d))
        .collect();

    egui::Window::new("Fleet Management")
        .default_width(350.0)
        .default_height(500.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Your Fleet");
            ui.separator();

            if player_fleet.ships.is_empty() {
                ui.label("No ships in fleet.");
                ui.weak("Capture enemy ships to build your fleet!");
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, ship_data) in player_fleet.ships.iter().enumerate() {
                        let entity = fleet_entities.entities.get(i).copied();
                        
                        ui.group(|ui| {
                            ui.strong(format!("{}. {}", i + 1, ship_data.name));
                            
                            // Health bar
                            let health_pct = ship_data.hull_health / ship_data.max_hull_health;
                            ui.horizontal(|ui| {
                                ui.label("Health:");
                                ui.add(egui::ProgressBar::new(health_pct)
                                    .text(format!("{:.0}/{:.0}", ship_data.hull_health, ship_data.max_hull_health))
                                    .fill(if health_pct > 0.5 { 
                                        egui::Color32::from_rgb(100, 180, 100) 
                                    } else { 
                                        egui::Color32::from_rgb(180, 80, 80) 
                                    })
                                );
                            });

                            // Cargo summary
                            ui.horizontal(|ui| {
                                ui.label("Cargo:");
                                if let Some(cargo) = &ship_data.cargo {
                                    ui.label(format!("{}/{}", cargo.total_units(), cargo.capacity));
                                } else {
                                    ui.label("None");
                                }
                            });

                            // Order display and selection
                            if let Some(ent) = entity {
                                let current_order_type = if let Ok(queue) = order_query.get(ent) {
                                    match queue.current() {
                                        Some(Order::Escort { .. }) => OrderType::Escort,
                                        Some(Order::Patrol { .. }) => OrderType::Patrol,
                                        Some(Order::TradeRoute { .. }) => OrderType::Idle, // Show as "busy"
                                        Some(Order::Idle) | None => OrderType::Idle,
                                        _ => OrderType::Idle,
                                    }
                                } else {
                                    OrderType::Idle
                                };

                                // Check if this ship has an assigned contract
                                let has_contract = contract_query.iter()
                                    .any(|(_, _, assigned)| {
                                        assigned.map(|a| a.ship_entity == ent).unwrap_or(false)
                                    });

                                ui.horizontal(|ui| {
                                    ui.label("Order:");
                                    
                                    if has_contract {
                                        ui.label("📜 Fulfilling Contract");
                                    } else {
                                        let combo_id = format!("order_combo_{}", i);
                                        egui::ComboBox::from_id_salt(combo_id)
                                            .selected_text(current_order_type.label())
                                            .show_ui(ui, |ui| {
                                                for order_type in [OrderType::Escort, OrderType::Patrol, OrderType::Idle] {
                                                    if ui.selectable_label(
                                                        current_order_type == order_type,
                                                        order_type.label()
                                                    ).clicked() {
                                                        let new_order = match order_type {
                                                            OrderType::Escort => {
                                                                if let Some(p) = player_entity {
                                                                    Order::Escort {
                                                                        target: p,
                                                                        follow_distance: 60.0 + (i as f32 * 20.0),
                                                                    }
                                                                } else {
                                                                    Order::Idle
                                                                }
                                                            }
                                                            OrderType::Patrol => {
                                                                let center = ship_transform_query
                                                                    .get(ent)
                                                                    .map(|t| t.translation.truncate())
                                                                    .unwrap_or_default();
                                                                Order::Patrol {
                                                                    center,
                                                                    radius: 500.0,
                                                                    waypoint_index: 0,
                                                                }
                                                            }
                                                            OrderType::Idle => Order::Idle,
                                                        };
                                                        
                                                        order_events.send(AssignOrderEvent {
                                                            ship_entity: ent,
                                                            order: new_order,
                                                        });
                                                    }
                                                }
                                            });
                                    }
                                });

                                // Contract assignment dropdown (only if no contract assigned)
                                if !has_contract && !unassigned_contracts.is_empty() {
                                    ui.horizontal(|ui| {
                                        ui.label("Assign:");
                                        let contract_combo_id = format!("contract_combo_{}", i);
                                        egui::ComboBox::from_id_salt(contract_combo_id)
                                            .selected_text("📜 Contract...")
                                            .show_ui(ui, |ui| {
                                                for (contract_entity, details) in &unassigned_contracts {
                                                    let label = format!(
                                                        "{} (💰{})", 
                                                        &details.description, 
                                                        (details.reward_gold as f32 * AssignedShip::DEFAULT_CUT) as u32
                                                    );
                                                    if ui.selectable_label(false, label).clicked() {
                                                        contract_events.send(AssignContractEvent {
                                                            contract_entity: *contract_entity,
                                                            ship_entity: ent,
                                                        });
                                                    }
                                                }
                                            });
                                    });
                                }
                            }
                        });
                        ui.add_space(5.0);
                    }
                });
            }
        });
}

/// System to apply order assignments from UI events.
fn apply_order_assignments(
    mut events: EventReader<AssignOrderEvent>,
    mut query: Query<&mut OrderQueue, With<PlayerOwned>>,
) {
    for event in events.read() {
        if let Ok(mut queue) = query.get_mut(event.ship_entity) {
            queue.clear();
            queue.push(event.order.clone());
            info!("Assigned order {:?} to fleet ship {:?}", event.order, event.ship_entity);
        }
    }
}

/// System to apply contract assignments from UI events.
fn apply_contract_assignments(
    mut commands: Commands,
    mut events: EventReader<AssignContractEvent>,
    contract_query: Query<Entity, (With<Contract>, With<AcceptedContract>)>,
) {
    for event in events.read() {
        // Verify contract exists and is accepted
        if contract_query.get(event.contract_entity).is_ok() {
            commands.entity(event.contract_entity).insert(
                AssignedShip::new(event.ship_entity)
            );
            info!(
                "Contract {:?} assigned to fleet ship {:?}",
                event.contract_entity, event.ship_entity
            );
        }
    }
}
