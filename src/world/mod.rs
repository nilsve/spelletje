use crate::PhysicsEntity;
use crate::enemy::Enemy;
use crate::hill::Hill;
use crate::input::PlayerInput;
use crate::obstacle::{Aabb, Obstacle, ObstacleKind, is_colliding};
use crate::physics::Physics;
use crate::player::Player;
use crate::powerup::{PowerUp, PowerUpKind};
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
    pub players: Vec<Player>,
    pub obstacles: Vec<Obstacle>,
    pub projectiles: Vec<Projectile>,
    pub enemies: Vec<Enemy>,
    pub hill: Hill,
    pub powerups: Vec<PowerUp>,
}

impl World {
    pub fn new() -> Self {
        Self {
            players: Vec::new(),
            obstacles: Vec::new(),
            projectiles: Vec::new(),
            enemies: Vec::new(),
            hill: Hill::new(),
            powerups: Vec::new(),
        }
    }

    pub fn add_entity(&mut self, entity: Player) {
        self.players.push(entity);
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
                for entity in &self.players {
                    let player_aabb = PlayerAabb {
                        x: entity.physics_data().x,
                        y: entity.physics_data().y + entity.physics_data().size / 2.0,
                        z: entity.physics_data().z,
                        size: entity.physics_data().size / 2.0,
                    };
                    if is_colliding(&projectile, &player_aabb) {
                        hit = true;
                        break;
                    }
                }
            }

            if !hit {
                for enemy in &self.enemies {
                    if is_colliding(&projectile, enemy) {
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

    pub fn update_all(&mut self, player_inputs: &[PlayerInput], physics: &Physics, dt: f32) {
        let default_input = PlayerInput::default();
        for (i, entity) in self.players.iter_mut().enumerate() {
            let input = player_inputs.get(i).unwrap_or(&default_input);
            entity.update(input, physics, dt);
        }

        // Reset BiggerHill if no player has it active
        let any_bigger_hill = self.players.iter().any(|p| p.active_powerup == Some(PowerUpKind::BiggerHill));
        if !any_bigger_hill {
            self.hill.width = self.hill.baseline_width;
            self.hill.depth = self.hill.baseline_depth;
        }

        // Update enemies and collect projectiles
        let mut enemy_projectiles = Vec::new();
        for enemy in &mut self.enemies {
            if let Some(player) = self.players.first() {
                enemy.update_ai(player, dt, physics);

                if let Some(projectile) = enemy.try_shoot(player) {
                    enemy_projectiles.push(projectile);
                }
            }
        }

        self.players.iter_mut().for_each(|e| e.update_position(dt));
        self.enemies.iter_mut().for_each(|e| e.update_position(dt));
        self.projectiles
            .iter_mut()
            .for_each(|e| e.update_position(dt));

        self.players
            .iter_mut()
            .for_each(|e| physics.update(e, &self.obstacles, dt));
        self.enemies
            .iter_mut()
            .for_each(|e| physics.update(e, &self.obstacles, dt));
        self.projectiles
            .iter_mut()
            .for_each(|e| physics.update(e, &self.obstacles, dt));

        // Hill: check which player is on it and update scores
        self.update_hill(dt);

        // Update power-ups and check collection
        self.update_powerups(dt);

        // Remove dead enemies
        self.enemies.retain(|e| e.alive);

        // Add enemy projectiles and update all projectiles
        for projectile in enemy_projectiles {
            self.add_projectile(projectile);
        }

        self.update_projectiles(dt);
    }

   pub fn update_hill(&mut self, dt: f32) {
        // Check which player is on the hill
        let hill_obstacle = self.hill.to_obstacle();
        let mut current_holder: Option<usize> = None;

        for (i, player) in self.players.iter().enumerate() {
            let player_aabb = PlayerAabb {
                x: player.physics_data().x,
                y: player.physics_data().y,
                z: player.physics_data().z,
                size: player.physics_data().size / 2.0,
            };

            if is_colliding(&hill_obstacle, &player_aabb) {
                // Check if player is standing on top of the hill (not inside)
                let player_top = player_aabb.max_y();
                let hill_top = hill_obstacle.max_y();
                if player_top >= hill_obstacle.min_y() && player_top <= hill_top + 1.0 {
                    current_holder = Some(i);
                    break;
                }
            }
        }

        // Update hill holder and score
        if let Some(holder) = current_holder {
            self.hill.ensure_scores(self.players.len());
            self.hill.register_holder(holder, dt);
        }

        // Update teleport timer
        if self.hill.update_teleport(dt) {
            // Remove old hill obstacle
            self.obstacles.retain(|o| {
                !((o.x - self.hill.x).abs() < 0.1 && (o.z - self.hill.z).abs() < 0.1 && (o.y - self.hill.y).abs() < 0.1)
            });

            // Pick new random position within arena bounds
            let (min_x, max_x, min_z, max_z) = crate::arena::arena_bounds();
            let t = self.hill.teleport_count as f32;
            let new_x = (min_x + 5.0) + (max_x - min_x - 10.0) * (t * 137.0_f32).sin().abs();
            let new_z = (min_z + 5.0) + (max_z - min_z - 10.0) * (t * 251.0_f32).cos().abs();
            let new_y = 0.5;
            self.hill.teleport(new_x, new_y, new_z);

            // Add new hill obstacle
            self.add_obstacle(self.hill.to_obstacle());
        }
    }

    /// Update power-up timers and check player collection.
    pub fn update_powerups(&mut self, dt: f32) {
        for pu in &mut self.powerups {
            pu.update(dt);
        }

        // Collect player positions first to avoid borrow conflicts
        let player_positions: Vec<(f32, f32, f32, f32)> = self.players.iter()
            .map(|p| {
                let pd = p.physics_data();
                (pd.x, pd.y, pd.z, pd.size)
            })
            .collect();

        for (i, (px, py, pz, psize)) in player_positions.iter().enumerate() {
            let player = &mut self.players[i];
            for pu in &mut self.powerups {
                if pu.check_collect(*px, *py, *pz, *psize) {
                    player.active_powerup = Some(pu.kind.clone());
                    player.powerup_timer = pu.kind.duration();
                    if pu.kind == PowerUpKind::DoubleJump {
                        player.double_jump_used = false;
                    }
                    if pu.kind == PowerUpKind::BiggerHill {
                        self.hill.width = self.hill.baseline_width * 2.0;
                        self.hill.depth = self.hill.baseline_depth * 2.0;
                    }
                    break;
                }
            }
        }
    }

    pub fn winning_player(&self) -> Option<usize> {
        self.hill.check_win()
    }

    pub fn get_hill(&self) -> &Hill {
        &self.hill
    }

    pub fn get_hill_mut(&mut self) -> &mut Hill {
        &mut self.hill
    }

    pub fn remove_entity(&mut self, index: usize) {
        self.players.remove(index);
    }

    pub fn entity_count(&self) -> usize {
        self.players.len()
    }

    pub fn obstacle_count(&self) -> usize {
        self.obstacles.len()
    }

    pub fn platform_count(&self) -> usize {
        self.obstacles.iter().filter(|o| o.kind == ObstacleKind::Platform).count()
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
    use crate::input::PlayerInput;
    use crate::obstacle::ObstacleKind;
    use crate::physics::Physics;

    fn default_input() -> PlayerInput {
        PlayerInput::default()
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
    fn test_world_with_arena() {
        let mut world = World::new();
        for obstacle in crate::arena::create_arena() {
            world.add_obstacle(obstacle);
        }
        assert_eq!(world.obstacle_count(), 14);
    }

    #[test]
    fn test_entity_updates_in_world() {
        let mut world = World::new();
        let mut player = Player::default();
        player.physics_data_mut().vel_x = 5.0;
        world.add_entity(player);

        let input = default_input();
        let physics = Physics::new();
        world.update_all(&[input], &physics, 0.016);

        assert!(world.players[0].physics_data().x > 0.0);
    }

    #[test]
    fn test_obstacle_collision_with_world() {
        let mut world = World::new();
        let mut player = Player::default();
        player.physics_data_mut().y = 5.0;
        player.physics_data_mut().vel_y = -10.0;
        world.add_entity(player);
        world.add_obstacle(Obstacle::solid(0.0, 0.0, 0.0, 10.0, 0.5, 10.0));

        let input = default_input();
        let physics = Physics::new();
        world.update_all(&[input], &physics, 0.016);

        let entity = &world.players[0];
        assert!(entity.physics_data().y >= 0.0);
    }

    #[test]
    fn test_multiple_entities_update() {
        let mut world = World::new();
        let mut player1 = Player::default();
        player1.physics_data_mut().vel_x = 5.0;
        world.add_entity(player1);

        let mut player2 = Player::default();
        player2.physics_data_mut().vel_z = -3.0;
        world.add_entity(player2);

        let input = default_input();
        let physics = Physics::new();
        world.update_all(&[input], &physics, 0.016);

        assert!(world.players[0].physics_data().x > 0.0);
        assert!(world.players[1].physics_data().z < 0.0);
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
        player.physics_data_mut().x = 3.0;
        player.physics_data_mut().vel_x = 2.0;
        player.physics_data_mut().y = 1.0;
        world.add_entity(player);
        // Solid wall at x=5
        world.add_obstacle(Obstacle::solid(5.0, 1.0, 0.0, 1.0, 2.0, 10.0));
        // Platform above (should not block horizontal movement)
        world.add_obstacle(Obstacle::platform(5.0, 3.0, 0.0, 4.0, 0.5, 10.0));

        let input = default_input();
        let physics = Physics::new();
        world.update_all(&[input], &physics, 0.016);

        let entity = &world.players[0];
        // Should hit the solid wall at x=5
        assert!(entity.physics_data().x < 5.5);
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

        assert!(world.projectiles[0].lifetime > 0.0);
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
        player.physics_data_mut().x = 5.0;
        player.physics_data_mut().y = 0.5;
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
        player.physics_data_mut().y = 10.0;
        world.add_entity(player);
        world.add_platform(0.0, -0.25, 0.0, 100.0, 0.5, 100.0);
        let projectile = Projectile::new(0.0, 1.0, 0.0, 10.0, 0.0, 0.0, 10.0);
        world.add_projectile(projectile);

        let input = default_input();
        let physics = Physics::new();
        world.update_all(&[input], &physics, 0.016);

        assert!(world.projectiles[0].physics_data().x > 0.0);
    }
}
