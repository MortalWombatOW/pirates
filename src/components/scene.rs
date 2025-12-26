//! Scene marker components for entity lifecycle management.
//!
//! Entities tagged with these markers are automatically despawned
//! when their associated GameState exits via the generic
//! `despawn_scene_entities<T>` system in CorePlugin.

use bevy::prelude::*;

/// Marker for entities that belong to the HighSeas scene.
/// Despawned automatically on `GameState::HighSeas` exit.
#[derive(Component, Default)]
pub struct HighSeasEntity;

/// Marker for entities that belong to the Combat scene.
/// Despawned automatically on `GameState::Combat` exit.
#[derive(Component, Default)]
pub struct CombatEntity;

/// Marker for entities that belong to the Port scene.
/// Despawned automatically on `GameState::Port` exit.
#[derive(Component, Default)]
pub struct PortEntity;
