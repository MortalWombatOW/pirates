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
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default());
    }
}

pub fn get_default_input_map() -> InputMap<PlayerAction> {
    let mut input_map = InputMap::default();
    
    // Movement
    input_map.insert(PlayerAction::Thrust, KeyCode::KeyW);
    input_map.insert(PlayerAction::TurnLeft, KeyCode::KeyA);
    input_map.insert(PlayerAction::TurnRight, KeyCode::KeyD);
    
    // Actions
    input_map.insert(PlayerAction::Fire, KeyCode::Space);
    input_map.insert(PlayerAction::Anchor, KeyCode::ShiftLeft);
    input_map.insert(PlayerAction::CycleTarget, KeyCode::Tab);
    
    input_map
}
