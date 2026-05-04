use crate::physics::{PhysicsConfig, PhysicsEntity, PhysicsImpl, PhysicsData};
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
    pub physics_data: PhysicsData,
    pub grounded: bool,
    pub config: PlayerConfig,
    pub gun_angle: f32,
    pub gun_pitch: f32,
}

impl Aabb for Player {
    fn min_x(&self) -> f32 {
        self.physics_data.x - self.physics_data.size / 2.0
    }
    fn max_x(&self) -> f32 {
        self.physics_data.x + self.physics_data.size / 2.0
    }
    fn min_y(&self) -> f32 {
        self.physics_data.y
    }
    fn max_y(&self) -> f32 {
        self.physics_data.y + self.physics_data.size
    }
    fn min_z(&self) -> f32 {
        self.physics_data.z - self.physics_data.size / 2.0
    }
    fn max_z(&self) -> f32 {
        self.physics_data.z + self.physics_data.size / 2.0
    }
}

impl Player {
    pub fn new() -> Self {
        Self {
            physics_data: PhysicsData {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                vel_x: 0.0,
                vel_y: 0.0,
                vel_z: 0.0,
                size: 1.0,
            },
            grounded: true,
            config: PlayerConfig::default(),
            gun_angle: 0.0,
            gun_pitch: 0.0,
        }
    }

    pub fn with_config(config: PlayerConfig) -> Self {
        Self {
            physics_data: PhysicsData {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                vel_x: 0.0,
                vel_y: 0.0,
                vel_z: 0.0,
                size: 1.0,
            },
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

    pub fn pos(&self) -> (f32, f32, f32) {
        (self.physics_data.x, self.physics_data.y, self.physics_data.z)
    }

    pub fn size(&self) -> f32 {
        self.physics_data.size
    }

    /// Returns the 3D position at the tip of the gun for rendering.
    pub fn gun_end(&self) -> (f32, f32, f32) {
        const GUN_LENGTH: f32 = 1.5;
        let cos_pitch = f32::cos(self.gun_pitch);
        let dir_x = f32::sin(self.gun_angle) * cos_pitch;
        let dir_y = f32::sin(self.gun_pitch);
        let dir_z = f32::cos(self.gun_angle) * cos_pitch;
        (
            self.physics_data.x + dir_x * GUN_LENGTH,
            self.physics_data.y + self.physics_data.size / 2.0 + dir_y * GUN_LENGTH,
            self.physics_data.z + dir_z * GUN_LENGTH,
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

    pub fn update(&mut self, input: &Input, physics: &PhysicsImpl, dt: f32) {

        let mut input_x = 0.0;
        if input.left {
            input_x -= 1.0;
        }
        if input.right {
            input_x += 1.0;
        }

        if input_x != 0.0 {
            physics.apply_acceleration(&mut self.physics_data.vel_x, input_x, dt);
        } else {
            physics.apply_friction(&mut self.physics_data.vel_x, dt);
        }

        physics.apply_gravity(&mut self.physics_data.vel_y, dt);

        if self.physics_data.vel_y > 0.0 {
            self.set_grounded(false);
        }

        if self.physics_data.y <= self.config.friction_threshold && self.physics_data.vel_y == 0.0 && !input.jump {
            self.set_grounded(true);
        }

        if input.jump && self.is_grounded() {
            physics.apply_jump(&mut self.physics_data.vel_y);
            self.set_grounded(false);
        }

        let mut input_z = 0.0;
        if input.backward {
            input_z -= 1.0;
        }

        if input_z != 0.0 {
            physics.apply_acceleration(&mut self.physics_data.vel_z, input_z, dt);
        } else {
            physics.apply_friction(&mut self.physics_data.vel_z, dt);
        }

        physics.clamp_speed(&mut self.physics_data.vel_x);
        physics.clamp_speed(&mut self.physics_data.vel_z);

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

impl PhysicsEntity for Player {
    fn physics_data(&self) -> &PhysicsData { &self.physics_data }
    fn physics_data_mut(&mut self) -> &mut PhysicsData { &mut self.physics_data }
    fn update_position(&mut self, dt: f32) {
        self.physics_data.x += self.physics_data.vel_x * dt;
        self.physics_data.y += self.physics_data.vel_y * dt;
        self.physics_data.z += self.physics_data.vel_z * dt;
    }
}

impl Default for Player {
    fn default() -> Self {
        Self {
            physics_data: PhysicsData {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                vel_x: 0.0,
                vel_y: 0.0,
                vel_z: 0.0,
                size: 1.0,
            },
            grounded: false,
            config: PlayerConfig::default(),
            gun_angle: 0.0,
            gun_pitch: 0.0,
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
        assert!((player.physics_data.x - 0.0).abs() < 0.01);
        assert!((player.physics_data.y - 0.0).abs() < 0.01);
        assert!((player.physics_data.z - 0.0).abs() < 0.01);
        assert!((player.physics_data.vel_x - 0.0).abs() < 0.01);
        assert!((player.physics_data.vel_y - 0.0).abs() < 0.01);
        assert!((player.physics_data.vel_z - 0.0).abs() < 0.01);
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

        assert!(player.physics_data.vel_x > 0.0);
    }

    #[test]
    fn test_friction_stops_player_when_no_input() {
        let mut player = Player::new();
        player.physics_data.vel_x = 5.0;
        let input = default_input();
        let physics = PhysicsImpl::new();

        // Update many frames until velocity should reach zero
        for _ in 0..100 {
            player.update(&input, &physics, 0.016);
        }

        assert!((player.physics_data.vel_x - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_jump_sets_vertical_velocity() {
        let mut player = Player::new();
        assert!(player.is_grounded());
        let physics = PhysicsImpl::new();

        player.update(&grounded_input(), &physics, 0.016);

        assert!((player.physics_data.vel_y - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_cannot_jump_when_airborne() {
        let mut player = Player::new();
        player.physics_data.y = 1.0;
        player.grounded = false;
        assert!(!player.is_grounded());

        let physics = PhysicsImpl::new();
        let initial_vel_y = player.physics_data.vel_y;

        player.update(&grounded_input(), &physics, 0.016);

        // Gravity still applies, so vel_y changes from initial
        assert!((player.physics_data.vel_y - (-0.32)).abs() < 0.1);
        assert!((player.physics_data.vel_y - initial_vel_y).abs() > 0.1);
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

        assert!((player.physics_data.vel_x - 5.0).abs() < 0.1);
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

        assert!(player.physics_data.vel_x < 0.0);
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

        assert!(player.physics_data.vel_z < 0.0);
    }

    #[test]
    fn test_position_updates_with_velocity() {
        let mut player = Player::new();
        player.physics_data.vel_x = 5.0;
        let input = default_input();
        let physics = PhysicsImpl::new();
        let dt = 0.016;

        player.update(&input, &physics, dt);

        assert!(player.physics_data.x > 0.0);
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
        assert!(player.physics_data.y >= -0.1);
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

        assert!((player.physics_data.vel_y - 20.0).abs() < 0.1);
    }

    #[test]
    fn test_velocity_decreases_to_zero_with_friction() {
        let mut player = Player::new();
        player.physics_data.vel_x = 10.0;
        let input = default_input();
        let physics = PhysicsImpl::new();

        player.update(&input, &physics, 0.016);

        assert!(player.physics_data.vel_x < 10.0);
        assert!(player.physics_data.vel_x > 0.0);
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
        assert!((end.0 - player.physics_data.x).abs() < 0.01);
        assert!(end.2 > player.physics_data.z);
        assert!((end.1 - (player.physics_data.y + player.physics_data.size / 2.0)).abs() < 0.01);
    }

    #[test]
    fn test_gun_end_points_right_at_pi_over_2() {
        let player = Player::new();
        let mut p = player;
        p.gun_angle = std::f32::consts::FRAC_PI_2;
        let end = p.gun_end();
        assert!(end.0 > p.physics_data.x);
        assert!((end.2 - p.physics_data.z).abs() < 0.01);
    }

    #[test]
    fn test_gun_end_points_left_at_minus_pi_over_2() {
        let player = Player::new();
        let mut p = player;
        p.gun_angle = -std::f32::consts::FRAC_PI_2;
        let end = p.gun_end();
        assert!(end.0 < p.physics_data.x);
        assert!((end.2 - p.physics_data.z).abs() < 0.01);
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
        assert!((projectile.physics_data.x - gun_end.0).abs() < 0.01);
        assert!((projectile.physics_data.y - gun_end.1).abs() < 0.01);
        assert!((projectile.physics_data.z - gun_end.2).abs() < 0.01);
    }

    #[test]
    fn test_fire_projectile_has_velocity() {
        let player = Player::new();
        let projectile = player.fire();
        let total_vel = (projectile.physics_data.vel_x * projectile.physics_data.vel_x
            + projectile.physics_data.vel_y * projectile.physics_data.vel_y
            + projectile.physics_data.vel_z * projectile.physics_data.vel_z).sqrt();
        assert!((total_vel - 15.0).abs() < 0.1);
    }

    #[test]
    fn test_fire_projectile_aims_correctly() {
        let player = Player::new();
        let mut p = player;
        p.gun_angle = std::f32::consts::FRAC_PI_2; // point right
        let projectile = p.fire();
        assert!(projectile.physics_data.vel_x > 10.0);
        assert!(projectile.physics_data.vel_z.abs() < 1.0);
    }
}
