/// Abstracted game input, decoupled from macroquad for testability.

#[derive(Clone, Debug, Default)]
pub struct Input {
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub forward: bool,
    pub backward: bool,
}

/// Reads hardware input and produces an Input state.
/// Implementations can be mocked in tests.
pub trait InputSource {
    fn read(&self) -> Input;
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
            jump: is_key_pressed(KeyCode::W),
            forward: is_key_down(KeyCode::S),
            backward: is_key_down(KeyCode::Space),
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
    }

    #[test]
    fn test_input_clone() {
        let input = Input {
            left: true,
            right: false,
            jump: true,
            forward: false,
            backward: false,
        };
        let cloned = input.clone();
        assert!(cloned.left);
        assert!(!cloned.right);
        assert!(cloned.jump);
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
}
