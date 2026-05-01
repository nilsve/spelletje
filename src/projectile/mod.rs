/// Projectile system for the game.
/// Projectiles are entities that move through the air and can collide with platforms and players.

#[derive(Clone, Debug)]
pub struct Projectile {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub vel_z: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub damage: f32,
}

impl Projectile {
    pub fn new(x: f32, y: f32, z: f32, vel_x: f32, vel_y: f32, vel_z: f32, damage: f32) -> Self {
        let max_lifetime = (vel_x.abs() + vel_y.abs() + vel_z.abs()).max(0.1) / 10.0;
        Self {
            x,
            y,
            z,
            vel_x,
            vel_y,
            vel_z,
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
        self.x += self.vel_x * dt;
        self.y += self.vel_y * dt;
        self.z += self.vel_z * dt;
    }

    /// Check if projectile collides with a platform.
    /// Returns true if the projectile hit the platform (and should be removed).
    pub fn resolve_platform_collision(
        &mut self,
        platform_left: f32,
        platform_right: f32,
        platform_bottom: f32,
        platform_top: f32,
        platform_front: f32,
        platform_back: f32,
    ) -> bool {
        let half_size = 0.1;
        let proj_left = self.x - half_size;
        let proj_right = self.x + half_size;
        let proj_bottom = self.y - half_size;
        let proj_top = self.y + half_size;
        let proj_front = self.z - half_size;
        let proj_back = self.z + half_size;

        // Check Y overlap
        if proj_top < platform_bottom || proj_bottom > platform_top {
            return false;
        }

        // Check Z overlap
        if proj_back < platform_front || proj_front > platform_back {
            return false;
        }

        // Check X overlap
        if proj_left >= platform_right || proj_right <= platform_left {
            return false;
        }

        true
    }

    /// Check if projectile collides with a player.
    /// Returns true if the projectile hit the player.
    pub fn resolve_player_collision(
        &self,
        player_x: f32,
        player_y: f32,
        player_z: f32,
        player_half_size: f32,
    ) -> bool {
        let half_size = 0.1;
        let proj_left = self.x - half_size;
        let proj_right = self.x + half_size;
        let proj_bottom = self.y - half_size;
        let proj_top = self.y + half_size;
        let proj_front = self.z - half_size;
        let proj_back = self.z + half_size;

        let player_left = player_x - player_half_size;
        let player_right = player_x + player_half_size;
        let player_bottom = player_y - player_half_size;
        let player_top = player_y + player_half_size;
        let player_front = player_z - player_half_size;
        let player_back = player_z + player_half_size;

        if proj_top < player_bottom || proj_bottom > player_top {
            return false;
        }

        if proj_back < player_front || proj_front > player_back {
            return false;
        }

        if proj_left >= player_right || proj_right <= player_left {
            return false;
        }

        true
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

        assert!((projectile.x - 0.08).abs() < 0.01);
        assert!((projectile.y - 0.0).abs() < 0.01);
        assert!((projectile.z - 0.0).abs() < 0.01);
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

        let x_before = projectile.x;
        let y_before = projectile.y;
        let z_before = projectile.z;

        projectile.update(dt);

        assert!((projectile.x - x_before).abs() < 0.001);
        assert!((projectile.y - y_before).abs() < 0.001);
        assert!((projectile.z - z_before).abs() < 0.001);
    }

    #[test]
    fn test_projectile_damage_is_stored() {
        let projectile = Projectile::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 25.0);
        assert!((projectile.damage - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_projectile_platform_collision_top() {
        let mut projectile = Projectile::new(0.0, 2.15, 0.0, 0.0, -5.0, 0.0, 10.0);
        let platform_left = -5.0;
        let platform_right = 5.0;
        let platform_bottom = 2.0;
        let platform_top = 2.1;
        let platform_front = -5.0;
        let platform_back = 5.0;

        let hit = projectile.resolve_platform_collision(
            platform_left,
            platform_right,
            platform_bottom,
            platform_top,
            platform_front,
            platform_back,
        );

        assert!(hit);
    }

    #[test]
    fn test_projectile_platform_collision_no_hit_above() {
        let mut projectile = Projectile::new(0.0, 1.0, 0.0, 0.0, 5.0, 0.0, 10.0);
        let platform_left = -5.0;
        let platform_right = 5.0;
        let platform_bottom = 2.0;
        let platform_top = 2.1;
        let platform_front = -5.0;
        let platform_back = 5.0;

        let hit = projectile.resolve_platform_collision(
            platform_left,
            platform_right,
            platform_bottom,
            platform_top,
            platform_front,
            platform_back,
        );

        assert!(!hit);
    }

    #[test]
    fn test_projectile_platform_collision_no_hit_outside_x() {
        let mut projectile = Projectile::new(10.0, 3.0, 0.0, 0.0, -5.0, 0.0, 10.0);
        let platform_left = -5.0;
        let platform_right = 5.0;
        let platform_bottom = 2.0;
        let platform_top = 2.1;
        let platform_front = -5.0;
        let platform_back = 5.0;

        let hit = projectile.resolve_platform_collision(
            platform_left,
            platform_right,
            platform_bottom,
            platform_top,
            platform_front,
            platform_back,
        );

        assert!(!hit);
    }

    #[test]
    fn test_projectile_platform_collision_no_hit_outside_z() {
        let mut projectile = Projectile::new(0.0, 3.0, 10.0, 0.0, -5.0, 0.0, 10.0);
        let platform_left = -5.0;
        let platform_right = 5.0;
        let platform_bottom = 2.0;
        let platform_top = 2.1;
        let platform_front = -5.0;
        let platform_back = 5.0;

        let hit = projectile.resolve_platform_collision(
            platform_left,
            platform_right,
            platform_bottom,
            platform_top,
            platform_front,
            platform_back,
        );

        assert!(!hit);
    }

    #[test]
    fn test_projectile_player_collision_hits() {
        let mut projectile = Projectile::new(0.5, 0.5, 0.0, 5.0, 0.0, 0.0, 10.0);
        let player_x = 1.0;
        let player_y = 0.5;
        let player_z = 0.0;
        let player_half_size = 0.5;

        let hit = projectile.resolve_player_collision(
            player_x, player_y, player_z, player_half_size,
        );

        assert!(hit);
    }

    #[test]
    fn test_projectile_player_collision_misses() {
        let mut projectile = Projectile::new(0.0, 0.0, 0.0, 5.0, 0.0, 0.0, 10.0);
        let player_x = 10.0;
        let player_y = 0.5;
        let player_z = 0.0;
        let player_half_size = 0.5;

        let hit = projectile.resolve_player_collision(
            player_x, player_y, player_z, player_half_size,
        );

        assert!(!hit);
    }
}
