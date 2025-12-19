use bevy::prelude::*;

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
            .add_systems(Update, (
                debug_state_transitions,
                log_state_transitions,
            ));
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
