//! Order types for AI ship behavior.
//!
//! Orders define what an AI-controlled ship should do. Ships execute orders
//! from their OrderQueue to perform trade, patrol, escort, and scouting tasks.

use bevy::prelude::*;
use std::collections::VecDeque;

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

/// Component that holds a queue of orders for an AI ship.
/// 
/// The ship processes orders from the front of the queue. When an order
/// completes, it is removed and the next order begins. Repeating orders
/// (like TradeRoute) re-add themselves to the back of the queue.
#[derive(Component, Debug, Default, Clone, Reflect)]
pub struct OrderQueue {
    /// The queue of pending orders. Front order is currently active.
    pub orders: VecDeque<Order>,
}

impl OrderQueue {
    /// Creates a new empty order queue.
    pub fn new() -> Self {
        Self {
            orders: VecDeque::new(),
        }
    }

    /// Creates a queue with a single order.
    pub fn with_order(order: Order) -> Self {
        let mut queue = Self::new();
        queue.push(order);
        queue
    }

    /// Returns the current (front) order, if any.
    pub fn current(&self) -> Option<&Order> {
        self.orders.front()
    }

    /// Returns a mutable reference to the current order.
    pub fn current_mut(&mut self) -> Option<&mut Order> {
        self.orders.front_mut()
    }

    /// Adds an order to the back of the queue.
    pub fn push(&mut self, order: Order) {
        self.orders.push_back(order);
    }

    /// Adds an order to the front of the queue (high priority).
    pub fn push_front(&mut self, order: Order) {
        self.orders.push_front(order);
    }

    /// Removes and returns the current order.
    pub fn pop(&mut self) -> Option<Order> {
        self.orders.pop_front()
    }

    /// Clears all orders from the queue.
    pub fn clear(&mut self) {
        self.orders.clear();
    }

    /// Returns true if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }

    /// Returns the number of orders in the queue.
    pub fn len(&self) -> usize {
        self.orders.len()
    }
}
