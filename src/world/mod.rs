use crate::enemy::Enemy;
use crate::input::Input;
use crate::obstacle::{Aabb, Obstacle, is_colliding};
use crate::physics::{CollisionResult, PhysicsImpl};
use crate::player::Player;
use crate::projectile::Projectile;

/// Temporary AABB wrapper for player collision checks.
struct PlayerAabb {
    x: f32,
    y: f32,
    z: f32,
    size: f32,
}

impl Aabb for PlayerAabb {
    fn min_x(&self) -> f32 {
        self.x - self.size
    }
    fn max_x(&self) -> f32 {
        self.x + self.size
    }
    fn min_y(&self) -> f32 {
        self.y - self.size
    }
    fn max_y(&self) -> f32 {
        self.y + self.size
    }
    fn min_z(&self) -> f32 {
        self.z - self.size
    }
    fn max_z(&self) -> f32 {
        self.z + self.size
    }
}

/// Manages the game world: entities, obstacles, and their interactions.
pub struct World {
    pub entities: Vec<Player>,
    pub obstacles: Vec<Obstacle>,
    pub projectiles: Vec<Projectile>,
    pub enemies: Vec<Enemy>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            obstacles: Vec::new(),
            projectiles: Vec::new(),
            enemies: Vec::new(),
        }
    }

    pub fn add_entity(&mut self, entity: Player) {
        self.entities.push(entity);
    }

    pub fn add_obstacle(&mut self, obstacle: Obstacle) {
        self.obstacles.push(obstacle);
    }

    pub fn add_platform(&mut self, x: f32, y: f32, z: f32, width: f32, height: f32, depth: f32) {
        self.obstacles
            .push(Obstacle::platform(x, y, z, width, height, depth));
    }

    pub fn add_enemy(&mut self, enemy: Enemy) {
        self.enemies.push(enemy);
    }

    pub fn add_projectile(&mut self, projectile: Projectile) {
        self.projectiles.push(projectile);
    }

    pub fn update_projectiles(&mut self, dt: f32) {
        for projectile in &mut self.projectiles {
            projectile.update(dt);
        }

        let mut alive = Vec::new();
        for projectile in self.projectiles.drain(..) {
            if !projectile.is_alive() {
                continue;
            }

            let mut hit = false;
            for obstacle in &self.obstacles {
                if is_colliding(&projectile, obstacle) {
                    hit = true;
                    break;
                }
            }

            if !hit {
                for entity in &self.entities {
                    let player_aabb = PlayerAabb {
                        x: entity.physics_data.x,
                        y: entity.physics_data.y + entity.physics_data.size / 2.0,
                        z: entity.physics_data.z,
                        size: entity.physics_data.size / 2.0,
                    };
                    if is_colliding(&projectile, &player_aabb) {
                        hit = true;
                        break;
                    }
                }
            }

            if !hit {
                alive.push(projectile);
            }
        }
        self.projectiles = alive;
    }

    pub fn update_all(&mut self, input: &Input, physics: &PhysicsImpl, dt: f32) {
        for entity in &mut self.entities {
            entity.update(input, physics, dt);
        }

        // Update enemies and collect projectiles
        let mut enemy_projectiles = Vec::new();
        for enemy in &mut self.enemies {
            if let Some(player) = self.entities.first() {
                enemy.update_ai(player, dt, physics);

                if let Some(projectile) = enemy.try_shoot(player) {
                    enemy_projectiles.push(projectile);
                }
            }
        }

        self.entities
            .iter_mut()
            .for_each(|e| physics.update(e, &self.obstacles, dt));
        self.enemies
            .iter_mut()
            .for_each(|e| physics.update(e, &self.obstacles, dt));
        self.projectiles
            .iter_mut()
            .for_each(|e| physics.update(e, &self.obstacles, dt));

        // Remove dead enemies
        self.enemies.retain(|e| e.alive);

        // Add enemy projectiles
        for projectile in enemy_projectiles {
            self.add_projectile(projectile);
        }

        self.update_projectiles(dt);
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

    pub fn projectile_count(&self) -> usize {
        self.projectiles.len()
    }

    pub fn enemy_count(&self) -> usize {
        self.enemies.len()
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
    use crate::obstacle::ObstacleKind;
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

    #[test]
    fn test_new_world_is_empty() {
        let world = World::new();
        assert_eq!(world.entity_count(), 0);
        assert_eq!(world.obstacle_count(), 0);
        assert_eq!(world.projectile_count(), 0);
    }

    #[test]
    fn test_add_and_remove_entity() {
        let mut world = World::new();
        world.add_entity(Player::default());
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
        let mut player = Player::default();
        player.physics_data.vel_x = 5.0;
        world.add_entity(player);

        let input = default_input();
        let physics = PhysicsImpl::new();
        world.update_all(&input, &physics, 0.016);

        assert!(world.entities[0].physics_data.x > 0.0);
    }

    #[test]
    fn test_obstacle_collision_with_world() {
        let mut world = World::new();
        let mut player = Player::default();
        player.physics_data.y = 5.0;
        player.physics_data.vel_y = -10.0;
        world.add_entity(player);
        world.add_obstacle(Obstacle::solid(0.0, 0.0, 0.0, 10.0, 0.5, 10.0));

        let input = default_input();
        let physics = PhysicsImpl::new();
        world.update_all(&input, &physics, 0.016);

        let entity = &world.entities[0];
        assert!(entity.physics_data.y >= 0.0);
    }

    #[test]
    fn test_multiple_entities_update() {
        let mut world = World::new();
        let mut player1 = Player::default();
        player1.physics_data.vel_x = 5.0;
        world.add_entity(player1);

        let mut player2 = Player::default();
        player2.physics_data.vel_z = -3.0;
        world.add_entity(player2);

        let input = default_input();
        let physics = PhysicsImpl::new();
        world.update_all(&input, &physics, 0.016);

        assert!(world.entities[0].physics_data.x > 0.0);
        assert!(world.entities[1].physics_data.z < 0.0);
    }

    #[test]
    fn test_multiple_obstacles() {
        let mut world = World::new();
        world.add_obstacle(Obstacle::solid(0.0, 1.0, 0.0, 4.0, 0.5, 4.0));
        world.add_obstacle(Obstacle::platform(5.0, 2.0, 0.0, 2.0, 0.5, 2.0));

        assert_eq!(world.obstacle_count(), 2);
    }

    #[test]
    fn test_platform_no_horizontal_collision() {
        let mut world = World::new();
        let mut player = Player::default();
        player.physics_data.x = 3.0;
        player.physics_data.vel_x = 2.0;
        player.physics_data.y = 1.0;
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
        assert!(entity.physics_data.x < 5.5);
    }

    #[test]
    fn test_add_and_remove_projectile() {
        let mut world = World::new();
        let projectile = Projectile::new(0.0, 1.0, 0.0, 5.0, 0.0, 0.0, 10.0);
        world.add_projectile(projectile);
        assert_eq!(world.projectile_count(), 1);

        world.projectiles.clear();
        assert_eq!(world.projectile_count(), 0);
    }

    #[test]
    fn test_projectile_updates_in_world() {
        let mut world = World::new();
        let projectile = Projectile::new(0.0, 1.0, 0.0, 10.0, 0.0, 0.0, 10.0);
        world.add_projectile(projectile);

        world.update_projectiles(0.016);

        assert!(world.projectiles[0].physics_data.x > 0.0);
    }

    #[test]
    fn test_projectile_expires_after_max_lifetime() {
        let mut world = World::new();
        let projectile = Projectile::new(0.0, 1.0, 0.0, 10.0, 0.0, 0.0, 10.0);
        world.add_projectile(projectile);

        while !world.projectiles.is_empty() {
            world.update_projectiles(0.1);
        }

        assert_eq!(world.projectile_count(), 0);
    }

    #[test]
    fn test_projectile_collides_with_obstacle() {
        let mut world = World::new();
        world.add_obstacle(Obstacle::solid(5.0, 1.0, 0.0, 1.0, 2.0, 10.0));
        let projectile = Projectile::new(4.5, 1.0, 0.0, 5.0, 0.0, 0.0, 10.0);
        world.add_projectile(projectile);

        world.update_projectiles(0.016);

        // Projectile should hit the obstacle and be removed
        assert_eq!(world.projectile_count(), 0);
    }

    #[test]
    fn test_projectile_collides_with_player() {
        let mut world = World::new();
        let mut player = Player::default();
        player.physics_data.x = 5.0;
        player.physics_data.y = 0.5;
        world.add_entity(player);
        let projectile = Projectile::new(4.5, 1.0, 0.0, 5.0, 0.0, 0.0, 10.0);
        world.add_projectile(projectile);

        world.update_projectiles(0.016);

        // Projectile should hit the player and be removed
        assert_eq!(world.projectile_count(), 0);
    }

    #[test]
    fn test_update_all_includes_projectile_update() {
        let mut world = World::new();
        let mut player = Player::default();
        player.physics_data.y = 10.0;
        world.add_entity(player);
        world.add_platform(0.0, -0.25, 0.0, 100.0, 0.5, 100.0);
        let projectile = Projectile::new(0.0, 1.0, 0.0, 10.0, 0.0, 0.0, 10.0);
        world.add_projectile(projectile);

        let input = default_input();
        let physics = PhysicsImpl::new();
        world.update_all(&input, &physics, 0.016);

        assert!(world.projectiles[0].physics_data.x > 0.0);
    }
}
