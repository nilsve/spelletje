/// Game states: Menu, Playing, GameOver.

use crate::input::PlayerInput;

/// Current state of the game.
#[derive(Clone, Debug, PartialEq)]
pub enum GameState {
    Menu,
    Playing,
    GameOver,
}

/// Game state manager.
pub struct GameStateManager {
    pub state: GameState,
    /// Number of players (1-4).
    pub player_count: usize,
    /// Index of the winner (if GameOver).
    pub winner: Option<usize>,
}

impl GameStateManager {
    pub fn new() -> Self {
        Self {
            state: GameState::Menu,
            player_count: 1,
            winner: None,
        }
    }

    /// Process input to transition states.
    pub fn update(&mut self, player_inputs: &[PlayerInput]) {
        match self.state {
            GameState::Menu => {
                // Any player pressing jump starts the game
                for input in player_inputs {
                    if input.jump {
                        self.state = GameState::Playing;
                        return;
                    }
                }
            }
            GameState::Playing => {
                // State transition checked externally (win condition)
            }
            GameState::GameOver => {
                // Any player pressing jump restarts
                for input in player_inputs {
                    if input.jump {
                        self.state = GameState::Menu;
                        self.winner = None;
                        return;
                    }
                }
            }
        }
    }

    /// Set game over with a winner.
    pub fn set_game_over(&mut self, winner_idx: usize) {
        self.state = GameState::GameOver;
        self.winner = Some(winner_idx);
    }

    /// Check if game is currently being played.
    pub fn is_playing(&self) -> bool {
        self.state == GameState::Playing
    }

    /// Check if game is in menu.
    pub fn is_menu(&self) -> bool {
        self.state == GameState::Menu
    }

    /// Check if game is over.
    pub fn is_game_over(&self) -> bool {
        self.state == GameState::GameOver
    }
}

impl Default for GameStateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state_is_menu() {
        let mgr = GameStateManager::new();
        assert_eq!(mgr.state, GameState::Menu);
    }

    #[test]
    fn test_menu_to_playing_on_jump() {
        let mut mgr = GameStateManager::new();
        let inputs = vec![PlayerInput { jump: true, ..PlayerInput::default() }];
        mgr.update(&inputs);
        assert_eq!(mgr.state, GameState::Playing);
    }

    #[test]
    fn test_menu_stays_on_no_jump() {
        let mut mgr = GameStateManager::new();
        let inputs = vec![PlayerInput::default()];
        mgr.update(&inputs);
        assert_eq!(mgr.state, GameState::Menu);
    }

    #[test]
    fn test_game_over_to_menu_on_jump() {
        let mut mgr = GameStateManager::new();
        mgr.state = GameState::GameOver;
        mgr.winner = Some(0);
        let inputs = vec![PlayerInput { jump: true, ..PlayerInput::default() }];
        mgr.update(&inputs);
        assert_eq!(mgr.state, GameState::Menu);
        assert_eq!(mgr.winner, None);
    }

    #[test]
    fn test_game_over_stays_on_no_jump() {
        let mut mgr = GameStateManager::new();
        mgr.state = GameState::GameOver;
        let inputs = vec![PlayerInput::default()];
        mgr.update(&inputs);
        assert_eq!(mgr.state, GameState::GameOver);
    }

    #[test]
    fn test_set_game_over() {
        let mut mgr = GameStateManager::new();
        mgr.set_game_over(2);
        assert_eq!(mgr.state, GameState::GameOver);
        assert_eq!(mgr.winner, Some(2));
    }

    #[test]
    fn test_is_playing() {
        let mut mgr = GameStateManager::new();
        assert!(!mgr.is_playing());
        mgr.state = GameState::Playing;
        assert!(mgr.is_playing());
    }

    #[test]
    fn test_is_menu() {
        let mgr = GameStateManager::new();
        assert!(mgr.is_menu());
    }

    #[test]
    fn test_is_game_over() {
        let mut mgr = GameStateManager::new();
        assert!(!mgr.is_game_over());
        mgr.set_game_over(0);
        assert!(mgr.is_game_over());
    }

    #[test]
    fn test_default() {
        let mgr = GameStateManager::default();
        assert_eq!(mgr.state, GameState::Menu);
    }

    #[test]
    fn test_playing_ignores_input() {
        let mut mgr = GameStateManager::new();
        mgr.state = GameState::Playing;
        let inputs = vec![PlayerInput { jump: true, ..PlayerInput::default() }];
        mgr.update(&inputs);
        assert_eq!(mgr.state, GameState::Playing);
    }

    #[test]
    fn test_menu_multiple_players() {
        let mut mgr = GameStateManager::new();
        mgr.player_count = 2;
        let inputs = vec![
            PlayerInput::default(),
            PlayerInput { jump: true, ..PlayerInput::default() },
        ];
        mgr.update(&inputs);
        assert_eq!(mgr.state, GameState::Playing);
    }
}
