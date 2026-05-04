/// Shooter trait: abstracts the common pattern of origin + direction + speed + damage → Projectile.
///
/// Both Player and Enemy share the same projectile creation logic:
/// 1. Compute an origin position
/// 2. Compute a direction vector
/// 3. Create a Projectile with configured speed and damage
///
/// The trait provides `fire_with_direction()` as the shared implementation.
/// Concrete types supply their own origin, speed, and damage.
use crate::projectile::Projectile;

pub trait Shooter {
    /// The 3D position where projectiles originate.
    fn shoot_origin(&self) -> (f32, f32, f32);

    /// Speed at which projectiles travel.
    fn projectile_speed(&self) -> f32;

    /// Damage dealt by projectiles.
    fn damage(&self) -> f32;

    /// Returns the normalized direction from shoot origin toward the target coordinate.
    fn get_direction(&self, target_coord: (f32, f32, f32)) -> (f32, f32, f32) {
        let (ox, oy, oz) = self.shoot_origin();
        let dx = target_coord.0 - ox;
        let dy = target_coord.1 - oy;
        let dz = target_coord.2 - oz;
        let len = (dx * dx + dy * dy + dz * dz).sqrt().max(0.01);
        (dx / len, dy / len, dz / len)
    }

    fn fire_at_direction(&self, direction: (f32, f32, f32)) -> Projectile {
        let origin = self.shoot_origin();
        let speed = self.projectile_speed();
        let damage = self.damage();

        let len = (direction.0 * direction.0 + direction.1 * direction.1 + direction.2 * direction.2).sqrt().max(0.01);
        Projectile::new(
            origin.0,
            origin.1,
            origin.2,
            direction.0 / len * speed,
            direction.1 / len * speed,
            direction.2 / len * speed,
            damage,
        )
    }

    fn fire_at_coord(&self, target_coord: (f32, f32, f32)) -> Projectile {
        let direction = self.get_direction(target_coord);

        self.fire_at_direction(direction)
    }
}
