use bevy::prelude::*;

use crate::components::intel::{IntelData, IntelType, AcquiredIntel};
use crate::events::IntelAcquiredEvent;
use crate::resources::FogOfWar;

/// System that processes acquired intel and applies its effects.
/// 
/// Handles different intel types:
/// - `MapReveal`: Reveals positions in the fog of war
/// - `ShipRoute`: Stores route for visualization (handled by UI)
/// - Other types: Marks as acquired for player reference
pub fn intel_acquisition_system(
    mut events: EventReader<IntelAcquiredEvent>,
    mut commands: Commands,
    mut fog_of_war: ResMut<FogOfWar>,
    intel_query: Query<&IntelData>,
) {
    for event in events.read() {
        // Get the intel data
        let Ok(intel_data) = intel_query.get(event.intel_entity) else {
            warn!("IntelAcquiredEvent for non-existent intel entity {:?}", event.intel_entity);
            continue;
        };

        // Mark intel as acquired
        commands.entity(event.intel_entity).insert(AcquiredIntel);

        // Process based on intel type
        match intel_data.intel_type {
            IntelType::MapReveal => {
                // Reveal all positions in the fog of war
                let revealed_count = intel_data.revealed_positions
                    .iter()
                    .filter(|pos| fog_of_war.explore(**pos))
                    .count();
                
                info!(
                    "MapReveal intel acquired: {} new tiles revealed",
                    revealed_count
                );
            }
            IntelType::TreasureLocation => {
                // Reveal treasure location on map
                for pos in &intel_data.revealed_positions {
                    fog_of_war.explore(*pos);
                }
                info!(
                    "TreasureLocation intel acquired at {:?}",
                    intel_data.revealed_positions.first()
                );
            }
            IntelType::FleetPosition => {
                // Reveal fleet position on map
                for pos in &intel_data.revealed_positions {
                    fog_of_war.explore(*pos);
                }
                info!(
                    "FleetPosition intel acquired: target {:?}",
                    intel_data.target_entity
                );
            }
            IntelType::ShipRoute => {
                // Route waypoints are stored for UI visualization
                // Optionally reveal waypoint tiles
                for pos in &intel_data.route_waypoints {
                    fog_of_war.explore(*pos);
                }
                info!(
                    "ShipRoute intel acquired: {} waypoints",
                    intel_data.route_waypoints.len()
                );
            }
            IntelType::PortInventory => {
                // No map reveal, just stored for player reference
                info!(
                    "PortInventory intel acquired for port {:?}",
                    intel_data.target_entity
                );
            }
            IntelType::Rumor => {
                // Rumors may hint at other intel, no direct map effect
                info!("Rumor acquired: {}", intel_data.description);
            }
        }
    }
}
