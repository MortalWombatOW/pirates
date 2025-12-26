//! FadeController plugin - provides smooth alpha fade animations.
//!
//! Registers the `animate_fades` system that lerps FadeController.current_alpha
//! toward target_alpha each frame. Also handles region-based fade triggers.

use bevy::prelude::*;

use crate::components::fade_controller::FadeController;
use crate::components::region::{CurrentRegion, RegionResponsive};
use crate::plugins::core::GameState;
use crate::plugins::worldmap::HighSeasPlayer;
use crate::resources::MapData;
use crate::utils::pathfinding::world_to_tile;

pub struct FadeControllerPlugin;

impl Plugin for FadeControllerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CurrentRegion>()
            .add_systems(
                Update,
                (
                    update_current_region,
                    trigger_region_fades.after(update_current_region),
                    animate_fades,
                ).run_if(in_state(GameState::HighSeas)),
            );
    }
}

/// System that updates CurrentRegion based on player position.
/// Detects when player is near ports or in specific map areas.
fn update_current_region(
    player_query: Query<&Transform, With<HighSeasPlayer>>,
    map_data: Res<MapData>,
    mut current_region: ResMut<CurrentRegion>,
) {
    let Ok(player_transform) = player_query.get_single() else { return; };
    let player_pos = player_transform.translation.truncate();
    
    // Convert to tile coordinates
    let tile_pos = world_to_tile(player_pos, map_data.width, map_data.height);
    
    // Get tile at player position
    let Some(tile) = map_data.tile(tile_pos.x as u32, tile_pos.y as u32) else {
        return;
    };
    
    // Determine region based on tile type
    let new_region = if tile.tile_type.is_port() {
        Some("port_area".to_string())
    } else if tile.tile_type.is_navigable() {
        Some("open_sea".to_string())
    } else {
        None
    };
    
    // Only update if changed (to trigger Changed<CurrentRegion>)
    if current_region.name != new_region {
        current_region.name = new_region;
    }
}

/// System that animates all FadeController components.
/// Lerps current_alpha toward target_alpha using delta time.
fn animate_fades(
    time: Res<Time>,
    mut query: Query<&mut FadeController>,
) {
    let dt = time.delta_secs();

    for mut fade in &mut query {
        if !fade.is_fading() {
            continue;
        }

        let direction = if fade.target_alpha > fade.current_alpha { 1.0 } else { -1.0 };
        let delta = fade.fade_speed * dt * direction;

        fade.current_alpha = if direction > 0.0 {
            (fade.current_alpha + delta).min(fade.target_alpha)
        } else {
            (fade.current_alpha + delta).max(fade.target_alpha)
        };
    }
}

/// System that updates FadeController targets based on CurrentRegion changes.
/// When the player enters/exits a region, RegionResponsive entities fade accordingly.
fn trigger_region_fades(
    region: Res<CurrentRegion>,
    mut query: Query<(&RegionResponsive, &mut FadeController)>,
) {
    // Only run when region changes
    if !region.is_changed() {
        return;
    }

    for (responsive, mut fade) in &mut query {
        let is_in_region = region.is_in(&responsive.region_name);

        // Determine target alpha based on enter/exit behavior
        let target = if is_in_region == responsive.fade_in_on_enter {
            1.0 // Show
        } else {
            0.0 // Hide
        };

        // Only update if target changed
        if (fade.target_alpha - target).abs() > 0.001 {
            if target > 0.5 {
                fade.fade_in(responsive.fade_duration);
            } else {
                fade.fade_out(responsive.fade_duration);
            }
        }
    }
}

