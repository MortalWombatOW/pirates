//! Order types for AI ship behavior.
//!
//! Orders define what an AI-controlled ship should do. Ships execute orders
//! from their OrderQueue to perform trade, patrol, escort, and scouting tasks.

use bevy::prelude::*;

/// An order that can be assigned to an AI ship.
/// 
/// Orders are processed by the OrderExecutionSystem which translates them
/// into navigation targets and actions.
#[derive(Clone, Debug, Reflect)]
pub enum Order {
    /// Navigate between two ports trading goods.
    /// The ship moves to origin, loads cargo, then moves to destination, sells cargo, repeats.
    TradeRoute {
        /// Starting port entity.
        origin: Entity,
        /// Destination port entity.
        destination: Entity,
        /// Current leg of the journey: true = going to destination, false = returning to origin.
        outbound: bool,
    },

    /// Patrol an area, engaging hostile ships encountered.
    Patrol {
        /// Center of the patrol area in world coordinates.
        center: Vec2,
        /// Radius of the patrol area.
        radius: f32,
        /// Current patrol waypoint index (cycles through random points).
        waypoint_index: u32,
    },

    /// Follow and protect a target entity.
    Escort {
        /// Entity to escort (player ship, merchant, etc.).
        target: Entity,
        /// Preferred distance to maintain from target.
        follow_distance: f32,
    },

    /// Explore an area and report discoveries.
    Scout {
        /// Center of the scouting area in world coordinates.
        area_center: Vec2,
        /// Radius of the area to scout.
        area_radius: f32,
        /// Percentage of area explored (0.0 to 1.0).
        progress: f32,
    },

    /// Idle at current position, awaiting further orders.
    Idle,
}

impl Default for Order {
    fn default() -> Self {
        Order::Idle
    }
}
