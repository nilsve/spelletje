/// King of the Hill game mode: timer, scoring, and hill teleportation.

use crate::obstacle::Obstacle;

/// Default score needed to win a King of the Hill round.
pub const WINNING_SCORE: u32 = 10;

/// Default time (in seconds) a player must stay on the hill to earn a point.
pub const POINT_TIME: f32 = 1.0;

/// Hill state tracked by the world.
#[derive(Debug, Clone)]
pub struct Hill {
    /// Current position of the hill platform.
    pub x: f32,
    pub y: f32,
    pub z: f32,
    /// Current dimensions of the hill.
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    /// How long the current player has been standing on the hill (seconds).
    pub timer: f32,
    /// Which player index currently has the hill (None if no one is on it).
    pub holder: Option<usize>,
    /// Scores per player index.
    pub scores: Vec<u32>,
    /// Has the round ended?
    pub finished: bool,
    /// Index of the winner (if finished).
    pub winner: Option<usize>,
    /// Time between hill teleports (seconds).
    pub teleport_interval: f32,
    /// Timer for next teleport.
    pub teleport_timer: f32,
}

impl Hill {
    /// Create a new Hill at the default arena position.
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.5,
            z: 0.0,
            width: 5.0,
            height: 0.5,
            depth: 5.0,
            timer: 0.0,
            holder: None,
            scores: Vec::new(),
            finished: false,
            winner: None,
            teleport_interval: 15.0,
            teleport_timer: 0.0,
        }
    }

    /// Initialize scores for the given number of players.
    pub fn init_scores(&mut self, player_count: usize) {
        self.scores = vec![0u32; player_count];
    }

    /// Ensure the scores vector has at least `min_players` entries.
    pub fn ensure_scores(&mut self, min_players: usize) {
        while self.scores.len() < min_players {
            self.scores.push(0);
        }
    }

    /// Check if any player has reached the winning score.
    pub fn check_win(&self) -> Option<usize> {
        for (i, &score) in self.scores.iter().enumerate() {
            if score >= WINNING_SCORE {
                return Some(i);
            }
        }
        None
    }

    /// Register that the given player index is on the hill.
    /// Returns true if this player just earned a point.
    pub fn register_holder(&mut self, player_idx: usize, dt: f32) -> bool {
        self.ensure_scores(player_idx + 1);
        if self.holder == Some(player_idx) {
            self.timer += dt;
            if self.timer >= POINT_TIME {
                self.scores[player_idx] += 1;
                self.timer = 0.0;
                return true;
            }
            false
        } else {
            self.holder = Some(player_idx);
            self.timer = 0.0;
            false
        }
    }

    /// Clear the current holder (e.g., when hill teleports).
    pub fn clear_holder(&mut self) {
        self.holder = None;
        self.timer = 0.0;
    }

    /// Teleport the hill to a new position with random dimensions.
    pub fn teleport(&mut self, new_x: f32, new_y: f32, new_z: f32) {
        self.x = new_x;
        self.y = new_y;
        self.z = new_z;
        // Randomize dimensions slightly for variety
        self.width = 3.0 + (new_x.abs() * 0.5).min(4.0);
        self.depth = 3.0 + (new_z.abs() * 0.5).min(4.0);
        self.height = 0.5;
        self.clear_holder();
    }

    /// Create the hill as an Obstacle.
    pub fn to_obstacle(&self) -> Obstacle {
        Obstacle::platform(self.x, self.y, self.z, self.width, self.height, self.depth)
    }

    /// Update the teleport timer. Returns true if the hill should teleport now.
    pub fn update_teleport(&mut self, dt: f32) -> bool {
        self.teleport_timer += dt;
        if self.teleport_timer >= self.teleport_interval {
            self.teleport_timer = 0.0;
            true
        } else {
            false
        }
    }

    /// Reset the teleport timer (e.g., after a round ends).
    pub fn reset_teleport_timer(&mut self) {
        self.teleport_timer = 0.0;
    }
}

impl Default for Hill {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_hill_defaults() {
        let hill = Hill::new();
        assert!((hill.x - 0.0).abs() < 0.01);
        assert!((hill.y - 0.5).abs() < 0.01);
        assert!((hill.z - 0.0).abs() < 0.01);
        assert!((hill.width - 5.0).abs() < 0.01);
        assert!((hill.height - 0.5).abs() < 0.01);
        assert!((hill.depth - 5.0).abs() < 0.01);
        assert_eq!(hill.timer, 0.0);
        assert_eq!(hill.holder, None);
        assert!(hill.scores.is_empty());
        assert!(!hill.finished);
        assert_eq!(hill.winner, None);
    }

    #[test]
    fn test_init_scores() {
        let mut hill = Hill::new();
        hill.init_scores(3);
        assert_eq!(hill.scores.len(), 3);
        assert_eq!(hill.scores, vec![0, 0, 0]);
    }

    #[test]
    fn test_ensure_scores_grows() {
        let mut hill = Hill::new();
        hill.init_scores(2);
        hill.ensure_scores(4);
        assert_eq!(hill.scores.len(), 4);
        assert_eq!(hill.scores, vec![0, 0, 0, 0]);
    }

    #[test]
    fn test_ensure_scores_no_change_when_sufficient() {
        let mut hill = Hill::new();
        hill.init_scores(3);
        hill.ensure_scores(2);
        assert_eq!(hill.scores.len(), 3);
    }

    #[test]
    fn test_no_win_with_zero_scores() {
        let hill = Hill::new();
        assert_eq!(hill.check_win(), None);
    }

    #[test]
    fn test_no_win_below_winning_score() {
        let mut hill = Hill::new();
        hill.init_scores(2);
        hill.scores[0] = WINNING_SCORE - 1;
        assert_eq!(hill.check_win(), None);
    }

    #[test]
    fn test_win_at_winning_score() {
        let mut hill = Hill::new();
        hill.init_scores(2);
        hill.scores[0] = WINNING_SCORE;
        assert_eq!(hill.check_win(), Some(0));
    }

    #[test]
    fn test_win_above_winning_score() {
        let mut hill = Hill::new();
        hill.init_scores(2);
        hill.scores[1] = WINNING_SCORE + 5;
        assert_eq!(hill.check_win(), Some(1));
    }

    #[test]
    fn test_register_holder_first_time() {
        let mut hill = Hill::new();
        hill.init_scores(2);
        // First time — no point earned, timer starts
        let earned = hill.register_holder(0, 0.016);
        assert!(!earned);
        assert_eq!(hill.holder, Some(0));
    }

    #[test]
    fn test_register_holder_same_player_accumulates_timer() {
        let mut hill = Hill::new();
        hill.init_scores(2);
        hill.holder = Some(0);

        // Simulate multiple frames (need 63 calls for 1.0 seconds: 63 * 0.016 = 1.008)
        for _ in 0..63 {
            hill.register_holder(0, 0.016);
        }

        assert_eq!(hill.scores[0], 1);
    }

    #[test]
    fn test_register_holder_switches_player() {
        let mut hill = Hill::new();
        hill.init_scores(2);
        hill.holder = Some(0);
        hill.timer = 1.0;

        let earned = hill.register_holder(1, 0.016);
        assert!(!earned);
        assert_eq!(hill.holder, Some(1));
        assert!((hill.timer - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_clear_holder_resets_state() {
        let mut hill = Hill::new();
        hill.init_scores(2);
        hill.holder = Some(0);
        hill.timer = 2.0;

        hill.clear_holder();
        assert_eq!(hill.holder, None);
        assert!((hill.timer - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_teleport_changes_position() {
        let mut hill = Hill::new();
        hill.teleport(10.0, 2.0, -5.0);
        assert!((hill.x - 10.0).abs() < 0.01);
        assert!((hill.y - 2.0).abs() < 0.01);
        assert!((hill.z - (-5.0)).abs() < 0.01);
    }

    #[test]
    fn test_teleport_clears_holder() {
        let mut hill = Hill::new();
        hill.init_scores(2);
        hill.holder = Some(0);
        hill.timer = 1.5;

        hill.teleport(5.0, 1.0, 5.0);
        assert_eq!(hill.holder, None);
        assert!((hill.timer - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_teleport_changes_dimensions() {
        let mut hill = Hill::new();
        let orig_width = hill.width;
        let orig_depth = hill.depth;
        hill.teleport(10.0, 1.0, 10.0);
        // Dimensions should change based on new position
        assert!((hill.width - orig_width).abs() > 0.0 || (hill.depth - orig_depth).abs() > 0.0);
    }

    #[test]
    fn test_to_obstacle() {
        let mut hill = Hill::new();
        hill.x = 5.0;
        hill.y = 2.0;
        hill.z = -3.0;
        hill.width = 4.0;
        hill.height = 0.5;
        hill.depth = 4.0;

        let obs = hill.to_obstacle();
        assert!((obs.x - 5.0).abs() < 0.01);
        assert!((obs.y - 2.0).abs() < 0.01);
        assert!((obs.z - (-3.0)).abs() < 0.01);
        assert!((obs.width - 4.0).abs() < 0.01);
        assert!((obs.height - 0.5).abs() < 0.01);
        assert!((obs.depth - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_default_hill() {
        let hill = Hill::default();
        assert_eq!(hill.holder, None);
        assert!(!hill.finished);
    }

    #[test]
    fn test_point_time_constant() {
        assert_eq!(POINT_TIME, 1.0);
    }

    #[test]
    fn test_winning_score_constant() {
        assert_eq!(WINNING_SCORE, 10);
    }
}
