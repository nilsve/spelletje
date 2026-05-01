use crate::physics::{Physics, PhysicsConfig};
use crate::input::Input;
use crate::obstacle::Aabb;

#[derive(Clone, Debug)]
pub struct PlayerConfig {
    pub speed: f32,
    pub acceleration: f32,
    pub friction: f32,
    pub gravity: f32,
    pub jump_force: f32,
    pub friction_threshold: f32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            speed: 5.0,
            acceleration: 20.0,
            friction: 5.0,
            gravity: 20.0,
            jump_force: 10.0,
            friction_threshold: 0.01,
        }
    }
}

impl PlayerConfig {
    pub fn to_physics_config(&self) -> PhysicsConfig {
        PhysicsConfig {
            gravity: self.gravity,
            jump_force: self.jump_force,
            max_speed: self.speed,
            acceleration: self.acceleration,
            friction: self.friction,
            friction_threshold: self.friction_threshold,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub size: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub vel_z: f32,
    pub grounded: bool,
    pub config: PlayerConfig,
}

impl Aabb for Player {
    fn min_x(&self) -> f32 {
        self.x - self.size / 2.0
    }
    fn max_x(&self) -> f32 {
        self.x + self.size / 2.0
    }
    fn min_y(&self) -> f32 {
        self.y
    }
    fn max_y(&self) -> f32 {
        self.y + self.size
    }
    fn min_z(&self) -> f32 {
        self.z - self.size / 2.0
    }
    fn max_z(&self) -> f32 {
        self.z + self.size / 2.0
    }
}

impl Player {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            size: 1.0,
            vel_x: 0.0,
            vel_y: 0.0,
            vel_z: 0.0,
            grounded: true,
            config: PlayerConfig::default(),
        }
    }

    pub fn with_config(config: PlayerConfig) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            size: 1.0,
            vel_x: 0.0,
            vel_y: 0.0,
            vel_z: 0.0,
            grounded: true,
            config,
        }
    }

    pub fn is_grounded(&self) -> bool {
        self.grounded
    }

    pub fn set_grounded(&mut self, grounded: bool) {
        self.grounded = grounded;
    }

    pub fn update(&mut self, input: &Input, physics: &dyn Physics, dt: f32) {
        let config = &self.config;
        let pc = config.to_physics_config();

        // Horizontal input (X axis)
        let mut input_x = 0.0;
        if input.left {
            input_x -= 1.0;
        }
        if input.right {
            input_x += 1.0;
        }

        if input_x != 0.0 {
            physics.apply_acceleration(&mut self.vel_x, input_x, dt, &pc);
        } else {
            physics.apply_friction(&mut self.vel_x, dt, &pc);
        }

        // Gravity
        physics.apply_gravity(&mut self.vel_y, dt, &pc);

        // If moving upward, not grounded
        if self.vel_y > 0.0 {
            self.grounded = false;
        }

        // Ground collision (before jump so grounded check works after gravity)
        physics.resolve_ground_collision(&mut self.y, &mut self.vel_y, &pc);
        if self.y <= self.config.friction_threshold && self.vel_y == 0.0 && !input.jump {
            self.grounded = true;
        }

        // Jump (Y axis)
        if input.jump && self.grounded {
            physics.apply_jump(&mut self.vel_y, &pc);
            self.grounded = false;
        }

        // Depth input (Z axis)
        let mut input_z = 0.0;
        if input.backward {
            input_z -= 1.0;
        }

        if input_z != 0.0 {
            physics.apply_acceleration(&mut self.vel_z, input_z, dt, &pc);
        } else {
            physics.apply_friction(&mut self.vel_z, dt, &pc);
        }

        // Update positions
        self.x += self.vel_x * dt;
        self.y += self.vel_y * dt;
        self.z += self.vel_z * dt;

        // Clamp speed
        physics.clamp_speed(&mut self.vel_x, &pc);
        physics.clamp_speed(&mut self.vel_z, &pc);
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::PhysicsImpl;

    fn default_input() -> Input {
        Input {
            left: false,
            right: false,
            jump: false,
            forward: false,
            backward: false,
        }
    }

    fn grounded_input() -> Input {
        Input {
            left: false,
            right: false,
            jump: true,
            forward: false,
            backward: false,
        }
    }

    #[test]
    fn test_new_player_starts_at_origin() {
        let player = Player::new();
        assert!((player.x - 0.0).abs() < 0.01);
        assert!((player.y - 0.0).abs() < 0.01);
        assert!((player.z - 0.0).abs() < 0.01);
        assert!((player.vel_x - 0.0).abs() < 0.01);
        assert!((player.vel_y - 0.0).abs() < 0.01);
        assert!((player.vel_z - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_player_is_grounded_when_at_y_zero() {
        let mut player = Player::new();
        assert!(player.is_grounded());
    }

    #[test]
    fn test_player_not_grounded_when_above_ground() {
        let mut player = Player::new();
        player.grounded = false;
        assert!(!player.is_grounded());
    }

    #[test]
    fn test_horizontal_acceleration() {
        let mut player = Player::new();
        let input = Input {
            right: true,
            ..default_input()
        };
        let physics = PhysicsImpl::new();
        let dt = 0.016;

        player.update(&input, &physics, dt);

        assert!(player.vel_x > 0.0);
    }

    #[test]
    fn test_friction_stops_player_when_no_input() {
        let mut player = Player::new();
        player.vel_x = 5.0;
        let input = default_input();
        let physics = PhysicsImpl::new();

        // Update many frames until velocity should reach zero
        for _ in 0..100 {
            player.update(&input, &physics, 0.016);
        }

        assert!((player.vel_x - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_jump_sets_vertical_velocity() {
        let mut player = Player::new();
        assert!(player.is_grounded());
        let physics = PhysicsImpl::new();

        player.update(&grounded_input(), &physics, 0.016);

        assert!((player.vel_y - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_cannot_jump_when_airborne() {
        let mut player = Player::new();
        player.y = 1.0;
        player.grounded = false;
        assert!(!player.is_grounded());

        let physics = PhysicsImpl::new();
        let initial_vel_y = player.vel_y;

        player.update(&grounded_input(), &physics, 0.016);

        // Gravity still applies, so vel_y changes from initial
        assert!((player.vel_y - (-0.32)).abs() < 0.1);
        assert!((player.vel_y - initial_vel_y).abs() > 0.1);
    }

    #[test]
    fn test_max_speed_is_enforced() {
        let mut player = Player::new();
        let mut input = Input {
            right: true,
            ..default_input()
        };
        let physics = PhysicsImpl::new();

        // Accelerate for many frames
        for _ in 0..100 {
            player.update(&input, &physics, 0.016);
        }

        assert!((player.vel_x - 5.0).abs() < 0.1);
    }

    #[test]
    fn test_negative_horizontal_movement() {
        let mut player = Player::new();
        let input = Input {
            left: true,
            ..default_input()
        };
        let physics = PhysicsImpl::new();
        let dt = 0.016;

        player.update(&input, &physics, dt);

        assert!(player.vel_x < 0.0);
    }

    #[test]
    fn test_z_axis_movement() {
        let mut player = Player::new();
        let input = Input {
            backward: true,
            ..default_input()
        };
        let physics = PhysicsImpl::new();
        let dt = 0.016;

        player.update(&input, &physics, dt);

        assert!(player.vel_z < 0.0);
    }

    #[test]
    fn test_position_updates_with_velocity() {
        let mut player = Player::new();
        player.vel_x = 5.0;
        let input = default_input();
        let physics = PhysicsImpl::new();
        let dt = 0.016;

        player.update(&input, &physics, dt);

        assert!(player.x > 0.0);
    }

    #[test]
    fn test_gravity_affects_vertical_position() {
        let mut player = Player::new();
        let mut jump_input = grounded_input();
        let physics = PhysicsImpl::new();

        // Jump
        player.update(&jump_input, &physics, 0.016);

        // Now no jump input
        let no_jump_input = default_input();
        for _ in 0..50 {
            player.update(&no_jump_input, &physics, 0.016);
        }

        // Player should have gone up and come back down
        // After enough frames, should be back near ground
        assert!(player.y >= -0.1);
    }

    #[test]
    fn test_custom_config_is_applied() {
        let config = PlayerConfig {
            jump_force: 20.0,
            speed: 10.0,
            ..PlayerConfig::default()
        };
        let mut player = Player::with_config(config);
        let physics = PhysicsImpl::new();

        player.update(&grounded_input(), &physics, 0.016);

        assert!((player.vel_y - 20.0).abs() < 0.1);
    }

    #[test]
    fn test_velocity_decreases_to_zero_with_friction() {
        let mut player = Player::new();
        player.vel_x = 10.0;
        let input = default_input();
        let physics = PhysicsImpl::new();

        player.update(&input, &physics, 0.016);

        assert!(player.vel_x < 10.0);
        assert!(player.vel_x > 0.0);
    }
}
