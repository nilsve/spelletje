use crate::input::PlayerInput;
use crate::obstacle::Aabb;
use crate::physics::{EntityPhysicsData, Physics, PhysicsEntity};
use crate::powerup::PowerUpKind;
use crate::projectile::Projectile;
use crate::shooter::Shooter;

#[derive(Clone, Debug)]
pub struct Player {
    pub jump_force: f32,
    physics_data: EntityPhysicsData,
    pub angle: f32,
    pub gun_pitch: f32,
    pub is_dead: bool,
    /// Currently active power-up effect.
    pub active_powerup: Option<PowerUpKind>,
    /// Remaining time on the active power-up (seconds).
    pub powerup_timer: f32,
    /// For DoubleJump: has the mid-air jump been used this jump arc?
    pub double_jump_used: bool,
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
    pub fn is_grounded(&self) -> bool {
        self.physics_data().is_grounded
    }

    pub fn is_player_dead(&self) -> bool {
        self.is_dead
    }

    pub fn respawn(&mut self) {
        self.is_dead = false;
        self.physics_data = EntityPhysicsData {
            size: 1.0,
            ..EntityPhysicsData::default()
        };
        self.angle = 0.0;
        self.gun_pitch = 0.0;
        self.active_powerup = None;
        self.powerup_timer = 0.0;
        self.double_jump_used = false;
    }

    /// Returns the 3D position at the tip of the gun for rendering.
    pub fn gun_end(&self) -> (f32, f32, f32) {
        const GUN_LENGTH: f32 = 1.5;
        let dir_x = f32::sin(self.angle);
        let dir_y = f32::sin(self.gun_pitch);
        let dir_z = f32::cos(self.angle);
        (
            self.physics_data.x + dir_x * GUN_LENGTH,
            self.physics_data.y + self.physics_data.size / 2.0 + dir_y * GUN_LENGTH,
            self.physics_data.z + dir_z * GUN_LENGTH,
        )
    }

    /// Returns the normalized shoot direction vector based on gun angle and pitch.
    pub fn shoot_direction(&self) -> (f32, f32, f32) {
        let cos_pitch = f32::cos(self.gun_pitch);
        let dir_x = f32::sin(self.angle) * cos_pitch;
        let dir_y = f32::sin(self.gun_pitch);
        let dir_z = f32::cos(self.angle) * cos_pitch;
        let len = (dir_x * dir_x + dir_y * dir_y + dir_z * dir_z).sqrt();
        (dir_x / len, dir_y / len, dir_z / len)
    }

    /// Creates a projectile fired from the gun tip in the current aim direction.
    pub fn fire(&self) -> Projectile {
        self.fire_at_direction(self.shoot_direction())
    }

    pub fn update(&mut self, input: &PlayerInput, physics: &Physics, dt: f32) {
        // Update power-up timer
        if self.active_powerup.is_some() {
            self.powerup_timer -= dt;
            if self.powerup_timer <= 0.0 {
                let was_double_jump = self.active_powerup == Some(PowerUpKind::DoubleJump);
                self.active_powerup = None;
                self.powerup_timer = 0.0;
                if was_double_jump {
                    self.double_jump_used = false;
                }
            }
        }

        let speed_mult = if self.active_powerup == Some(PowerUpKind::SpeedBoost) { 2.0 } else { 1.0 };

        if input.move_x != 0.0 {
            physics.apply_acceleration_x(&mut self.physics_data, input.move_x * speed_mult, dt);
        } else {
            physics.apply_friction(&mut self.physics_data.vel_x, dt);
        }

        if input.move_z != 0.0 {
            physics.apply_acceleration_z(&mut self.physics_data, input.move_z * speed_mult, dt);
        } else {
            physics.apply_friction(&mut self.physics_data.vel_z, dt);
        }

        physics.apply_gravity(&mut self.physics_data.vel_y, dt);

        let can_double_jump = self.active_powerup == Some(PowerUpKind::DoubleJump) && !self.double_jump_used;

        if input.jump && (self.is_grounded() || can_double_jump) {
            self.physics_data.vel_y = self.jump_force;
            if can_double_jump {
                self.double_jump_used = true;
            }
        }

        // Reset double jump flag when grounded
        if self.is_grounded() {
            self.double_jump_used = false;
        }

        physics.clamp_speed(&mut self.physics_data.vel_x);
        physics.clamp_speed(&mut self.physics_data.vel_z);
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
    fn physics_data(&self) -> &EntityPhysicsData {
        &self.physics_data
    }
    fn physics_data_mut(&mut self) -> &mut EntityPhysicsData {
        &mut self.physics_data
    }
}

impl Default for Player {
    fn default() -> Self {
        Self {
            physics_data: EntityPhysicsData {
                size: 1.0,
                ..EntityPhysicsData::default()
            },
            jump_force: 10.0,
            angle: 0.0,
            gun_pitch: 0.0,
            is_dead: false,
            active_powerup: None,
            powerup_timer: 0.0,
            double_jump_used: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::Physics;

    fn default_input() -> PlayerInput {
        PlayerInput::default()
    }

    fn grounded_input() -> PlayerInput {
        PlayerInput {
            jump: true,
            ..PlayerInput::default()
        }
    }

    #[test]
    fn test_new_player_starts_at_origin() {
        let player = Player::default();
        assert!((player.physics_data.x - 0.0).abs() < 0.01);
        assert!((player.physics_data.y - 0.0).abs() < 0.01);
        assert!((player.physics_data.z - 0.0).abs() < 0.01);
        assert!((player.physics_data.vel_x - 0.0).abs() < 0.01);
        assert!((player.physics_data.vel_y - 0.0).abs() < 0.01);
        assert!((player.physics_data.vel_z - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_player_is_grounded_when_at_y_zero() {
        let player = Player::default();
        assert!(player.is_grounded());
    }

    #[test]
    fn test_player_not_grounded_when_above_ground() {
        let mut player = Player::default();
        player.physics_data.is_grounded = false;
        assert!(!player.is_grounded());
    }

    #[test]
    fn test_horizontal_acceleration() {
        let mut player = Player::default();
        let input = PlayerInput {
            move_x: 1.0,
            ..default_input()
        };
        let physics = Physics::new();
        let dt = 0.016;

        player.update(&input, &physics, dt);

        assert!(player.physics_data.vel_x > 0.0);
    }

    #[test]
    fn test_friction_stops_player_when_no_input() {
        let mut player = Player::default();
        player.physics_data.vel_x = 5.0;
        let input = default_input();
        let physics = Physics::new();

        // Update many frames until velocity should reach zero
        for _ in 0..100 {
            player.update(&input, &physics, 0.016);
        }

        assert!((player.physics_data.vel_x - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_jump_sets_vertical_velocity() {
        let mut player = Player::default();
        assert!(player.is_grounded());
        let physics = Physics::new();

        player.update(&grounded_input(), &physics, 0.016);

        assert!((player.physics_data.vel_y - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_cannot_jump_when_airborne() {
        let mut player = Player::default();
        player.physics_data.y = 1.0;
        player.physics_data.is_grounded = false;
        assert!(!player.is_grounded());

        let physics = Physics::new();
        let initial_vel_y = player.physics_data.vel_y;

        player.update(&grounded_input(), &physics, 0.016);

        // Gravity still applies, so vel_y changes from initial
        assert!((player.physics_data.vel_y - (-0.32)).abs() < 0.1);
        assert!((player.physics_data.vel_y - initial_vel_y).abs() > 0.1);
    }

    #[test]
    fn test_max_speed_is_enforced() {
        let mut player = Player::default();
        let input = PlayerInput {
            move_x: 1.0,
            ..default_input()
        };
        let physics = Physics::new();

        // Accelerate for many frames
        for _ in 0..100 {
            player.update(&input, &physics, 0.016);
        }

        assert!((player.physics_data.vel_x - 5.0).abs() < 0.1);
    }

    #[test]
    fn test_negative_horizontal_movement() {
        let mut player = Player::default();
        let input = PlayerInput {
            move_x: -1.0,
            ..default_input()
        };
        let physics = Physics::new();
        let dt = 0.016;

        player.update(&input, &physics, dt);

        assert!(player.physics_data.vel_x < 0.0);
    }

    #[test]
    fn test_z_axis_movement() {
        let mut player = Player::default();
        let input = PlayerInput {
            move_z: -1.0,
            ..default_input()
        };
        let physics = Physics::new();
        let dt = 0.016;

        player.update(&input, &physics, dt);

        assert!(player.physics_data.vel_z < 0.0);
    }

    #[test]
    fn test_position_updates_with_velocity() {
        let mut player = Player::default();
        player.physics_data.vel_x = 5.0;
        let input = default_input();
        let physics = Physics::new();
        let dt = 0.016;

        player.update(&input, &physics, dt);
        player.update_position(dt);

        assert!(player.physics_data.x > 0.0);
    }

    #[test]
    fn test_gravity_affects_vertical_position() {
        let mut player = Player::default();
        let jump_input = grounded_input();
        let physics = Physics::new();

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
        let mut player = Player::default();

        player.jump_force = 20.0;

        let physics = Physics::new();

        player.update(&grounded_input(), &physics, 0.016);

        assert!((player.physics_data.vel_y - 20.0).abs() < 0.1);
    }

    #[test]
    fn test_velocity_decreases_to_zero_with_friction() {
        let mut player = Player::default();
        player.physics_data.vel_x = 10.0;
        let input = default_input();
        let physics = Physics::new();

        player.update(&input, &physics, 0.016);

        assert!(player.physics_data.vel_x < 10.0);
        assert!(player.physics_data.vel_x > 0.0);
    }

    #[test]
    fn test_angle_default_is_zero() {
        let player = Player::default();
        assert!((player.angle - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_gun_end_points_forward_by_default() {
        let player = Player::default();
        let end = player.gun_end();
        // With angle=0, should point in +Z direction
        assert!((end.0 - player.physics_data.x).abs() < 0.01);
        assert!(end.2 > player.physics_data.z);
        assert!((end.1 - (player.physics_data.y + player.physics_data.size / 2.0)).abs() < 0.01);
    }

    #[test]
    fn test_gun_end_points_right_at_pi_over_2() {
        let player = Player::default();
        let mut p = player;
        p.angle = std::f32::consts::FRAC_PI_2;
        let end = p.gun_end();
        assert!(end.0 > p.physics_data.x);
        assert!((end.2 - p.physics_data.z).abs() < 0.01);
    }

    #[test]
    fn test_gun_end_points_left_at_minus_pi_over_2() {
        let player = Player::default();
        let mut p = player;
        p.angle = -std::f32::consts::FRAC_PI_2;
        let end = p.gun_end();
        assert!(end.0 < p.physics_data.x);
        assert!((end.2 - p.physics_data.z).abs() < 0.01);
    }

   #[test]
    fn test_shoot_direction_normalized() {
        let player = Player::default();
        let dir = player.shoot_direction();
        let len = (dir.0 * dir.0 + dir.1 * dir.1 + dir.2 * dir.2).sqrt();
        assert!((len - 1.0).abs() < 0.01);
        // With default gun_pitch=0, dir_y is 0 (horizontal)
        assert!((dir.1 - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_shoot_direction_with_positive_pitch() {
        let mut player = Player::default();
        player.gun_pitch = 0.3;
        let dir = player.shoot_direction();
        assert!(dir.1 > 0.0);
    }

    #[test]
    fn test_shoot_direction_with_negative_pitch() {
        let mut player = Player::default();
        player.gun_pitch = -0.3;
        let dir = player.shoot_direction();
        assert!(dir.1 < 0.0);
    }

    #[test]
    fn test_shoot_direction_matches_angle() {
        let player = Player::default();
        let mut p = player;
        p.angle = std::f32::consts::FRAC_PI_4;
        let dir = p.shoot_direction();
        assert!(dir.0 > 0.0);
        assert!(dir.2 > 0.0);
    }

    #[test]
    fn test_fire_creates_projectile() {
        let player = Player::default();
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
        let player = Player::default();
        let projectile = player.fire();
        let total_vel = (projectile.physics_data.vel_x * projectile.physics_data.vel_x
            + projectile.physics_data.vel_y * projectile.physics_data.vel_y
            + projectile.physics_data.vel_z * projectile.physics_data.vel_z)
            .sqrt();
        assert!((total_vel - 15.0).abs() < 0.1);
    }

    #[test]
    fn test_fire_projectile_aims_correctly() {
        let player = Player::default();
        let mut p = player;
        p.angle = std::f32::consts::FRAC_PI_2; // point right
        let projectile = p.fire();
        assert!(projectile.physics_data.vel_x > 10.0);
        assert!(projectile.physics_data.vel_z.abs() < 1.0);
    }

    #[test]
    fn test_new_player_is_not_dead() {
        let player = Player::default();
        assert!(!player.is_player_dead());
    }

    #[test]
    fn test_respawn_sets_player_alive() {
        let mut player = Player::default();
        player.is_dead = true;
        player.physics_data.x = 100.0;
        player.physics_data.vel_x = 50.0;
        player.angle = 1.5;

        player.respawn();

        assert!(!player.is_player_dead());
        assert!((player.physics_data.x - 0.0).abs() < 0.01);
        assert!((player.physics_data.y - 0.0).abs() < 0.01);
        assert!((player.physics_data.z - 0.0).abs() < 0.01);
        assert!((player.physics_data.vel_x - 0.0).abs() < 0.01);
        assert!((player.physics_data.vel_y - 0.0).abs() < 0.01);
        assert!((player.physics_data.vel_z - 0.0).abs() < 0.01);
        assert!((player.angle - 0.0).abs() < 0.01);
        assert!((player.gun_pitch - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_respawn_resets_size() {
        let mut player = Player::default();
        player.physics_data.size = 5.0;
        player.respawn();
        assert!((player.physics_data.size - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_movement_direction_follows_angle() {
        let mut player = Player::default();
        let physics = Physics::new();
        let dt = 0.016;

        // Angle 0: facing +X direction (cos(0)=1, sin(0)=0)
        player.angle = 0.0;
        let input = PlayerInput {
            move_x: 1.0, // cos(0) * forward
            move_z: 0.0, // sin(0) * forward
            ..default_input()
        };
        player.update(&input, &physics, dt);
        assert!(player.physics_data.vel_x > 0.0, "angle=0 should move +X");
        assert!((player.physics_data.vel_z).abs() < 0.01, "angle=0 should not move Z");

        // Reset velocity
        player.physics_data.vel_x = 0.0;
        player.physics_data.vel_z = 0.0;

        // Angle PI/2 (90 degrees): facing +Z direction (cos=0, sin=1)
        player.angle = std::f32::consts::FRAC_PI_2;
        let input = PlayerInput {
            move_x: 0.0, // cos(PI/2) * forward
            move_z: 1.0, // sin(PI/2) * forward
            ..default_input()
        };
        player.update(&input, &physics, dt);
        assert!((player.physics_data.vel_x).abs() < 0.01, "angle=PI/2 should not move X");
        assert!(player.physics_data.vel_z > 0.0, "angle=PI/2 should move +Z");

        // Reset velocity
        player.physics_data.vel_x = 0.0;
        player.physics_data.vel_z = 0.0;

        // Angle PI (180 degrees): facing -X direction (cos=-1, sin=0)
        player.angle = std::f32::consts::PI;
        let input = PlayerInput {
            move_x: -1.0, // cos(PI) * forward
            move_z: 0.0,   // sin(PI) * forward
            ..default_input()
        };
        player.update(&input, &physics, dt);
        assert!(player.physics_data.vel_x < 0.0, "angle=PI should move -X");
        assert!((player.physics_data.vel_z).abs() < 0.01, "angle=PI should not move Z");
    }

    #[test]
    fn test_strafe_direction_follows_angle() {
        let mut player = Player::default();
        let physics = Physics::new();
        let dt = 0.016;

        // Angle 0: strafe right maps to +Z (cos(0)=1, sin(0)=0)
        player.angle = 0.0;
        let input = PlayerInput {
            move_x: 0.0,
            move_z: 1.0,
            ..default_input()
        };
        player.update(&input, &physics, dt);
        assert!((player.physics_data.vel_x).abs() < 0.01, "angle=0 strafe should not move X");
        assert!(player.physics_data.vel_z > 0.0, "angle=0 strafe should move +Z");
    }
}
