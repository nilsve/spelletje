/// Core physics calculations: gravity, friction, collision resolution.
/// All constants are configurable via PhysicsConfig for tuning and testing.
use crate::obstacle::{Aabb, ObstacleKind};

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
    /// Collided from the top moving up.
    Top,
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

/// Shared positioning data for all physics-enabled entities.
#[derive(Clone, Debug, Default)]
pub struct PhysicsData {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub vel_z: f32,
    pub size: f32,
}

/// Trait for entities that provide positioning data for physics calculations.
pub trait PhysicsEntity {
    fn physics_data(&self) -> &PhysicsData;
    fn physics_data_mut(&mut self) -> &mut PhysicsData;
    fn update_position(&mut self, dt: f32);
}

/// Physics engine: wraps configuration and provides physics calculations.
pub struct PhysicsImpl {
    config: PhysicsConfig,
}

impl Default for PhysicsImpl {
    fn default() -> Self {
        Self { config: PhysicsConfig::default() }
    }
}

impl PhysicsImpl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: PhysicsConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &PhysicsConfig {
        &self.config
    }

    pub fn apply_gravity(&self, vel_y: &mut f32, dt: f32) {
        *vel_y -= self.config.gravity * dt;
    }

    pub fn apply_jump(&self, vel_y: &mut f32) {
        *vel_y = self.config.jump_force;
    }

    pub fn apply_friction(&self, vel: &mut f32, dt: f32) {
        if vel.abs() > self.config.friction_threshold {
            let step = self.config.friction * dt;
            if vel.abs() <= step {
                *vel = 0.0;
            } else {
                *vel -= vel.signum() * step;
                if vel.abs() < self.config.friction_threshold {
                    *vel = 0.0;
                }
            }
        } else {
            *vel = 0.0;
        }
    }

    pub fn apply_acceleration(&self, vel: &mut f32, input: f32, dt: f32) {
        *vel += input * self.config.acceleration * dt;
    }

    pub fn clamp_speed(&self, vel: &mut f32) {
        if vel.abs() > self.config.max_speed {
            *vel = self.config.max_speed * vel.signum();
        }
    }

    pub fn resolve_platform_collision(
        &self,
        player: &dyn PhysicsEntity,
        vel_y: &f32,
        obstacle: &dyn Aabb,
        kind: ObstacleKind,
    ) -> CollisionResult {
        let pd = player.physics_data();
        if kind == ObstacleKind::Platform {
            let overlap_x = pd.x + pd.size / 2.0 >= obstacle.min_x() && pd.x - pd.size / 2.0 <= obstacle.max_x();
            let overlap_z = pd.z + pd.size / 2.0 >= obstacle.min_z() && pd.z - pd.size / 2.0 <= obstacle.max_z();
            if !overlap_x || !overlap_z {
                return CollisionResult::None;
            }

            if *vel_y < 0.0 && pd.y <= obstacle.max_y() {
                CollisionResult::Bottom
            } else {
                CollisionResult::None
            }
        } else {
            let overlap_x = pd.x + pd.size / 2.0 >= obstacle.min_x() && pd.x - pd.size / 2.0 <= obstacle.max_x();
            let overlap_z = pd.z + pd.size / 2.0 >= obstacle.min_z() && pd.z - pd.size / 2.0 <= obstacle.max_z();
            if !overlap_x || !overlap_z {
                return CollisionResult::None;
            }

            if *vel_y < 0.0 && pd.y <= obstacle.max_y() && pd.y >= obstacle.max_y() - 0.5 {
                CollisionResult::Bottom
            } else if *vel_y > 0.0 && pd.y + pd.size >= obstacle.min_y() && pd.y + pd.size <= obstacle.min_y() + 0.5 {
                CollisionResult::Top
            } else {
                CollisionResult::None
            }
        }
    }

    pub fn resolve_horizontal_collision(
        &self,
        player: &dyn PhysicsEntity,
        vel_x: &f32,
        obstacle: &dyn Aabb,
        kind: ObstacleKind,
    ) -> CollisionResult {
        if kind == ObstacleKind::Platform {
            return CollisionResult::None;
        }

        let pd = player.physics_data();
        let overlap_y = pd.y + pd.size > obstacle.min_y() && pd.y < obstacle.max_y();
        let overlap_z = pd.z + pd.size / 2.0 >= obstacle.min_z() && pd.z - pd.size / 2.0 <= obstacle.max_z();
        if !overlap_y || !overlap_z {
            return CollisionResult::None;
        }

        if *vel_x > 0.0 && pd.x + pd.size / 2.0 > obstacle.min_x() && pd.x - pd.size / 2.0 <= obstacle.min_x() {
            CollisionResult::Right
        } else if *vel_x < 0.0 && pd.x - pd.size / 2.0 < obstacle.max_x() && pd.x + pd.size / 2.0 >= obstacle.max_x() {
            CollisionResult::Left
        } else {
            CollisionResult::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obstacle::Obstacle;

    fn default_config() -> PhysicsConfig {
        PhysicsConfig::default()
    }

    #[test]
    fn test_gravity_decreases_vertical_velocity() {
        let physics = PhysicsImpl::new();
        let mut vel_y = 0.0;
        let dt = 0.016;

        physics.apply_gravity(&mut vel_y, dt);

        assert!((vel_y - (-0.32)).abs() < 0.01);
    }

    #[test]
    fn test_jump_sets_vertical_velocity() {
        let physics = PhysicsImpl::new();
        let mut vel_y = 0.0;

        physics.apply_jump(&mut vel_y);

        assert!((vel_y - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_friction_reduces_velocity() {
        let physics = PhysicsImpl::new();
        let mut vel_x = 5.0;
        let dt = 0.016;

        physics.apply_friction(&mut vel_x, dt);

        assert!(vel_x < 5.0);
        assert!(vel_x > 0.0);
    }

    #[test]
    fn test_friction_stops_velocity_below_threshold() {
        let physics = PhysicsImpl::new();
        let mut vel_x = 0.005;
        let dt = 0.016;

        physics.apply_friction(&mut vel_x, dt);

        assert!((vel_x - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_clamp_speed_reduces_velocity_above_max() {
        let physics = PhysicsImpl::new();
        let mut vel_x = -10.0;

        physics.clamp_speed(&mut vel_x);

        assert!((vel_x - (-5.0)).abs() < 0.01);
    }

    #[test]
    fn test_clamp_speed_no_change_when_below_max() {
        let physics = PhysicsImpl::new();
        let mut vel_x = 3.0;

        let original = vel_x;
        physics.clamp_speed(&mut vel_x);

        assert!((vel_x - original).abs() < 0.01);
    }

    #[test]
    fn test_acceleration_increases_velocity() {
        let physics = PhysicsImpl::new();
        let mut vel_x = 0.0;
        let input = 1.0;
        let dt = 0.016;

        physics.apply_acceleration(&mut vel_x, input, dt);

        assert!(vel_x > 0.0);
    }

    #[test]
    fn test_acceleration_with_negative_input() {
        let physics = PhysicsImpl::new();
        let mut vel_x = 0.0;
        let input = -1.0;
        let dt = 0.016;

        physics.apply_acceleration(&mut vel_x, input, dt);

        assert!(vel_x < 0.0);
    }

    #[test]
    fn test_resolve_platform_collision_bottom() {
        let physics = PhysicsImpl::new();
        let player = TestPlayer::new(1.0, 1.0);
        let vel_y = -2.0;
        let obstacle = Obstacle::solid(0.0, 1.05, 0.0, 4.0, 0.1, 4.0);

        let result = physics.resolve_platform_collision(&player, &vel_y, &obstacle, obstacle.kind.clone());

        assert_eq!(result, CollisionResult::Bottom);
    }

    #[test]
    fn test_resolve_platform_collision_no_collision() {
        let physics = PhysicsImpl::new();
        let player = TestPlayer::new(10.0, 1.0);
        let vel_y = -2.0;
        let obstacle = Obstacle::solid(0.0, 1.05, 0.0, 4.0, 0.1, 4.0);

        let result = physics.resolve_platform_collision(&player, &vel_y, &obstacle, obstacle.kind.clone());

        assert_eq!(result, CollisionResult::None);
    }

    #[test]
    fn test_resolve_platform_collision_top() {
        let physics = PhysicsImpl::new();
        let player = TestPlayer::new(1.5, 1.0);
        let vel_y = 2.0;
        let obstacle = Obstacle::solid(0.0, 2.55, 0.0, 4.0, 0.1, 4.0);

        let result = physics.resolve_platform_collision(&player, &vel_y, &obstacle, obstacle.kind.clone());

        assert_eq!(result, CollisionResult::Top);
    }

    #[test]
    fn test_resolve_platform_collision_no_overlap_x() {
        let physics = PhysicsImpl::new();
        let player = TestPlayer::new(1.0, 1.0);
        let vel_y = -2.0;
        let obstacle = Obstacle::solid(10.0, 1.05, 0.0, 4.0, 0.1, 4.0);

        let result = physics.resolve_platform_collision(&player, &vel_y, &obstacle, obstacle.kind.clone());

        assert_eq!(result, CollisionResult::None);
    }

    #[test]
    fn test_resolve_platform_collision_no_overlap_z() {
        let physics = PhysicsImpl::new();
        let player = TestPlayer::new(1.0, 1.0);
        let vel_y = -2.0;
        let obstacle = Obstacle::solid(0.0, 1.05, 10.0, 4.0, 0.1, 4.0);

        let result = physics.resolve_platform_collision(&player, &vel_y, &obstacle, obstacle.kind.clone());

        assert_eq!(result, CollisionResult::None);
    }

    #[test]
    fn test_resolve_horizontal_collision_right() {
        let physics = PhysicsImpl::new();
        let player = TestPlayer::new_at(-0.6, 0.5, 0.0, 1.0);
        let vel_x = 1.0;
        let obstacle = Obstacle::solid(0.0, 0.5, 0.0, 1.0, 1.0, 2.0);

        let result = physics.resolve_horizontal_collision(&player, &vel_x, &obstacle, obstacle.kind.clone());

        assert_eq!(result, CollisionResult::Right);
    }

    #[test]
    fn test_resolve_horizontal_collision_left() {
        let physics = PhysicsImpl::new();
        let player = TestPlayer::new_at(0.6, 0.5, 0.0, 1.0);
        let vel_x = -1.0;
        let obstacle = Obstacle::solid(0.0, 0.5, 0.0, 1.0, 1.0, 2.0);

        let result = physics.resolve_horizontal_collision(&player, &vel_x, &obstacle, obstacle.kind.clone());

        assert_eq!(result, CollisionResult::Left);
    }

    #[test]
    fn test_resolve_horizontal_collision_no_collision() {
        let physics = PhysicsImpl::new();
        let player = TestPlayer::new_at(4.6, 0.5, 0.0, 1.0);
        let vel_x = 1.0;
        let obstacle = Obstacle::solid(5.0, 0.5, 0.0, 1.0, 1.0, 2.0);

        let result = physics.resolve_horizontal_collision(&player, &vel_x, &obstacle, obstacle.kind.clone());

        assert_eq!(result, CollisionResult::None);
    }

    #[test]
    fn test_resolve_horizontal_collision_no_overlap_y() {
        let physics = PhysicsImpl::new();
        let player = TestPlayer::new_at(0.5, 2.0, 0.0, 1.0);
        let vel_x = 1.0;
        let obstacle = Obstacle::solid(0.0, 0.5, 0.0, 1.0, 1.0, 2.0);

        let result = physics.resolve_horizontal_collision(&player, &vel_x, &obstacle, obstacle.kind.clone());

        assert_eq!(result, CollisionResult::None);
    }

    #[test]
    fn test_platform_collision_no_landing_when_falling_from_far() {
        let physics = PhysicsImpl::new();
        let player = TestPlayer::new(4.0, 1.0);
        let vel_y = -2.0;
        let obstacle = Obstacle::solid(0.0, 2.05, 0.0, 4.0, 0.1, 4.0);

        let result = physics.resolve_platform_collision(&player, &vel_y, &obstacle, obstacle.kind.clone());

        assert_eq!(result, CollisionResult::None);
    }

    /// Simple test player that implements PhysicsEntity for physics tests.
    struct TestPlayer {
        data: PhysicsData,
    }

    impl TestPlayer {
        fn new(py: f32, size: f32) -> Self {
            Self { data: PhysicsData { x: 0.0, y: py, z: 0.0, vel_x: 0.0, vel_y: 0.0, vel_z: 0.0, size } }
        }
        fn new_at(px: f32, py: f32, pz: f32, size: f32) -> Self {
            Self { data: PhysicsData { x: px, y: py, z: pz, vel_x: 0.0, vel_y: 0.0, vel_z: 0.0, size } }
        }
    }

    impl PhysicsEntity for TestPlayer {
        fn physics_data(&self) -> &PhysicsData { &self.data }
        fn physics_data_mut(&mut self) -> &mut PhysicsData { &mut self.data }
        fn update_position(&mut self, dt: f32) {
            self.data.x += self.data.vel_x * dt;
            self.data.y += self.data.vel_y * dt;
            self.data.z += self.data.vel_z * dt;
        }
    }

    impl Aabb for TestPlayer {
        fn min_x(&self) -> f32 { self.data.x - self.data.size / 2.0 }
        fn max_x(&self) -> f32 { self.data.x + self.data.size / 2.0 }
        fn min_y(&self) -> f32 { self.data.y }
        fn max_y(&self) -> f32 { self.data.y + self.data.size }
        fn min_z(&self) -> f32 { self.data.z - self.data.size / 2.0 }
        fn max_z(&self) -> f32 { self.data.z + self.data.size / 2.0 }
    }
}
