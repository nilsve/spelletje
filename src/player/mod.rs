use crate::physics::{Physics, PhysicsConfig, PositionUpdate};
use crate::input::Input;
use crate::obstacle::Aabb;
use crate::projectile::Projectile;
use crate::shooter::Shooter;

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
    pub gun_angle: f32,
    pub gun_pitch: f32,
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
            gun_angle: 0.0,
            gun_pitch: 0.0,
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
            gun_angle: 0.0,
            gun_pitch: 0.0,
        }
    }

    pub fn is_grounded(&self) -> bool {
        self.grounded
    }

    pub fn set_grounded(&mut self, grounded: bool) {
        self.grounded = grounded;
    }

    /// Returns the 3D position at the tip of the gun for rendering.
    pub fn gun_end(&self) -> (f32, f32, f32) {
        const GUN_LENGTH: f32 = 1.5;
        let cos_pitch = f32::cos(self.gun_pitch);
        let dir_x = f32::sin(self.gun_angle) * cos_pitch;
        let dir_y = f32::sin(self.gun_pitch);
        let dir_z = f32::cos(self.gun_angle) * cos_pitch;
        (
            self.x + dir_x * GUN_LENGTH,
            self.y + self.size / 2.0 + dir_y * GUN_LENGTH,
            self.z + dir_z * GUN_LENGTH,
        )
    }

    /// Returns the normalized shoot direction vector based on gun angle and pitch.
    pub fn shoot_direction(&self) -> (f32, f32, f32) {
        let cos_pitch = f32::cos(self.gun_pitch);
        let dir_x = f32::sin(self.gun_angle) * cos_pitch;
        let dir_y = f32::sin(self.gun_pitch);
        let dir_z = f32::cos(self.gun_angle) * cos_pitch;
        let len = (dir_x * dir_x + dir_y * dir_y + dir_z * dir_z).sqrt();
        (dir_x / len, dir_y / len, dir_z / len)
    }

    /// Creates a projectile fired from the gun tip in the current aim direction.
    pub fn fire(&self) -> Projectile {
        self.fire_with_direction(self.shoot_direction())
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
            self.set_grounded(false);
        }

        // Ground collision (before jump so grounded check works after gravity)
        if self.y <= self.config.friction_threshold && self.vel_y == 0.0 && !input.jump {
            self.set_grounded(true);
        }

        // Jump (Y axis)
        if input.jump && self.is_grounded() {
            physics.apply_jump(&mut self.vel_y, &pc);
            self.set_grounded(false);
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

        // Clamp speed
        physics.clamp_speed(&mut self.vel_x, &pc);
        physics.clamp_speed(&mut self.vel_z, &pc);

        self.update_position(dt);
    }
}

impl Shooter for Player {
    fn shoot_origin(&self) -> (f32, f32, f32) {
        self.gun_end()
    }

    fn projectile_speed(&self) -> f32 {
        15.0
    }

    fn damage(&self) -> f32 {
        10.0
    }
}

impl PositionUpdate for Player {
    fn update_position(&mut self, dt: f32) {
        self.x += self.vel_x * dt;
        self.y += self.vel_y * dt;
        self.z += self.vel_z * dt;
    }
}

impl Default for Player {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            size: 1.0,
            vel_x: 0.0,
            vel_y: 0.0,
            vel_z: 0.0,
            config: PlayerConfig::default(),
            gun_angle: 0.0,
            gun_pitch: 0.0,
            grounded: false,
        }
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
            shoot: false,
        }
    }

    fn grounded_input() -> Input {
        Input {
            left: false,
            right: false,
            jump: true,
            forward: false,
            backward: false,
            shoot: false,
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
        let input = Input {
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
        let jump_input = grounded_input();
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

    #[test]
    fn test_gun_angle_default_is_zero() {
        let player = Player::new();
        assert!((player.gun_angle - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_gun_end_points_forward_by_default() {
        let player = Player::new();
        let end = player.gun_end();
        // With gun_angle=0, should point in +Z direction
        assert!((end.0 - player.x).abs() < 0.01);
        assert!(end.2 > player.z);
        assert!((end.1 - (player.y + player.size / 2.0)).abs() < 0.01);
    }

    #[test]
    fn test_gun_end_points_right_at_pi_over_2() {
        let player = Player::new();
        let mut p = player;
        p.gun_angle = std::f32::consts::FRAC_PI_2;
        let end = p.gun_end();
        assert!(end.0 > p.x);
        assert!((end.2 - p.z).abs() < 0.01);
    }

    #[test]
    fn test_gun_end_points_left_at_minus_pi_over_2() {
        let player = Player::new();
        let mut p = player;
        p.gun_angle = -std::f32::consts::FRAC_PI_2;
        let end = p.gun_end();
        assert!(end.0 < p.x);
        assert!((end.2 - p.z).abs() < 0.01);
    }

    #[test]
    fn test_shoot_direction_normalized() {
        let player = Player::new();
        let dir = player.shoot_direction();
        let len = (dir.0 * dir.0 + dir.1 * dir.1 + dir.2 * dir.2).sqrt();
        assert!((len - 1.0).abs() < 0.01);
        // With default gun_pitch=0, dir_y is 0 (horizontal)
        assert!((dir.1 - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_shoot_direction_with_positive_pitch() {
        let mut player = Player::new();
        player.gun_pitch = 0.3;
        let dir = player.shoot_direction();
        assert!(dir.1 > 0.0);
    }

    #[test]
    fn test_shoot_direction_with_negative_pitch() {
        let mut player = Player::new();
        player.gun_pitch = -0.3;
        let dir = player.shoot_direction();
        assert!(dir.1 < 0.0);
    }

    #[test]
    fn test_shoot_direction_matches_gun_angle() {
        let player = Player::new();
        let mut p = player;
        p.gun_angle = std::f32::consts::FRAC_PI_4;
        let dir = p.shoot_direction();
        assert!(dir.0 > 0.0);
        assert!(dir.2 > 0.0);
    }

    #[test]
    fn test_fire_creates_projectile() {
        let player = Player::new();
        let projectile = player.fire();
        assert!(projectile.is_alive());
        assert!((projectile.damage - 10.0).abs() < 0.1);
        // Projectile starts at gun tip
        let gun_end = player.gun_end();
        assert!((projectile.x - gun_end.0).abs() < 0.01);
        assert!((projectile.y - gun_end.1).abs() < 0.01);
        assert!((projectile.z - gun_end.2).abs() < 0.01);
    }

    #[test]
    fn test_fire_projectile_has_velocity() {
        let player = Player::new();
        let projectile = player.fire();
        let total_vel = (projectile.vel_x * projectile.vel_x
            + projectile.vel_y * projectile.vel_y
            + projectile.vel_z * projectile.vel_z).sqrt();
        assert!((total_vel - 15.0).abs() < 0.1);
    }

    #[test]
    fn test_fire_projectile_aims_correctly() {
        let player = Player::new();
        let mut p = player;
        p.gun_angle = std::f32::consts::FRAC_PI_2; // point right
        let projectile = p.fire();
        assert!(projectile.vel_x > 10.0);
        assert!(projectile.vel_z.abs() < 1.0);
    }
}
