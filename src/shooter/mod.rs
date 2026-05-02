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

    /// Creates a projectile given a direction vector.
    /// This is the shared implementation used by both Player and Enemy.
    fn fire_with_direction(&self, dir: (f32, f32, f32)) -> Projectile {
        let origin = self.shoot_origin();
        let speed = self.projectile_speed();
        let damage = self.damage();
        Projectile::new(
            origin.0, origin.1, origin.2,
            dir.0 * speed, dir.1 * speed, dir.2 * speed,
            damage,
        )
    }
}
