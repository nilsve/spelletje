/// Enemy system: autonomous enemies that chase and shoot at the player.

use crate::player::Player;
use crate::obstacle::Aabb;
use crate::projectile::Projectile;
use crate::physics::{PhysicsConfig, PhysicsData, PhysicsEntity};
use crate::shooter::Shooter;

#[derive(Clone, Debug)]
pub struct EnemyConfig {
    pub speed: f32,
    pub acceleration: f32,
    pub friction: f32,
    pub gravity: f32,
    pub jump_force: f32,
    pub friction_threshold: f32,
    pub health: f32,
    pub damage: f32,
    pub shoot_interval: f32,
    pub detection_range: f32,
    pub shoot_range: f32,
}

impl Default for EnemyConfig {
    fn default() -> Self {
        Self {
            speed: 3.0,
            acceleration: 10.0,
            friction: 3.0,
            gravity: 20.0,
            jump_force: 8.0,
            friction_threshold: 0.01,
            health: 50.0,
            damage: 10.0,
            shoot_interval: 2.0,
            detection_range: 20.0,
            shoot_range: 12.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Enemy {
    pub physics_data: PhysicsData,
    pub grounded: bool,
    pub config: EnemyConfig,
    pub alive: bool,
    pub shoot_timer: f32,
    pub last_player_pos: (f32, f32, f32),
}

impl Enemy {
    pub fn new(config: EnemyConfig) -> Self {
        Self {
            physics_data: PhysicsData::default(),
            grounded: true,
            config,
            alive: true,
            shoot_timer: 0.0,
            last_player_pos: (0.0, 0.0, 0.0),
        }
    }

    pub fn with_config(config: EnemyConfig) -> Self {
        Self {
            physics_data: PhysicsData::default(),
            grounded: true,
            config,
            alive: true,
            shoot_timer: 0.0,
            last_player_pos: (0.0, 0.0, 0.0),
        }
    }

    pub fn pos(&self) -> (f32, f32, f32) {
        (self.physics_data.x, self.physics_data.y, self.physics_data.z)
    }

    pub fn size(&self) -> f32 {
        self.physics_data.size
    }

    /// Returns the shoot direction toward the player.
    fn shoot_direction_toward(&self, target_x: f32, target_y: f32, target_z: f32) -> (f32, f32, f32) {
        let dx = target_x - self.physics_data.x;
        let dy = target_y - (self.physics_data.y + self.physics_data.size / 2.0);
        let dz = target_z - self.physics_data.z;
        let len = (dx * dx + dy * dy + dz * dz).sqrt().max(0.01);
        (dx / len, dy / len, dz / len)
    }

    /// Creates a projectile fired at the given target position.
    pub fn shoot_at(&self, target_x: f32, target_y: f32, target_z: f32) -> Projectile {
        let dir = self.shoot_direction_toward(target_x, target_y, target_z);
        self.fire_with_direction(dir)
    }

    /// Returns the shoot origin point for the enemy.
    fn shoot_origin(&self) -> (f32, f32, f32) {
        (self.physics_data.x, self.physics_data.y + self.physics_data.size / 2.0, self.physics_data.z)
    }

    /// Updates enemy AI: movement toward player and shooting.
    pub fn update_ai(&mut self, player: &Player, dt: f32) {
        if !self.alive {
            return;
        }

        let dx = player.physics_data.x - self.physics_data.x;
        let dy = player.physics_data.y - self.physics_data.y;
        let dz = player.physics_data.z - self.physics_data.z;
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();

        self.last_player_pos = (player.physics_data.x, player.physics_data.y, player.physics_data.z);

        // Movement toward player
        if dist > 2.0 && dist < self.config.detection_range {
            let target_x = player.physics_data.x;
            let target_z = player.physics_data.z;
            let move_dx = target_x - self.physics_data.x;
            let move_dz = target_z - self.physics_data.z;
            let move_len = (move_dx * move_dx + move_dz * move_dz).sqrt().max(0.01);
            let normalized_dx = move_dx / move_len;
            let normalized_dz = move_dz / move_len;

            let _pc = self.config.to_physics_config();
            
            // Horizontal acceleration toward player
            let input_x = normalized_dx;
            let input_z = normalized_dz;
            
            if input_x.abs() > 0.01 {
                self.physics_data.vel_x += input_x * self.config.acceleration * dt;
            }
            if input_z.abs() > 0.01 {
                self.physics_data.vel_z += input_z * self.config.acceleration * dt;
            }

            // Clamp speed
            let speed = (self.physics_data.vel_x * self.physics_data.vel_x + self.physics_data.vel_z * self.physics_data.vel_z).sqrt();
            if speed > self.config.speed {
                let scale = self.config.speed / speed;
                self.physics_data.vel_x *= scale;
                self.physics_data.vel_z *= scale;
            }

            // Jump if player is above
            if dy > 1.0 && self.grounded && dist < self.config.shoot_range * 1.5 {
                self.physics_data.vel_y = self.config.jump_force;
                self.grounded = false;
            }
        } else if dist <= self.config.shoot_range {
            // Strafe when in shooting range
            let strafe_angle = (self.last_player_pos.0 - self.last_player_pos.2).atan2(dist);
            let strafe_dx = f32::cos(strafe_angle + std::f32::consts::FRAC_PI_2) * 0.5;
            let strafe_dz = f32::sin(strafe_angle + std::f32::consts::FRAC_PI_2) * 0.5;
            
            let _pc = self.config.to_physics_config();
            self.physics_data.vel_x += strafe_dx * self.config.acceleration * dt;
            self.physics_data.vel_z += strafe_dz * self.config.acceleration * dt;
            
            let speed = (self.physics_data.vel_x * self.physics_data.vel_x + self.physics_data.vel_z * self.physics_data.vel_z).sqrt();
            if speed > self.config.speed * 0.5 {
                let scale = (self.config.speed * 0.5) / speed;
                self.physics_data.vel_x *= scale;
                self.physics_data.vel_z *= scale;
            }
        } else {
            // Apply friction when not moving
            self.physics_data.vel_x *= (1.0 - self.config.friction * dt).max(0.0);
            self.physics_data.vel_z *= (1.0 - self.config.friction * dt).max(0.0);
        }

        // Gravity
        self.physics_data.vel_y -= self.config.gravity * dt;

        // Ground check
        if self.physics_data.y <= self.config.friction_threshold && self.physics_data.vel_y <= 0.0 {
            self.physics_data.y = self.config.friction_threshold;
            self.physics_data.vel_y = 0.0;
            self.grounded = true;
        }

        // Update position
        self.update_position(dt);

        // Shooting timer
        self.shoot_timer += dt;
        if self.shoot_timer >= self.config.shoot_interval && dist < self.config.shoot_range {
            self.shoot_timer = 0.0;
        }
    }

    /// Checks if enemy should shoot and creates projectile if so.
    pub fn try_shoot(&self, player: &Player) -> Option<Projectile> {
        if !self.alive {
            return None;
        }
        
        let dx = player.physics_data.x - self.physics_data.x;
        let dy = player.physics_data.y + player.physics_data.size / 2.0 - (self.physics_data.y + self.physics_data.size / 2.0);
        let dz = player.physics_data.z - self.physics_data.z;
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();

        if self.shoot_timer >= self.config.shoot_interval && dist < self.config.shoot_range {
            Some(self.shoot_at(player.physics_data.x, player.physics_data.y + player.physics_data.size / 2.0, player.physics_data.z))
        } else {
            None
        }
    }

    /// Takes damage, dies if health reaches zero.
    pub fn take_damage(&mut self, damage: f32) {
        self.config.health -= damage;
        if self.config.health <= 0.0 {
            self.alive = false;
        }
    }

    pub fn to_physics_config(&self) -> PhysicsConfig {
        self.config.to_physics_config()
    }
}

impl PhysicsEntity for Enemy {
    fn physics_data(&self) -> &PhysicsData {
        &self.physics_data
    }

    fn physics_data_mut(&mut self) -> &mut PhysicsData {
        &mut self.physics_data
    }

    fn update_position(&mut self, dt: f32) {
        self.physics_data.x += self.physics_data.vel_x * dt;
        self.physics_data.y += self.physics_data.vel_y * dt;
        self.physics_data.z += self.physics_data.vel_z * dt;
    }
}

impl Shooter for Enemy {
    fn shoot_origin(&self) -> (f32, f32, f32) {
        (self.physics_data.x, self.physics_data.y + self.physics_data.size / 2.0, self.physics_data.z)
    }

    fn projectile_speed(&self) -> f32 {
        12.0
    }

    fn damage(&self) -> f32 {
        self.config.damage
    }
}

impl EnemyConfig {
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

impl Aabb for Enemy {
    fn min_x(&self) -> f32 { self.physics_data.x - self.physics_data.size / 2.0 }
    fn max_x(&self) -> f32 { self.physics_data.x + self.physics_data.size / 2.0 }
    fn min_y(&self) -> f32 { self.physics_data.y }
    fn max_y(&self) -> f32 { self.physics_data.y + self.physics_data.size }
    fn min_z(&self) -> f32 { self.physics_data.z - self.physics_data.size / 2.0 }
    fn max_z(&self) -> f32 { self.physics_data.z + self.physics_data.size / 2.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enemy_creation() {
        let enemy = Enemy::new(EnemyConfig::default());
        assert!(enemy.alive);
        assert!((enemy.physics_data.x - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_enemy_take_damage() {
        let mut enemy = Enemy::new(EnemyConfig::default());
        enemy.take_damage(20.0);
        assert!(enemy.alive);
        enemy.take_damage(30.0);
        assert!(!enemy.alive);
    }

    #[test]
    fn test_enemy_shoot_direction() {
        let enemy = Enemy::new(EnemyConfig::default());
        let dir = enemy.shoot_direction_toward(10.0, 0.0, 0.0);
        assert!(dir.0 > 0.0);
        let len = (dir.0 * dir.0 + dir.1 * dir.1 + dir.2 * dir.2).sqrt();
        assert!((len - 1.0).abs() < 0.01);
    }
}
