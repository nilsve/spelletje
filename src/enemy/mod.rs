use crate::PhysicsImpl;
use crate::obstacle::Aabb;
use crate::physics::{EntityPhysicsData, GlobalPhysicsConfig, PhysicsEntity};
/// Enemy system: autonomous enemies that chase and shoot at the player.
use crate::player::Player;
use crate::projectile::Projectile;
use crate::shooter::Shooter;

#[derive(Clone, Debug)]
pub struct Enemy {
    pub physics_data: EntityPhysicsData,
    pub alive: bool,
    pub shoot_timer: f32,
    pub shoot_interval: f32,
    shoot_range: f32,
    pub health: f32,
    pub damage: f32,
    detection_range: f32,
    jump_force: f32,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            physics_data: EntityPhysicsData::default(),
            alive: true,
            shoot_timer: 0.0,
            health: 10.0,
            damage: 1.0,
            shoot_range: 10.0,
            shoot_interval: 1.0,
            detection_range: 10.0,
            jump_force: 10.0,
        }
    }
}

impl Enemy {
    pub fn pos(&self) -> (f32, f32, f32) {
        (
            self.physics_data.x,
            self.physics_data.y,
            self.physics_data.z,
        )
    }

    pub fn size(&self) -> f32 {
        self.physics_data.size
    }

    /// Returns the shoot direction toward the player.
    fn shoot_direction_toward(
        &self,
        target_x: f32,
        target_y: f32,
        target_z: f32,
    ) -> (f32, f32, f32) {
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
        (
            self.physics_data.x,
            self.physics_data.y + self.physics_data.size / 2.0,
            self.physics_data.z,
        )
    }

    /// Updates enemy AI: movement toward player and shooting.
    pub fn update_ai(&mut self, player: &Player, dt: f32, physics: &PhysicsImpl) {
        if !self.alive {
            return;
        }

        let dx = player.physics_data.x - self.physics_data.x;
        let dy = player.physics_data.y - self.physics_data.y;
        let dist = (dx * dx + dy * dy).sqrt();

        // Movement toward player
        if dx.abs() > 2.0 && dx.abs() < self.detection_range {
            let target_x = player.physics_data.x;
            let move_dx = target_x - self.physics_data.x;
            let move_len = (move_dx * move_dx).sqrt().max(0.01);
            let normalized_dx = move_dx / move_len;

            // Horizontal acceleration toward player
            let input_x = normalized_dx;

            if input_x.abs() != 0.0 {
                physics.apply_acceleration_x(&mut self.physics_data, input_x, dt);
            }

            // Jump if player is above
            if dy > 1.0 && self.physics_data().is_grounded && dist < self.shoot_range * 1.5 {
                self.physics_data.vel_y = self.jump_force;
            }

            physics.clamp_speed(&mut self.physics_data.vel_x);
        } else {
            physics.apply_friction(&mut self.physics_data_mut().y, dt);
        }

        physics.apply_gravity(&mut self.physics_data_mut().vel_y, dt);

        // Update position
        self.update_position(dt);

        // // Shooting timer
        self.shoot_timer += dt;
        if self.shoot_timer >= self.shoot_interval && dist < self.shoot_range {
            self.shoot_timer = 0.0;
        }
    }

    /// Checks if enemy should shoot and creates projectile if so.
    pub fn try_shoot(&self, player: &Player) -> Option<Projectile> {
        if !self.alive {
            return None;
        }

        let dx = player.physics_data.x - self.physics_data.x;
        let dy = player.physics_data.y + player.physics_data.size / 2.0
            - (self.physics_data.y + self.physics_data.size / 2.0);
        let dz = player.physics_data.z - self.physics_data.z;
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();

        if self.shoot_timer >= self.shoot_interval && dist < self.shoot_range {
            Some(self.shoot_at(
                player.physics_data.x,
                player.physics_data.y + player.physics_data.size / 2.0,
                player.physics_data.z,
            ))
        } else {
            None
        }
    }

    /// Takes damage, dies if health reaches zero.
    pub fn take_damage(&mut self, damage: f32) {
        self.health -= damage;
        if self.health <= 0.0 {
            self.alive = false;
        }
    }
}

impl PhysicsEntity for Enemy {
    fn physics_data(&self) -> &EntityPhysicsData {
        &self.physics_data
    }

    fn physics_data_mut(&mut self) -> &mut EntityPhysicsData {
        &mut self.physics_data
    }
}

impl Shooter for Enemy {
    fn shoot_origin(&self) -> (f32, f32, f32) {
        (
            self.physics_data.x,
            self.physics_data.y + self.physics_data.size / 2.0,
            self.physics_data.z,
        )
    }

    fn projectile_speed(&self) -> f32 {
        12.0
    }

    fn damage(&self) -> f32 {
        self.damage
    }
}

impl Aabb for Enemy {
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

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_enemy_creation() {
    //     let enemy = Enemy::new(EnemyConfig::default());
    //     assert!(enemy.alive);
    //     assert!((enemy.physics_data.x - 0.0).abs() < 0.01);
    // }
    //
    // #[test]
    // fn test_enemy_take_damage() {
    //     let mut enemy = Enemy::new(EnemyConfig::default());
    //     enemy.take_damage(20.0);
    //     assert!(enemy.alive);
    //     enemy.take_damage(30.0);
    //     assert!(!enemy.alive);
    // }
    //
    // #[test]
    // fn test_enemy_shoot_direction() {
    //     let enemy = Enemy::new(EnemyConfig::default());
    //     let dir = enemy.shoot_direction_toward(10.0, 0.0, 0.0);
    //     assert!(dir.0 > 0.0);
    //     let len = (dir.0 * dir.0 + dir.1 * dir.1 + dir.2 * dir.2).sqrt();
    //     assert!((len - 1.0).abs() < 0.01);
    // }
}
