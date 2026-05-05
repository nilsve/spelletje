/// Abstracted game input, decoupled from macroquad for testability.

pub mod gamepad;
pub use gamepad::*;

/// Per-player input with analog stick support.
#[derive(Clone, Debug, Default)]
pub struct PlayerInput {
    pub move_x: f32,
    pub move_z: f32,
    pub jump: bool,
    pub shoot: bool,
}

/// Backward-compatible input struct for keyboard/mouse input.
#[derive(Clone, Debug, Default)]
pub struct Input {
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub forward: bool,
    pub backward: bool,
    pub shoot: bool,
}

/// Reads hardware input and produces an Input state.
/// Implementations can be mocked in tests.
pub trait InputSource {
    fn read(&self) -> Input;
}

/// Reads hardware input and produces GameInput for all players.
pub trait GamepadInputSource {
    fn read(&self) -> GameInput;
}

/// Maximum number of simultaneous gamepad players.
pub const MAX_GAMEPAD_PLAYERS: usize = 8;

/// Combined input from all players in a game.
/// Supports up to MAX_GAMEPAD_PLAYERS connected gamepads.
/// Players without a gamepad get default (zeroed) input.
#[derive(Clone, Debug, Default)]
pub struct GameInput {
    pub players: Vec<PlayerInput>,
}

impl GameInput {
    /// Returns the number of active players (connected gamepads).
    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    /// Returns input for a specific player index, or default if no gamepad.
    pub fn get(&self, index: usize) -> Option<&PlayerInput> {
        self.players.get(index)
    }

    /// Returns a slice of all player inputs.
    pub fn as_slice(&self) -> &[PlayerInput] {
        &self.players
    }
}

#[cfg(all(not(test), feature = "gui"))]
pub struct MacroquadInput;

#[cfg(all(not(test), feature = "gui"))]
impl InputSource for MacroquadInput {
    fn read(&self) -> Input {
        use macroquad::prelude::*;
        Input {
            left: is_key_down(KeyCode::A),
            right: is_key_down(KeyCode::D),
            jump: is_key_pressed(KeyCode::Space),
            forward: is_key_down(KeyCode::S),
            backward: is_key_down(KeyCode::W),
            shoot: is_mouse_button_pressed(MouseButton::Left),
        }
    }
}

impl From<&Input> for PlayerInput {
    fn from(value: &Input) -> Self {
        Self {
            move_x: if value.left { -1.0 } else if value.right { 1.0 } else { 0.0 },
            move_z: {
               let mut z = 0.0;
               if value.forward { z += 1.0; }
               if value.backward { z -= 1.0; }
               z
           },
            jump: value.jump,
            shoot: value.shoot,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_default_is_all_false() {
        let input = Input::default();
        assert!(!input.left);
        assert!(!input.right);
        assert!(!input.jump);
        assert!(!input.forward);
        assert!(!input.backward);
        assert!(!input.shoot);
    }

    #[test]
    fn test_input_clone() {
        let input = Input {
            left: true,
            right: false,
            jump: true,
            forward: false,
            backward: false,
            shoot: true,
        };
        let cloned = input.clone();
        assert!(cloned.left);
        assert!(!cloned.right);
        assert!(cloned.jump);
        assert!(cloned.shoot);
    }

    #[test]
    fn test_input_debug_format() {
        let input = Input::default();
        let debug_str = format!("{:?}", input);
        assert!(debug_str.contains("Input"));
    }

    #[test]
    fn test_input_partial_update() {
        let mut input = Input::default();
        input.left = true;
        assert!(input.left);
        assert!(!input.right);
    }

    #[test]
    fn test_player_input_default_is_all_zero() {
        let input = PlayerInput::default();
        assert!((input.move_x - 0.0).abs() < 0.001);
        assert!((input.move_z - 0.0).abs() < 0.001);
        assert!(!input.jump);
        assert!(!input.shoot);
    }

    #[test]
    fn test_player_input_clone() {
        let input = PlayerInput {
            move_x: 0.8,
            move_z: -0.5,
            jump: true,
            shoot: true,
        };
        let cloned = input.clone();
        assert!((cloned.move_x - 0.8).abs() < 0.001);
        assert!((cloned.move_z - (-0.5)).abs() < 0.001);
        assert!(cloned.jump);
        assert!(cloned.shoot);
    }

    #[test]
    fn test_player_input_debug_format() {
        let input = PlayerInput::default();
        let debug_str = format!("{:?}", input);
        assert!(debug_str.contains("PlayerInput"));
    }

    #[test]
    fn test_game_input_default_empty() {
        let game_input = GameInput::default();
        assert_eq!(game_input.player_count(), 0);
        assert!(game_input.players.is_empty());
    }

    #[test]
    fn test_game_input_with_players() {
        let game_input = GameInput {
            players: vec![
                PlayerInput {
                    move_x: 1.0,
                    move_z: 0.0,
                    jump: true,
                    shoot: false,
                },
                PlayerInput {
                    move_x: -1.0,
                    move_z: 0.0,
                    jump: false,
                    shoot: true,
                },
            ],
        };
        let cloned = game_input.clone();
        assert_eq!(cloned.player_count(), 2);
        assert!((cloned.players[0].move_x - 1.0).abs() < 0.001);
        assert!((cloned.players[1].move_x - (-1.0)).abs() < 0.001);
        assert!(cloned.players[0].jump);
        assert!(cloned.players[1].shoot);
    }

    #[test]
    fn test_game_input_get() {
        let game_input = GameInput {
            players: vec![
                PlayerInput {
                    move_x: 0.5,
                    ..Default::default()
                },
                PlayerInput {
                    move_x: -0.5,
                    ..Default::default()
                },
            ],
        };
        assert!(game_input.get(0).is_some());
        assert!(game_input.get(1).is_some());
        assert!(game_input.get(2).is_none());
        assert!((game_input.get(0).unwrap().move_x - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_game_input_as_slice() {
        let game_input = GameInput {
            players: vec![
                PlayerInput::default(),
                PlayerInput {
                    jump: true,
                    ..Default::default()
                },
            ],
        };
        let slice = game_input.as_slice();
        assert_eq!(slice.len(), 2);
        assert!(!slice[0].jump);
        assert!(slice[1].jump);
    }

    #[test]
    fn test_max_gamepad_players_constant() {
        assert_eq!(MAX_GAMEPAD_PLAYERS, 8);
    }

    #[test]
    fn test_gamepad_deadzone_imported() {
        assert!((DEFAULT_DEADZONE - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_deadzone_function() {
        assert!((apply_deadzone(0.1, 0.2) - 0.0).abs() < 0.001);
        assert!((apply_deadzone(0.5, 0.2) - 0.5).abs() < 0.001);
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_gamepad_btn_constants() {
        assert_eq!(gamepad_btn::BUTTON_A as i32, gilrs::Button::South as i32);
        assert_eq!(gamepad_btn::BUTTON_X as i32, gilrs::Button::West as i32);
        assert_eq!(gamepad_btn::LEFT_TRIGGER as i32, gilrs::Button::LeftTrigger2 as i32);
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_gamepad_axis_constants() {
        assert_eq!(gamepad_axis::LEFT_X as i32, gilrs::Axis::LeftStickX as i32);
        assert_eq!(gamepad_axis::RIGHT_X as i32, gilrs::Axis::RightStickX as i32);
    }
}
