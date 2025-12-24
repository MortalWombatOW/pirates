use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy_egui::{egui, EguiContexts};
use crate::plugins::core::GameState;
use crate::resources::{Wind, WorldClock, MapData};
use crate::components::{Ship, AI, Health, Order, OrderQueue, FactionId, Faction};
use crate::plugins::worldmap::{HighSeasAI, WorldMap, FogMap};
use crate::utils::pathfinding::tile_to_world;

/// Resource to track debug visibility toggles.
#[derive(Resource)]
pub struct DebugToggles {
    pub show_tilemap: bool,
}

impl Default for DebugToggles {
    fn default() -> Self {
        Self { show_tilemap: true }
    }
}

pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<FrameTimeDiagnosticsPlugin>() {
            app.add_plugins(FrameTimeDiagnosticsPlugin::default());
        }
        
        app.init_resource::<DebugToggles>()
            .add_systems(Update, (
                debug_panel,
                apply_tilemap_visibility,
                spawn_scale_test_ships.run_if(in_state(GameState::HighSeas)),
            ));
    }
}

/// Applies tilemap visibility based on DebugToggles resource.
fn apply_tilemap_visibility(
    toggles: Res<DebugToggles>,
    mut tilemap_query: Query<&mut Visibility, Or<(With<WorldMap>, With<FogMap>)>>,
) {
    if !toggles.is_changed() {
        return;
    }
    
    let visibility = if toggles.show_tilemap {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    
    for mut vis in &mut tilemap_query {
        *vis = visibility;
    }
}

fn debug_panel(
    mut contexts: EguiContexts,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    diagnostics: Res<DiagnosticsStore>,
    wind: Option<Res<Wind>>,
    world_clock: Res<WorldClock>,
    ship_query: Query<Entity, With<Ship>>,
    mut toggles: ResMut<DebugToggles>,
) {
    egui::Window::new("Debug Panel").show(contexts.ctx_mut(), |ui| {
        ui.label(format!("Current State: {:?}", state.get()));
        
        if let Some(fps) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|diag| diag.smoothed())
        {
            ui.label(format!("FPS: {:.1}", fps));
        }

        // Ship count for scale testing
        ui.label(format!("Ships: {}", ship_query.iter().count()));

        // World Clock display
        ui.separator();
        ui.heading("World Clock");
        ui.label(world_clock.formatted_time());

        // Wind display
        if let Some(wind) = wind {
            ui.separator();
            ui.heading("Wind");
            ui.label(format!(
                "Direction: {} ({:.0}°)",
                wind.cardinal_direction(),
                wind.direction.to_degrees().rem_euclid(360.0)
            ));
            ui.label(format!("Strength: {:.0}%", wind.strength * 100.0));
        }

        // Visibility toggles
        ui.separator();
        ui.heading("Visibility");
        ui.checkbox(&mut toggles.show_tilemap, "Show Tilemap");

        ui.separator();
        ui.heading("State Transitions");
        
        if ui.button("Main Menu").clicked() {
            next_state.set(GameState::MainMenu);
        }
        if ui.button("Port").clicked() {
            next_state.set(GameState::Port);
        }
        if ui.button("High Seas").clicked() {
            next_state.set(GameState::HighSeas);
        }
        if ui.button("Combat").clicked() {
            next_state.set(GameState::Combat);
        }
        if ui.button("Game Over").clicked() {
            next_state.set(GameState::GameOver);
        }

        // Scale test info
        if state.get() == &GameState::HighSeas {
            ui.separator();
            ui.weak("Press 'B' to spawn 1000 ships for scale testing");
        }
    });
}

/// Debug system: spawns 1000 AI ships for scale testing.
/// 
/// Triggered by pressing 'B' key in HighSeas state.
/// Ships spawn on navigable water tiles with Idle orders.
fn spawn_scale_test_ships(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    map_data: Res<MapData>,
    ship_query: Query<Entity, With<Ship>>,
) {
    if !input.just_pressed(KeyCode::KeyB) {
        return;
    }

    const SHIPS_TO_SPAWN: usize = 1000;

    // Collect all navigable tile positions
    let water_tiles: Vec<(u32, u32)> = map_data.iter()
        .filter(|(_, _, tile)| tile.is_navigable())
        .map(|(x, y, _)| (x, y))
        .collect();

    if water_tiles.is_empty() {
        warn!("No navigable tiles found for scale test!");
        return;
    }

    let texture_handle: Handle<Image> = asset_server.load("sprites/ships/enemy.png");
    let map_width = map_data.width;
    let map_height = map_data.height;

    // Sample from water tiles for ship positions
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    for _ in 0..SHIPS_TO_SPAWN {
        // Random tile from water tiles
        let idx = rng.gen_range(0..water_tiles.len());
        let (tile_x, tile_y) = water_tiles[idx];
        
        // Convert to world position with small random offset
        let base_pos = tile_to_world(
            IVec2::new(tile_x as i32, tile_y as i32),
            map_width,
            map_height,
        );
        let offset = Vec2::new(
            rng.gen_range(-8.0..8.0),
            rng.gen_range(-8.0..8.0),
        );
        let spawn_pos = base_pos + offset;

        commands.spawn((
            Name::new("Scale Test Ship"),
            Ship,
            AI,
            Faction(FactionId::Pirates), // Neutral faction for testing
            HighSeasAI,
            Health::default(),
            OrderQueue::with_order(Order::Idle),
            Sprite {
                image: texture_handle.clone(),
                custom_size: Some(Vec2::splat(32.0)), // Smaller for density
                flip_y: true,
                ..default()
            },
            Transform::from_xyz(spawn_pos.x, spawn_pos.y, 1.0),
        ));
    }

    let total_ships = ship_query.iter().count() + SHIPS_TO_SPAWN;
    info!("Scale test: Spawned {} ships (total: {})", SHIPS_TO_SPAWN, total_ships);
}
