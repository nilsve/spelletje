use crate::player::Player;
use crate::platform::{Aabb, Platform};
use crate::physics::{Physics, PhysicsConfig, CollisionResult};
use crate::input::Input;

/// Manages the game world: entities, platforms, and their interactions.
pub struct World {
    pub entities: Vec<Player>,
    pub platforms: Vec<Platform>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            platforms: Vec::new(),
        }
    }

    pub fn add_entity(&mut self, entity: Player) {
        self.entities.push(entity);
    }

    pub fn add_platform(&mut self, platform: Platform) {
        self.platforms.push(platform);
    }

    pub fn update_all(&mut self, input: &Input, physics: &dyn Physics, dt: f32) {
        for entity in &mut self.entities {
            entity.update(input, physics, dt);
        }

        for entity in &mut self.entities {
            for platform in &self.platforms {
                let collision = physics.resolve_platform_collision(
                    entity,
                    &entity.vel_y,
                    platform,
                );
                if collision == CollisionResult::Bottom {
                    entity.y = platform.max_y();
                    entity.vel_y = 0.0;
                    entity.set_grounded(true);
                    let config = entity.config.to_physics_config();
                    physics.apply_friction(&mut entity.vel_x, dt, &config);
                }

                let h_collision = physics.resolve_horizontal_collision(
                    entity,
                    &entity.vel_x,
                    platform,
                );
                match h_collision {
                    CollisionResult::Right => {
                        if entity.x < platform.x {
                            entity.x = platform.min_x() + entity.size / 2.0;
                        } else {
                            entity.x = platform.max_x() - entity.size / 2.0;
                        }
                        entity.vel_x = 0.0;
                    }
                    CollisionResult::Left => {
                        if entity.x < platform.x {
                            entity.x = platform.min_x() - entity.size / 2.0;
                        } else {
                            entity.x = platform.max_x() + entity.size / 2.0;
                        }
                        entity.vel_x = 0.0;
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn remove_entity(&mut self, index: usize) {
        self.entities.remove(index);
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn platform_count(&self) -> usize {
        self.platforms.len()
    }
}

impl Default for World {
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

    #[test]
    fn test_new_world_is_empty() {
        let world = World::new();
        assert_eq!(world.entity_count(), 0);
        assert_eq!(world.platform_count(), 0);
    }

    #[test]
    fn test_add_and_remove_entity() {
        let mut world = World::new();
        world.add_entity(Player::new());
        assert_eq!(world.entity_count(), 1);

        world.remove_entity(0);
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_add_and_remove_platform() {
        let mut world = World::new();
        world.add_platform(Platform::new(0.0, 0.0, 0.0, 4.0, 0.5, 4.0));
        assert_eq!(world.platform_count(), 1);

        world.platforms.clear();
        assert_eq!(world.platform_count(), 0);
    }

    #[test]
    fn test_entity_updates_in_world() {
        let mut world = World::new();
        let mut player = Player::new();
        player.vel_x = 5.0;
        world.add_entity(player);

        let input = default_input();
        let physics = PhysicsImpl::new();
        world.update_all(&input, &physics, 0.016);

        assert!(world.entities[0].x > 0.0);
    }

    #[test]
    fn test_platform_collision_with_world() {
        let mut world = World::new();
        let mut player = Player::new();
        player.y = 5.0;
        player.vel_y = -10.0;
        world.add_entity(player);
        world.add_platform(Platform::new(0.0, 0.0, 0.0, 10.0, 0.5, 10.0));

        let input = default_input();
        let physics = PhysicsImpl::new();
        world.update_all(&input, &physics, 0.016);

        let entity = &world.entities[0];
        assert!(entity.y >= 0.0);
    }

    #[test]
    fn test_multiple_entities_update() {
        let mut world = World::new();
        let mut player1 = Player::new();
        player1.vel_x = 5.0;
        world.add_entity(player1);

        let mut player2 = Player::new();
        player2.vel_z = -3.0;
        world.add_entity(player2);

        let input = default_input();
        let physics = PhysicsImpl::new();
        world.update_all(&input, &physics, 0.016);

        assert!(world.entities[0].x > 0.0);
        assert!(world.entities[1].z < 0.0);
    }

    #[test]
    fn test_multiple_platforms() {
        let mut world = World::new();
        world.add_platform(Platform::new(0.0, 1.0, 0.0, 4.0, 0.5, 4.0));
        world.add_platform(Platform::new(5.0, 2.0, 0.0, 2.0, 0.5, 2.0));

        assert_eq!(world.platform_count(), 2);
    }

    #[test]
    fn test_player_on_platform_stops_falling() {
        let mut world = World::new();
        let mut player = Player::new();
        player.y = 1.55;
        player.vel_y = -0.5;
        world.add_entity(player);
        world.add_platform(Platform::new(0.0, 1.0, 0.0, 10.0, 0.5, 10.0));

        let input = default_input();
        let physics = PhysicsImpl::new();
        world.update_all(&input, &physics, 0.016);

        let entity = &world.entities[0];
        assert!((entity.vel_y - 0.0).abs() < 0.01);
    }
}
