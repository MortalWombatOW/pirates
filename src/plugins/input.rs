use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum PlayerAction {
    Thrust,
    TurnLeft,
    TurnRight,
    Fire,
    Anchor,
    CycleTarget,
    #[actionlike(DualAxis)]
    CameraMove,
    #[actionlike(Axis)]
    CameraZoom,
    CameraDrag,
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default());
    }
}

pub fn get_default_input_map() -> InputMap<PlayerAction> {
    let mut input_map = InputMap::default();
    
    // Movement (Ship)
    input_map.insert(PlayerAction::Thrust, KeyCode::KeyW);
    input_map.insert(PlayerAction::TurnLeft, KeyCode::KeyA);
    input_map.insert(PlayerAction::TurnRight, KeyCode::KeyD);
    
    // Actions
    input_map.insert(PlayerAction::Fire, KeyCode::Space);
    input_map.insert(PlayerAction::Anchor, KeyCode::ShiftLeft);
    input_map.insert(PlayerAction::CycleTarget, KeyCode::Tab);
    
    // Camera (arrow keys for pan, scroll for zoom)
    // Note: MouseMove removed - was causing camera to fly away on any mouse movement
    // TODO: Implement proper mouse drag with CameraDrag action + modifier button
    input_map.insert_dual_axis(PlayerAction::CameraMove, VirtualDPad::arrow_keys());
    input_map.insert_axis(PlayerAction::CameraZoom, MouseScrollAxis::Y);
    
    input_map
}
