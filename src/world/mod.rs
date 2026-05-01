use crate::player::Player;
use crate::obstacle::{Aabb, Obstacle};
use crate::physics::{Physics, CollisionResult};
use crate::input::Input;

/// Manages the game world: entities, obstacles, and their interactions.
pub struct World {
    pub entities: Vec<Player>,
    pub obstacles: Vec<Obstacle>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            obstacles: Vec::new(),
        }
    }

    pub fn add_entity(&mut self, entity: Player) {
        self.entities.push(entity);
    }

    pub fn add_obstacle(&mut self, obstacle: Obstacle) {
        self.obstacles.push(obstacle);
    }

    pub fn add_platform(&mut self, x: f32, y: f32, z: f32, width: f32, height: f32, depth: f32) {
        self.obstacles.push(Obstacle::platform(x, y, z, width, height, depth));
    }

    pub fn update_all(&mut self, input: &Input, physics: &dyn Physics, dt: f32) {
        for entity in &mut self.entities {
            entity.update(input, physics, dt);
        }

        for entity in &mut self.entities {
            for obstacle in &self.obstacles {
                let collision = physics.resolve_platform_collision(
                    entity,
                    &entity.vel_y,
                    obstacle,
                    obstacle.kind.clone(),
                );
                if collision == CollisionResult::Bottom {
                    entity.y = obstacle.max_y();
                    entity.vel_y = 0.0;
                    entity.set_grounded(true);
                    let config = entity.config.to_physics_config();
                    physics.apply_friction(&mut entity.vel_x, dt, &config);
                }

                let h_collision = physics.resolve_horizontal_collision(
                    entity,
                    &entity.vel_x,
                    obstacle,
                    obstacle.kind.clone(),
                );
                match h_collision {
                    CollisionResult::Right => {
                        if entity.x < obstacle.x {
                            entity.x = obstacle.min_x() + entity.size / 2.0;
                        } else {
                            entity.x = obstacle.max_x() - entity.size / 2.0;
                        }
                        entity.vel_x = 0.0;
                    }
                    CollisionResult::Left => {
                        if entity.x < obstacle.x {
                            entity.x = obstacle.min_x() - entity.size / 2.0;
                        } else {
                            entity.x = obstacle.max_x() + entity.size / 2.0;
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

    pub fn obstacle_count(&self) -> usize {
        self.obstacles.len()
    }

    pub fn platform_count(&self) -> usize {
        self.obstacle_count()
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
        assert_eq!(world.obstacle_count(), 0);
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
    fn test_add_and_remove_obstacle() {
        let mut world = World::new();
        world.add_obstacle(Obstacle::solid(0.0, 0.0, 0.0, 4.0, 0.5, 4.0));
        assert_eq!(world.obstacle_count(), 1);

        world.obstacles.clear();
        assert_eq!(world.obstacle_count(), 0);
    }

    #[test]
    fn test_add_platform_convenience() {
        let mut world = World::new();
        world.add_platform(0.0, -0.25, 0.0, 100.0, 0.5, 100.0);
        assert_eq!(world.obstacle_count(), 1);
        assert_eq!(world.obstacles[0].kind, ObstacleKind::Platform);
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
    fn test_obstacle_collision_with_world() {
        let mut world = World::new();
        let mut player = Player::new();
        player.y = 5.0;
        player.vel_y = -10.0;
        world.add_entity(player);
        world.add_obstacle(Obstacle::solid(0.0, 0.0, 0.0, 10.0, 0.5, 10.0));

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
    fn test_multiple_obstacles() {
        let mut world = World::new();
        world.add_obstacle(Obstacle::solid(0.0, 1.0, 0.0, 4.0, 0.5, 4.0));
        world.add_obstacle(Obstacle::platform(5.0, 2.0, 0.0, 2.0, 0.5, 2.0));

        assert_eq!(world.obstacle_count(), 2);
    }

    #[test]
    fn test_player_on_obstacle_stops_falling() {
        let mut world = World::new();
        let mut player = Player::new();
        player.y = 1.55;
        player.vel_y = -0.5;
        world.add_entity(player);
        world.add_obstacle(Obstacle::solid(0.0, 1.0, 0.0, 10.0, 0.5, 10.0));

        let input = default_input();
        let physics = PhysicsImpl::new();
        world.update_all(&input, &physics, 0.016);

        let entity = &world.entities[0];
        assert!((entity.vel_y - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_platform_no_horizontal_collision() {
        let mut world = World::new();
        let mut player = Player::new();
        player.x = 3.0;
        player.vel_x = 2.0;
        player.y = 1.0;
        world.add_entity(player);
        // Solid wall at x=5
        world.add_obstacle(Obstacle::solid(5.0, 1.0, 0.0, 1.0, 2.0, 10.0));
        // Platform above (should not block horizontal movement)
        world.add_obstacle(Obstacle::platform(5.0, 3.0, 0.0, 4.0, 0.5, 10.0));

        let input = default_input();
        let physics = PhysicsImpl::new();
        world.update_all(&input, &physics, 0.016);

        let entity = &world.entities[0];
        // Should hit the solid wall at x=5
        assert!(entity.x < 5.5);
    }
}
