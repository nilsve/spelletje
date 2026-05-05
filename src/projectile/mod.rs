/// Projectile system for the game.
/// Projectiles are entities that move through the air and can collide with platforms and players.
use crate::obstacle::Aabb;
use crate::physics::{EntityPhysicsData, PhysicsEntity};

#[derive(Clone, Debug)]
pub struct Projectile {
    pub physics_data: EntityPhysicsData,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub damage: f32,
}

impl Aabb for Projectile {
    fn min_x(&self) -> f32 {
        self.physics_data.x - 0.1
    }
    fn max_x(&self) -> f32 {
        self.physics_data.x + 0.1
    }
    fn min_y(&self) -> f32 {
        self.physics_data.y - 0.1
    }
    fn max_y(&self) -> f32 {
        self.physics_data.y + 0.1
    }
    fn min_z(&self) -> f32 {
        self.physics_data.z - 0.1
    }
    fn max_z(&self) -> f32 {
        self.physics_data.z + 0.1
    }
}

impl Projectile {
    pub fn new(x: f32, y: f32, z: f32, vel_x: f32, vel_y: f32, vel_z: f32, damage: f32) -> Self {
        let speed = (vel_x * vel_x + vel_y * vel_y + vel_z * vel_z)
            .sqrt()
            .max(0.1);
        let max_lifetime = speed; // / 10.0;
        Self {
            physics_data: EntityPhysicsData {
                x,
                y,
                z,
                vel_x,
                vel_y,
                vel_z,
                size: 0.1,
                is_grounded: false,
            },
            lifetime: 0.0,
            max_lifetime,
            damage,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.lifetime < self.max_lifetime
    }

    pub fn update(&mut self, dt: f32) {
        if !self.is_alive() {
            return;
        }
        self.lifetime += dt;
        self.physics_data.x += self.physics_data.vel_x * dt;
        self.physics_data.y += self.physics_data.vel_y * dt;
        self.physics_data.z += self.physics_data.vel_z * dt;
    }

    /// Returns the position of the projectile.
    pub fn position(&self) -> (f32, f32, f32) {
        (
            self.physics_data.x,
            self.physics_data.y,
            self.physics_data.z,
        )
    }

    pub fn size(&self) -> f32 {
        self.physics_data.size
    }
}

impl PhysicsEntity for Projectile {
    fn physics_data(&self) -> &EntityPhysicsData {
        &self.physics_data
    }
    fn physics_data_mut(&mut self) -> &mut EntityPhysicsData {
        &mut self.physics_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_projectile_starts_with_zero_lifetime() {
        let projectile = Projectile::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 10.0);
        assert!((projectile.lifetime - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_new_projectile_is_alive() {
        let projectile = Projectile::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 10.0);
        assert!(projectile.is_alive());
    }

    #[test]
    fn test_projectile_update_moves_position() {
        let mut projectile = Projectile::new(0.0, 0.0, 0.0, 5.0, 0.0, 0.0, 10.0);
        let dt = 0.016;

        projectile.update(dt);

        assert!((projectile.physics_data.x - 0.08).abs() < 0.01);
        assert!((projectile.physics_data.y - 0.0).abs() < 0.01);
        assert!((projectile.physics_data.z - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_projectile_update_accumulates_lifetime() {
        let mut projectile = Projectile::new(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 10.0);
        let dt = 0.016;

        projectile.update(dt);

        assert!(projectile.lifetime > 0.0);
    }

    #[test]
    fn test_projectile_dies_after_max_lifetime() {
        let mut projectile = Projectile::new(0.0, 0.0, 0.0, 10.0, 0.0, 0.0, 10.0);
        let dt = 0.1;

        while projectile.is_alive() {
            projectile.update(dt);
        }

        assert!(!projectile.is_alive());
    }

    #[test]
    fn test_projectile_does_not_update_when_dead() {
        let mut projectile = Projectile::new(0.0, 0.0, 0.0, 10.0, 0.0, 0.0, 10.0);
        let dt = 0.1;

        while projectile.is_alive() {
            projectile.update(dt);
        }

        let x_before = projectile.physics_data.x;
        let y_before = projectile.physics_data.y;
        let z_before = projectile.physics_data.z;

        projectile.update(dt);

        assert!((projectile.physics_data.x - x_before).abs() < 0.001);
        assert!((projectile.physics_data.y - y_before).abs() < 0.001);
        assert!((projectile.physics_data.z - z_before).abs() < 0.001);
    }

    #[test]
    fn test_projectile_damage_is_stored() {
        let projectile = Projectile::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 25.0);
        assert!((projectile.damage - 25.0).abs() < 0.01);
    }
}
