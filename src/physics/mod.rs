/// Core physics calculations: gravity, friction, collision resolution.
/// All constants are configurable via PhysicsConfig for tuning and testing.

use crate::platform::Aabb;

/// Result of a collision detection check.
#[derive(Clone, Debug, PartialEq)]
pub enum CollisionResult {
    /// No collision.
    None,
    /// Collided from below (landing on platform).
    Bottom,
    /// Collided from the side moving right.
    Right,
    /// Collided from the side moving left.
    Left,
}

#[derive(Clone, Debug)]
pub struct PhysicsConfig {
    pub gravity: f32,
    pub jump_force: f32,
    pub max_speed: f32,
    pub acceleration: f32,
    pub friction: f32,
    pub friction_threshold: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: 20.0,
            jump_force: 10.0,
            max_speed: 5.0,
            acceleration: 20.0,
            friction: 5.0,
            friction_threshold: 0.01,
        }
    }
}

pub trait Physics {
    fn apply_gravity(&self, vel_y: &mut f32, dt: f32, config: &PhysicsConfig);
    fn apply_jump(&self, vel_y: &mut f32, config: &PhysicsConfig);
    fn apply_friction(&self, vel: &mut f32, dt: f32, config: &PhysicsConfig);
    fn apply_acceleration(&self, vel: &mut f32, input: f32, dt: f32, config: &PhysicsConfig);
    fn clamp_speed(&self, vel: &mut f32, config: &PhysicsConfig);
    fn resolve_ground_collision(
        &self,
        pos_y: &mut f32,
        vel_y: &mut f32,
        config: &PhysicsConfig,
    );
    fn resolve_platform_collision(
        &self,
        player: &dyn Aabb,
        vel_y: &f32,
        platform: &dyn Aabb,
    ) -> CollisionResult;
    fn resolve_horizontal_collision(
        &self,
        player: &dyn Aabb,
        vel_x: &f32,
        platform: &dyn Aabb,
    ) -> CollisionResult;
}

pub struct PhysicsImpl;

impl PhysicsImpl {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PhysicsImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl Physics for PhysicsImpl {
    fn apply_gravity(&self, vel_y: &mut f32, dt: f32, config: &PhysicsConfig) {
        *vel_y -= config.gravity * dt;
    }

    fn apply_jump(&self, vel_y: &mut f32, config: &PhysicsConfig) {
        *vel_y = config.jump_force;
    }

    fn apply_friction(&self, vel: &mut f32, dt: f32, config: &PhysicsConfig) {
        if vel.abs() > config.friction_threshold {
            let step = config.friction * dt;
            if vel.abs() <= step {
                *vel = 0.0;
            } else {
                *vel -= vel.signum() * step;
                if vel.abs() < config.friction_threshold {
                    *vel = 0.0;
                }
            }
        } else {
            *vel = 0.0;
        }
    }

    fn apply_acceleration(&self, vel: &mut f32, input: f32, dt: f32, config: &PhysicsConfig) {
        *vel += input * config.acceleration * dt;
    }

    fn clamp_speed(&self, vel: &mut f32, config: &PhysicsConfig) {
        if vel.abs() > config.max_speed {
            *vel = config.max_speed * vel.signum();
        }
    }

    fn resolve_ground_collision(
        &self,
        pos_y: &mut f32,
        vel_y: &mut f32,
        config: &PhysicsConfig,
    ) {
        if *pos_y < 0.0 {
            *pos_y = 0.0;
            *vel_y = 0.0;
        }
    }

    fn resolve_platform_collision(
        &self,
        player: &dyn Aabb,
        vel_y: &f32,
        platform: &dyn Aabb,
    ) -> CollisionResult {
        let overlap_x = player.max_x() >= platform.min_x() && player.min_x() <= platform.max_x();
        let overlap_z = player.max_z() >= platform.min_z() && player.min_z() <= platform.max_z();
        if !overlap_x || !overlap_z {
            return CollisionResult::None;
        }

        if *vel_y < 0.0 && player.min_y() <= platform.max_y() + 0.01 {
            CollisionResult::Bottom
        } else {
            CollisionResult::None
        }
    }

    fn resolve_horizontal_collision(
        &self,
        player: &dyn Aabb,
        vel_x: &f32,
        platform: &dyn Aabb,
    ) -> CollisionResult {
        let overlap_y = player.max_y() > platform.min_y() && player.min_y() < platform.max_y();
        let overlap_z = player.max_z() >= platform.min_z() && player.min_z() <= platform.max_z();
        if !overlap_y || !overlap_z {
            return CollisionResult::None;
        }

        if *vel_x > 0.0 && player.max_x() > platform.min_x() && player.min_x() <= platform.min_x() {
            CollisionResult::Right
        } else if *vel_x < 0.0 && player.min_x() < platform.max_x() && player.max_x() >= platform.max_x() {
            CollisionResult::Left
        } else {
            CollisionResult::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::Platform;

    fn default_config() -> PhysicsConfig {
        PhysicsConfig::default()
    }

    #[test]
    fn test_gravity_decreases_vertical_velocity() {
        let physics = PhysicsImpl::new();
        let mut vel_y = 0.0;
        let dt = 0.016;
        let config = default_config();

        physics.apply_gravity(&mut vel_y, dt, &config);

        assert!((vel_y - (-0.32)).abs() < 0.01);
    }

    #[test]
    fn test_jump_sets_vertical_velocity() {
        let physics = PhysicsImpl::new();
        let mut vel_y = 0.0;
        let config = default_config();

        physics.apply_jump(&mut vel_y, &config);

        assert!((vel_y - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_ground_collision_resets_velocity() {
        let physics = PhysicsImpl::new();
        let mut pos_y = -1.0;
        let mut vel_y = -5.0;
        let config = default_config();

        physics.resolve_ground_collision(&mut pos_y, &mut vel_y, &config);

        assert!((pos_y - 0.0).abs() < 0.01);
        assert!((vel_y - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_friction_reduces_velocity() {
        let physics = PhysicsImpl::new();
        let mut vel_x = 5.0;
        let dt = 0.016;
        let config = PhysicsConfig {
            friction: 5.0,
            ..default_config()
        };

        physics.apply_friction(&mut vel_x, dt, &config);

        assert!(vel_x < 5.0);
        assert!(vel_x > 0.0);
    }

    #[test]
    fn test_friction_stops_velocity_below_threshold() {
        let physics = PhysicsImpl::new();
        let mut vel_x = 0.005;
        let dt = 0.016;
        let config = PhysicsConfig {
            friction_threshold: 0.01,
            ..default_config()
        };

        physics.apply_friction(&mut vel_x, dt, &config);

        assert!((vel_x - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_acceleration_increases_velocity() {
        let physics = PhysicsImpl::new();
        let mut vel_x = 0.0;
        let dt = 0.016;
        let config = default_config();

        physics.apply_acceleration(&mut vel_x, 1.0, dt, &config);

        assert!((vel_x - 0.32).abs() < 0.01);
    }

    #[test]
    fn test_clamp_speed_caps_velocity() {
        let physics = PhysicsImpl::new();
        let mut vel_x = 10.0;
        let config = PhysicsConfig {
            max_speed: 5.0,
            ..default_config()
        };

        physics.clamp_speed(&mut vel_x, &config);

        assert!((vel_x - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_clamp_speed_preserves_negative_direction() {
        let physics = PhysicsImpl::new();
        let mut vel_x = -10.0;
        let config = PhysicsConfig {
            max_speed: 5.0,
            ..default_config()
        };

        physics.clamp_speed(&mut vel_x, &config);

        assert!((vel_x - (-5.0)).abs() < 0.01);
    }

    #[test]
    fn test_clamp_speed_no_change_when_below_max() {
        let physics = PhysicsImpl::new();
        let mut vel_x = 3.0;
        let config = PhysicsConfig {
            max_speed: 5.0,
            ..default_config()
        };

        let original = vel_x;
        physics.clamp_speed(&mut vel_x, &config);

        assert!((vel_x - original).abs() < 0.01);
    }

    #[test]
    fn test_platform_collision_detects_landing() {
        let physics = PhysicsImpl::new();
        let player = TestPlayer::new(2.55, 1.0);
        let vel_y = -2.0;
        let platform = Platform::new(0.0, 2.05, 0.0, 4.0, 0.1, 4.0);

        let result = physics.resolve_platform_collision(&player, &vel_y, &platform);

        assert_eq!(result, CollisionResult::Bottom);
    }

    #[test]
    fn test_platform_collision_no_landing_when_falling_from_far() {
        let physics = PhysicsImpl::new();
        let player = TestPlayer::new(0.0, 1.0);
        let vel_y = -2.0;
        let platform = Platform::new(0.0, 2.05, 0.0, 4.0, 0.1, 4.0);

        let result = physics.resolve_platform_collision(&player, &vel_y, &platform);

        assert_eq!(result, CollisionResult::None);
    }

    #[test]
    fn test_platform_collision_no_when_rising() {
        let physics = PhysicsImpl::new();
        let player = TestPlayer::new(2.5, 1.0);
        let vel_y = 2.0;
        let platform = Platform::new(0.0, 2.05, 0.0, 4.0, 0.1, 4.0);

        let result = physics.resolve_platform_collision(&player, &vel_y, &platform);

        assert_eq!(result, CollisionResult::None);
    }

    #[test]
    fn test_horizontal_collision_no_collision_outside_z_range() {
        let physics = PhysicsImpl::new();
        let player = TestPlayer::new_at(0.0, 0.5, 6.0, 1.0);
        let vel_x = 1.0;
        let platform = Platform::new(0.0, 0.5, 0.0, 10.0, 1.0, 4.0);

        let result = physics.resolve_horizontal_collision(&player, &vel_x, &platform);

        assert_eq!(result, CollisionResult::None);
    }

    #[test]
    fn test_horizontal_collision_no_collision_outside_y_range() {
        let physics = PhysicsImpl::new();
        let player = TestPlayer::new_at(0.0, 5.0, 0.0, 1.0);
        let vel_x = 1.0;
        let platform = Platform::new(0.0, 0.5, 0.0, 10.0, 1.0, 10.0);

        let result = physics.resolve_horizontal_collision(&player, &vel_x, &platform);

        assert_eq!(result, CollisionResult::None);
    }

    #[test]
    fn test_horizontal_collision_hits_from_left() {
        let physics = PhysicsImpl::new();
        let player = TestPlayer::new_at(4.6, 0.5, 0.0, 1.0);
        let vel_x = 1.0;
        let platform = Platform::new(5.0, 0.5, 0.0, 1.0, 1.0, 2.0);

        let result = physics.resolve_horizontal_collision(&player, &vel_x, &platform);

        assert_eq!(result, CollisionResult::Right);
    }

    #[test]
    fn test_horizontal_collision_hits_from_right() {
        let physics = PhysicsImpl::new();
        let player = TestPlayer::new_at(-4.6, 0.5, 0.0, 1.0);
        let vel_x = -1.0;
        let platform = Platform::new(-5.0, 0.5, 0.0, 1.0, 1.0, 2.0);

        let result = physics.resolve_horizontal_collision(&player, &vel_x, &platform);

        assert_eq!(result, CollisionResult::Left);
    }

    #[test]
    fn test_horizontal_collision_no_collision_when_stationary() {
        let physics = PhysicsImpl::new();
        let player = TestPlayer::new_at(4.0, 0.5, 0.0, 1.0);
        let vel_x = 0.0;
        let platform = Platform::new(5.0, 0.5, 0.0, 1.0, 1.0, 2.0);

        let result = physics.resolve_horizontal_collision(&player, &vel_x, &platform);

        assert_eq!(result, CollisionResult::None);
    }

    /// Simple test player that implements Aabb for physics tests.
    struct TestPlayer {
        px: f32,
        py: f32,
        pz: f32,
        size: f32,
    }

    impl TestPlayer {
        fn new(py: f32, size: f32) -> Self {
            Self { px: 0.0, py, pz: 0.0, size }
        }
        fn new_at(px: f32, py: f32, pz: f32, size: f32) -> Self {
            Self { px, py, pz, size }
        }
    }

    impl Aabb for TestPlayer {
        fn min_x(&self) -> f32 { self.px - self.size / 2.0 }
        fn max_x(&self) -> f32 { self.px + self.size / 2.0 }
        fn min_y(&self) -> f32 { self.py }
        fn max_y(&self) -> f32 { self.py + self.size }
        fn min_z(&self) -> f32 { self.pz - self.size / 2.0 }
        fn max_z(&self) -> f32 { self.pz + self.size / 2.0 }
    }
}
