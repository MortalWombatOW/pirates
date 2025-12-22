use bevy::prelude::*;
use crate::plugins::input::{get_default_input_map, PlayerAction};
use crate::components::{Player, Ship};
use crate::resources::{Wind, WorldClock, FactionRegistry};
use crate::systems::{wind_system, world_tick_system, price_calculation_system, goods_decay_system, contract_expiry_system, intel_expiry_system, faction_ai_system, trade_route_generation_system, faction_ship_spawning_system, faction_threat_response_system, ThreatResponseCooldown, GlobalDemand};
use crate::events::ContractExpiredEvent;
use leafwing_input_manager::prelude::*;

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    #[default]
    MainMenu,
    Port,
    HighSeas,
    Combat,
    GameOver,
}

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_resource::<Wind>()
            .init_resource::<WorldClock>()
            .init_resource::<GlobalDemand>()
            .init_resource::<ThreatResponseCooldown>()
            .insert_resource(FactionRegistry::new())
            .add_event::<ContractExpiredEvent>()
            .add_systems(Startup, (spawn_camera, init_meta_profile))
            .add_systems(Update, (
                debug_state_transitions,
                log_state_transitions,
                camera_control,
                camera_follow.run_if(in_state(GameState::Combat).or(in_state(GameState::HighSeas))),
                draw_ocean_grid,
                wind_system,
                faction_threat_response_system.run_if(in_state(GameState::HighSeas)),
            ))
            .add_systems(FixedUpdate, (
                world_tick_system,
                price_calculation_system.after(world_tick_system),
                goods_decay_system.after(world_tick_system),
                contract_expiry_system.after(world_tick_system),
                intel_expiry_system.after(world_tick_system),
                faction_ai_system.after(world_tick_system),
                trade_route_generation_system.after(faction_ai_system),
                faction_ship_spawning_system.after(trade_route_generation_system),
            ));
    }
}

fn camera_follow(
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, With<Ship>)>,
) {
    if let (Ok(mut camera_transform), Ok(player_transform)) = (camera_query.get_single_mut(), player_query.get_single()) {
        let player_pos = player_transform.translation;
        camera_transform.translation.x = player_pos.x;
        camera_transform.translation.y = player_pos.y;
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            ..default()
        },
        OrthographicProjection {
            near: -1000.0,
            far: 1000.0,
            scale: 1.0,
            ..OrthographicProjection::default_2d()
        },
        Transform::from_xyz(0.0, 0.0, 100.0),
        GlobalTransform::default(),
        InputManagerBundle::with_map(get_default_input_map()),
    ));
}

fn camera_control(
    mut query: Query<(&ActionState<PlayerAction>, &mut Transform, &mut OrthographicProjection), With<Camera2d>>,
    time: Res<Time>,
) {
    let (action_state, mut transform, mut projection) = query.single_mut();
    
    // Debug Camera
    if time.elapsed_secs() % 1.0 < 0.1 {
        // println!("Camera Pos: {:.2}, {:.2}, {:.2} | Proj: near={:.1} far={:.1} scale={:.1}", 
        //    transform.translation.x, transform.translation.y, transform.translation.z,
        //    projection.near, projection.far, projection.scale);
    }

    
    // Pan
    let axis_pair = action_state.axis_pair(&PlayerAction::CameraMove);
    if axis_pair != Vec2::ZERO {
        let move_speed = 500.0 * projection.scale;
        transform.translation.x += axis_pair.x * move_speed * time.delta_secs();
        transform.translation.y += axis_pair.y * move_speed * time.delta_secs();
    }

    // Zoom
    let zoom_delta = action_state.value(&PlayerAction::CameraZoom);
    if zoom_delta != 0.0 {
        let zoom_speed = 1.5;
        projection.scale *= 1.0 - zoom_delta * zoom_speed * time.delta_secs();
        projection.scale = projection.scale.clamp(0.1, 5.0);
    }
}

fn debug_state_transitions(
    keys: Res<ButtonInput<KeyCode>>,
    _state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Digit1) {
        next_state.set(GameState::MainMenu);
    } else if keys.just_pressed(KeyCode::Digit2) {
        next_state.set(GameState::Port);
    } else if keys.just_pressed(KeyCode::Digit3) {
        next_state.set(GameState::HighSeas);
    } else if keys.just_pressed(KeyCode::Digit4) {
        next_state.set(GameState::Combat);
    } else if keys.just_pressed(KeyCode::Digit5) {
        next_state.set(GameState::GameOver);
    }
}

fn log_state_transitions(state: Res<State<GameState>>) {
    if state.is_changed() {
        println!("Current State: {:?}", state.get());
    }
}

/// Draws a static grid to provide visual reference for movement.
fn draw_ocean_grid(mut gizmos: Gizmos) {
    let grid_size = 5000.0;
    let cell_size = 100.0;
    let color = Color::srgba(1.0, 1.0, 1.0, 0.05); // Very faint white

    gizmos.grid_2d(
        Isometry2d::IDENTITY,
        UVec2::new((grid_size / cell_size) as u32, (grid_size / cell_size) as u32),
        Vec2::splat(cell_size),
        color,
    );
}

/// Loads the MetaProfile from disk on app start.
/// Creates a fresh profile if no save file exists.
fn init_meta_profile(mut commands: Commands) {
    let profile = crate::resources::MetaProfile::load_from_file();
    info!(
        "MetaProfile loaded: {} runs completed, {} deaths, {} wrecks",
        profile.runs_completed, profile.deaths, profile.legacy_wrecks.len()
    );
    commands.insert_resource(profile);
}

