use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::resources::{PlayerFleet, FleetEntities};
use crate::components::{OrderQueue, Order, Player, PlayerOwned, Health, Cargo};
use crate::components::contract::{Contract, ContractDetails, AcceptedContract, AssignedShip, ContractType};
use crate::systems::ai::AIState;
use crate::plugins::port_ui::PlayerContracts;
use bevy::math::Vec2;

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
    pub selected_tab: usize,
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
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut ui_state: ResMut<FleetUiState>,
    player_fleet: Res<PlayerFleet>,
    fleet_entities: Res<FleetEntities>,
    // Queries for render_ship_list
    ship_query: Query<(Entity, Option<&Name>, &Health, Option<&Cargo>, Option<&OrderQueue>, Option<&AIState>)>,
    order_query: Query<&OrderQueue, With<PlayerOwned>>, // Used for checking current order in list? Wait, render_ship_list does that.
    ship_transform_query: Query<&Transform, With<PlayerOwned>>, // Used? render_ship_list used it?
    player_query: Query<Entity, With<Player>>,
    // Contract queries
    player_contracts: Option<Res<PlayerContracts>>,
    contract_query: Query<(Entity, &ContractDetails, Option<&AssignedShip>), (With<Contract>, With<AcceptedContract>)>,
    companion_query: Query<(&crate::components::companion::CompanionName, &crate::components::companion::CompanionRole, Option<&crate::components::companion::AssignedTo>), With<crate::components::companion::Companion>>,
    mut _order_events: EventWriter<AssignOrderEvent>,
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
            ui.heading("Fleet Management");
            
            // Tab selection
            ui.horizontal(|ui| {
                if ui.selectable_label(ui_state.selected_tab == 0, "Ships").clicked() {
                    ui_state.selected_tab = 0;
                }
                if ui.selectable_label(ui_state.selected_tab == 1, "Companions").clicked() {
                    ui_state.selected_tab = 1;
                }
            });
            
            ui.separator();
            
            // Tab content
            match ui_state.selected_tab {
                0 => {
                    render_ship_list(ui, &mut commands, &player_fleet, &fleet_entities, &ship_query, &mut contract_events, &contract_query);
                },
                1 => {
                    render_companion_roster(ui, &companion_query);
                },
                _ => {},
            }
        });
}


fn render_ship_list(
    ui: &mut egui::Ui,
    commands: &mut Commands,
    player_ships: &PlayerFleet,
    fleet_entities: &FleetEntities,
    ship_query: &Query<(Entity, Option<&bevy::prelude::Name>, &Health, Option<&Cargo>, Option<&OrderQueue>, Option<&AIState>)>,
    contract_events: &mut EventWriter<AssignContractEvent>,
    contract_query: &Query<(Entity, &ContractDetails, Option<&AssignedShip>), (With<Contract>, With<AcceptedContract>)>,
) {
    use crate::components::order::OrderQueue;
    use std::collections::VecDeque;

    egui::ScrollArea::vertical().show(ui, |ui| {
        if player_ships.ships.is_empty() {
            ui.label("You have no ships in your fleet.");
            return;
        }

        for (i, ship_data) in player_ships.ships.iter().enumerate() {
            let entity = fleet_entities.entities.get(i).copied();
            
            if let Some(entity) = entity {
                ui.group(|ui| {
                    if let Ok((_ent, name, health, cargo, order_queue, ai_state)) = ship_query.get(entity) {
                         ui.horizontal(|ui| {
                            ui.strong(format!("{}. {}", i+1, if let Some(n) = name { n.as_str() } else { &ship_data.name }));
                            ui.label(format!("HP: {:.0}/{:.0}", health.hull, health.hull_max));
                        });
                        
                        if let Some(cargo) = cargo {
                            ui.label(format!("Cargo: {}/{}", cargo.total_units(), cargo.capacity));
                        }
                        
                        if let Some(queue) = order_queue {
                            if let Some(current) = queue.current() {
                                ui.label(format!("Order: {:?}", current));
                            } else {
                                ui.label("Idle");
                            }
                        }
                        
                        let assigned_contract = contract_query.iter().find(|(_, _, assigned)| {
                            assigned.map(|a| a.ship_entity == entity).unwrap_or(false)
                        });
                         
                        if let Some((_, details, _)) = assigned_contract {
                             ui.label(format!("Contract: {}", details.description));
                        }
    
                        ui.collapsing("Give Orders", |ui| {
                            if ui.button("Patrol Here").clicked() {
                                let mut orders = VecDeque::new();
                                orders.push_back(Order::Patrol {
                                    center: Vec2::ZERO,
                                    radius: 500.0,
                                    waypoint_index: 0,
                                });
                                commands.entity(entity).insert(OrderQueue { orders });
                            }
                            if ui.button("Clear Orders").clicked() {
                                commands.entity(entity).insert(OrderQueue::new());
                            }
                        });
                        
                        if let Some(ai) = ai_state {
                            ui.weak(format!("AI: {:?}", ai));
                        }
                    } else {
                         ui.label("Ship lost or not found.");
                    }
                });
            }
        }
    });
}


fn render_companion_roster(
    ui: &mut egui::Ui,
    companion_query: &Query<(&crate::components::companion::CompanionName, &crate::components::companion::CompanionRole, Option<&crate::components::companion::AssignedTo>), With<crate::components::companion::Companion>>,
) {
    ui.heading("Companion Roster");
    ui.add_space(5.0);
    
    if companion_query.is_empty() {
        ui.label("You have no companions. Recruit them at taverns!");
        return;
    }
    
    egui::Grid::new("roster_grid")
        .num_columns(3)
        .striped(true)
        .min_col_width(100.0)
        .show(ui, |ui| {
            ui.strong("Name");
            ui.strong("Role");
            ui.strong("Assignment");
            ui.end_row();
            
            for (name, role, assigned) in companion_query.iter() {
                ui.label(&name.0);
                ui.label(role.name()).on_hover_text(role.description());
                
                if let Some(assigned_to) = assigned {
                    ui.label(format!("Ship {:?}", assigned_to.0)); // Could lookup name if we had query access
                } else {
                    ui.label("Unassigned");
                }
                ui.end_row();
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
