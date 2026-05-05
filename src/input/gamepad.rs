/// Gamepad input abstraction using gilrs.
///
/// gilrs provides a cross-platform gamepad API.
/// Steam Input works transparently with gilrs.
///
/// Usage:
///   1. Call `init()` once before the game loop
///   2. Call `poll()` each frame to pump events
///   3. Call `read_all()` to get input from all connected gamepads
///   4. Call `exit()` once after the game loop

/// Gamepad button mapping (gilrs Button enum).
#[cfg(feature = "gui")]
pub mod gamepad_btn {
    pub use gilrs::Button as Type;
    pub const BUTTON_A: gilrs::Button = gilrs::Button::South;
    pub const BUTTON_B: gilrs::Button = gilrs::Button::East;
    pub const BUTTON_X: gilrs::Button = gilrs::Button::West;
    pub const BUTTON_Y: gilrs::Button = gilrs::Button::North;
    pub const LEFT_TRIGGER: gilrs::Button = gilrs::Button::LeftTrigger2;
    pub const RIGHT_TRIGGER: gilrs::Button = gilrs::Button::RightTrigger2;
}

/// Gamepad axis mapping (gilrs Axis enum).
#[cfg(feature = "gui")]
pub mod gamepad_axis {
    pub use gilrs::Axis as Type;
    pub const LEFT_X: gilrs::Axis = gilrs::Axis::LeftStickX;
    pub const LEFT_Y: gilrs::Axis = gilrs::Axis::LeftStickY;
    pub const RIGHT_X: gilrs::Axis = gilrs::Axis::RightStickX;
    pub const RIGHT_Y: gilrs::Axis = gilrs::Axis::RightStickY;
}

/// Default deadzone for analog sticks — ignores small drift.
pub const DEFAULT_DEADZONE: f32 = 0.2;

/// Apply deadzone to an analog axis value.
/// Values within [-deadzone, deadzone] are clamped to 0.
pub fn apply_deadzone(value: f32, deadzone: f32) -> f32 {
    if value.abs() < deadzone { 0.0 } else { value }
}

/// Gamepad input state for a single player.
#[derive(Clone, Debug, Default)]
pub struct GamepadInput {
    pub move_x: f32,
    pub move_z: f32,
    pub jump: bool,
    pub shoot: bool,
    /// Right stick X for aiming (normalized -1 to 1).
    pub aim_x: f32,
    /// Right stick Y for aiming (normalized -1 to 1).
    pub aim_y: f32,
}

impl From<GamepadInput> for crate::input::PlayerInput {
    fn from(value: GamepadInput) -> Self {
        Self {
            move_x: value.move_x,
            move_z: value.move_z,
            jump: value.jump,
            shoot: value.shoot,
        }
    }
}

/// Reads gamepad input and produces GamepadInput state.
/// Only available when not running tests (uses gilrs).
///
/// Usage:
///   1. Call `poll()` each frame to pump events (must happen before reading state)
///   2. Call `read_all()` to get input from all connected gamepads
///
/// NOTE: `poll()` and `read_all()` are no-ops when no gamepad is connected.
#[cfg(all(not(test), feature = "gui"))]
pub struct GamepadInputImpl {
    pub deadzone: f32,
    gilrs: Option<gilrs::Gilrs>,
}

#[cfg(all(not(test), feature = "gui"))]
impl Default for GamepadInputImpl {
    fn default() -> Self {
        Self {
            deadzone: DEFAULT_DEADZONE,
            gilrs: None,
        }
    }
}

#[cfg(all(not(test), feature = "gui"))]
impl GamepadInputImpl {
    pub fn new() -> Self {
        let gilrs = gilrs::Gilrs::new().ok();
        Self {
            deadzone: DEFAULT_DEADZONE,
            gilrs,
        }
    }

    /// Pump pending gamepad events. Call each frame before reading gamepad state.
    /// Handles gamepad connect/disconnect events.
    /// Must be called before `read_gamepad` or `read_all` for button states to be current.
    pub fn poll(&mut self) {
        if let Some(ref mut gilrs) = self.gilrs {
            while gilrs.next_event().is_some() {}
        }
    }

    /// Read input from a specific gamepad.
    /// Left stick controls movement, right stick controls aiming.
    /// Must be called after `poll()` for accurate button state.
    pub fn read_gamepad(&self, gamepad: gilrs::Gamepad) -> GamepadInput {
        let mut input = GamepadInput::default();

        let left_x = gamepad.value(gamepad_axis::LEFT_X);
        let left_y = gamepad.value(gamepad_axis::LEFT_Y);
        let right_x = gamepad.value(gamepad_axis::RIGHT_X);
        let right_y = gamepad.value(gamepad_axis::RIGHT_Y);

        input.move_x = apply_deadzone(left_x, self.deadzone);
        input.move_z = apply_deadzone(left_y, self.deadzone);

        // Right stick always controls aiming (no deadzone for precision)
        input.aim_x = right_x;
        input.aim_y = right_y;

        // Jump: A (South) button
        input.jump = gamepad.is_pressed(gamepad_btn::BUTTON_A);

        // Shoot: Right trigger
        input.shoot = gamepad.is_pressed(gamepad_btn::RIGHT_TRIGGER);

        input
    }

    /// Read input from all connected gamepads.
    /// Returns a GameInput with one PlayerInput per connected gamepad.
    /// Gamepad order maps to player index (first connected → player 0).
    pub fn read_all(&self) -> crate::input::GameInput {
        let mut game_input = crate::input::GameInput {
            players: Vec::new(),
        };

        if let Some(ref gilrs) = self.gilrs {
            for (_id, gamepad) in gilrs.gamepads() {
                let mut input = GamepadInput::default();

                let left_x = gamepad.value(gamepad_axis::LEFT_X);
                let left_y = gamepad.value(gamepad_axis::LEFT_Y);
                let right_x = gamepad.value(gamepad_axis::RIGHT_X);
                let right_y = gamepad.value(gamepad_axis::RIGHT_Y);

                input.move_x = apply_deadzone(left_x, self.deadzone);
                input.move_z = apply_deadzone(left_y, self.deadzone);
                input.aim_x = right_x;
                input.aim_y = right_y;
                input.jump = gamepad.is_pressed(gamepad_btn::BUTTON_A);
                input.shoot = gamepad.is_pressed(gamepad_btn::RIGHT_TRIGGER);

                game_input.players.push(input.into());
            }
        }

        game_input
    }
}

#[cfg(all(not(test), feature = "gui"))]
impl crate::input::GamepadInputSource for GamepadInputImpl {
    fn read(&self) -> crate::input::GameInput {
        self.read_all()
    }
}

#[cfg(not(all(not(test), feature = "gui")))]
pub struct GamepadInputImpl;

#[cfg(not(all(not(test), feature = "gui")))]
impl Default for GamepadInputImpl {
    fn default() -> Self {
        Self
    }
}

#[cfg(not(all(not(test), feature = "gui")))]
impl GamepadInputImpl {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn poll() {}
    pub fn read_all(&self) -> crate::input::GameInput {
        crate::input::GameInput::default()
    }
}

#[cfg(not(all(not(test), feature = "gui")))]
impl crate::input::GamepadInputSource for GamepadInputImpl {
    fn read(&self) -> crate::input::GameInput {
        self.read_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::apply_deadzone;

    #[test]
    fn test_apply_deadzone_clamps_small_values() {
        assert!((apply_deadzone(0.1, 0.2) - 0.0).abs() < 0.001);
        assert!((apply_deadzone(-0.1, 0.2) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_apply_deadzone_passes_large_values() {
        assert!((apply_deadzone(0.5, 0.2) - 0.5).abs() < 0.001);
        assert!((apply_deadzone(-0.5, 0.2) - (-0.5)).abs() < 0.001);
    }

    #[test]
    fn test_apply_deadzone_at_deadzone_boundary() {
        assert!((apply_deadzone(0.2, 0.2) - 0.2).abs() < 0.001);
        assert!((apply_deadzone(-0.2, 0.2) - (-0.2)).abs() < 0.001);
    }

    #[test]
    fn test_apply_deadzone_zero() {
        assert!((apply_deadzone(0.0, 0.2) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_apply_deadzone_one() {
        assert!((apply_deadzone(1.0, 0.2) - 1.0).abs() < 0.001);
        assert!((apply_deadzone(-1.0, 0.2) - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_gamepad_input_default_is_all_zero() {
        let input = GamepadInput::default();
        assert!((input.move_x - 0.0).abs() < 0.001);
        assert!((input.move_z - 0.0).abs() < 0.001);
        assert!(!input.jump);
        assert!(!input.shoot);
    }

    #[test]
    fn test_gamepad_input_clone() {
        let input = GamepadInput {
            move_x: 0.8,
            move_z: -0.5,
            jump: true,
            shoot: true,
            ..Default::default()
        };
        let cloned = input.clone();
        assert!((cloned.move_x - 0.8).abs() < 0.001);
        assert!((cloned.move_z - (-0.5)).abs() < 0.001);
        assert!(cloned.jump);
        assert!(cloned.shoot);
    }

    #[test]
    fn test_gamepad_input_debug() {
        let input = GamepadInput::default();
        let debug_str = format!("{:?}", input);
        assert!(debug_str.contains("GamepadInput"));
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_gamepad_btn_constants() {
        assert_eq!(gamepad_btn::BUTTON_A as i32, gilrs::Button::South as i32);
        assert_eq!(gamepad_btn::BUTTON_B as i32, gilrs::Button::East as i32);
        assert_eq!(gamepad_btn::BUTTON_X as i32, gilrs::Button::West as i32);
        assert_eq!(gamepad_btn::BUTTON_Y as i32, gilrs::Button::North as i32);
        assert_eq!(
            gamepad_btn::LEFT_TRIGGER as i32,
            gilrs::Button::LeftTrigger2 as i32
        );
        assert_eq!(
            gamepad_btn::RIGHT_TRIGGER as i32,
            gilrs::Button::RightTrigger2 as i32
        );
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_gamepad_axis_constants() {
        assert_eq!(gamepad_axis::LEFT_X as i32, gilrs::Axis::LeftStickX as i32);
        assert_eq!(gamepad_axis::LEFT_Y as i32, gilrs::Axis::LeftStickY as i32);
        assert_eq!(
            gamepad_axis::RIGHT_X as i32,
            gilrs::Axis::RightStickX as i32
        );
        assert_eq!(
            gamepad_axis::RIGHT_Y as i32,
            gilrs::Axis::RightStickY as i32
        );
    }

    #[test]
    fn test_default_deadzone_value() {
        assert!((DEFAULT_DEADZONE - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_gamepad_input_partial_update() {
        let mut input = GamepadInput::default();
        input.jump = true;
        assert!(input.jump);
        assert!(!input.shoot);
        assert!((input.move_x - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_gamepad_input_all_buttons() {
        let input = GamepadInput {
            move_x: 0.0,
            move_z: 0.0,
            jump: true,
            shoot: true,
            ..Default::default()
        };
        assert!(input.jump);
        assert!(input.shoot);
    }

    #[test]
    fn test_gamepad_input_analog_range() {
        let input = GamepadInput {
            move_x: 1.0,
            move_z: -1.0,
            jump: false,
            shoot: false,
            ..Default::default()
        };
        assert!((input.move_x - 1.0).abs() < 0.001);
        assert!((input.move_z - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_gamepad_input_negative_analog() {
        let input = GamepadInput {
            move_x: -0.7,
            move_z: 0.3,
            jump: false,
            shoot: false,
            ..Default::default()
        };
        assert!((input.move_x - (-0.7)).abs() < 0.001);
        assert!((input.move_z - 0.3).abs() < 0.001);
    }
}
