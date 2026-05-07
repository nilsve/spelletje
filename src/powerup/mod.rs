use crate::obstacle::Aabb;

/// Types of power-ups collectible in the arena.
#[derive(Clone, Debug, PartialEq)]
pub enum PowerUpKind {
    /// 2x movement speed for duration.
    SpeedBoost,
    /// Can jump while airborne for duration.
    DoubleJump,
    /// Hill platform doubles in size for duration.
    BiggerHill,
    /// Absorbs one hit for duration.
    Shield,
}

impl PowerUpKind {
    /// Duration in seconds the power-up effect lasts.
    pub fn duration(&self) -> f32 {
        match self {
            PowerUpKind::SpeedBoost => 5.0,
            PowerUpKind::DoubleJump => 5.0,
            PowerUpKind::BiggerHill => 8.0,
            PowerUpKind::Shield => 10.0,
        }
    }

    /// Render color for the power-up cube.
    pub fn color(&self) -> (f32, f32, f32) {
        match self {
            PowerUpKind::SpeedBoost => (0.0, 1.0, 1.0),    // Cyan
            PowerUpKind::DoubleJump => (0.0, 1.0, 0.0),    // Green
            PowerUpKind::BiggerHill => (1.0, 1.0, 0.0),    // Gold
            PowerUpKind::Shield => (1.0, 1.0, 1.0),        // White
        }
    }
}

/// A collectible power-up item in the arena.
#[derive(Clone, Debug)]
pub struct PowerUp {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub kind: PowerUpKind,
    pub active: bool,
    /// Seconds remaining before this power-up despawns if not collected.
    pub lifetime: f32,
    /// Seconds until this spawn point produces a new power-up.
    pub respawn_timer: f32,
}

impl PowerUp {
    pub fn new(x: f32, y: f32, z: f32, kind: PowerUpKind) -> Self {
        Self {
            x,
            y,
            z,
            kind,
            active: true,
            lifetime: 15.0,
            respawn_timer: 0.0,
        }
    }

    /// Update lifetime and respawn timer. Returns true if collected.
    pub fn update(&mut self, dt: f32) {
        if self.active {
            self.lifetime -= dt;
            if self.lifetime <= 0.0 {
                self.active = false;
                self.respawn_timer = 15.0;
            }
        } else {
            self.respawn_timer -= dt;
            if self.respawn_timer <= 0.0 {
                self.active = true;
                self.lifetime = 15.0;
                self.kind = random_powerup_kind();
            }
        }
    }

    /// Check if an entity at the given position collects this power-up.
    pub fn check_collect(&mut self, entity_x: f32, entity_y: f32, entity_z: f32, entity_size: f32) -> bool {
        if !self.active {
            return false;
        }
        let half = entity_size / 2.0;
        let dx = (entity_x - self.x).abs();
        let dy = (entity_y - self.y).abs();
        let dz = (entity_z - self.z).abs();
        if dx < half + 0.5 && dy < half + 0.5 && dz < half + 0.5 {
            self.active = false;
            self.respawn_timer = 15.0;
            true
        } else {
            false
        }
    }
}

impl Aabb for PowerUp {
    fn min_x(&self) -> f32 { self.x - 0.5 }
    fn max_x(&self) -> f32 { self.x + 0.5 }
    fn min_y(&self) -> f32 { self.y - 0.5 }
    fn max_y(&self) -> f32 { self.y + 0.5 }
    fn min_z(&self) -> f32 { self.z - 0.5 }
    fn max_z(&self) -> f32 { self.z + 0.5 }
}

/// Returns the 4 corner spawn positions for power-ups.
pub fn spawn_positions() -> [(f32, f32, f32); 4] {
    [
        (-20.0, 0.5, -15.0),  // Back-left corner
        (20.0, 0.5, -15.0),   // Back-right corner
        (-20.0, 0.5, 15.0),   // Front-left corner
        (20.0, 0.5, 15.0),    // Front-right corner
    ]
}

/// Creates initial power-up spawn points (all inactive, ready to spawn).
pub fn create_powerups() -> Vec<PowerUp> {
    let positions = spawn_positions();
    let kinds = [
        PowerUpKind::SpeedBoost,
        PowerUpKind::DoubleJump,
        PowerUpKind::BiggerHill,
        PowerUpKind::Shield,
    ];
    positions.iter().zip(kinds.iter()).enumerate().map(|(i, (pos, kind))| {
        let mut pu = PowerUp::new(pos.0, pos.1, pos.2, kind.clone());
        pu.active = false;
        pu.respawn_timer = (i + 1) as f32 * 2.5; // Staggered initial spawns
        pu
    }).collect()
}

/// Returns a random power-up kind based on a pseudo-random value.
fn random_powerup_kind() -> PowerUpKind {
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let val = ms.wrapping_add(n.wrapping_mul(6364136223846793005)) >> 33;
    match val % 4 {
        0 => PowerUpKind::SpeedBoost,
        1 => PowerUpKind::DoubleJump,
        2 => PowerUpKind::BiggerHill,
        _ => PowerUpKind::Shield,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speedboost_duration() {
        assert!((PowerUpKind::SpeedBoost.duration() - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_doublejump_duration() {
        assert!((PowerUpKind::DoubleJump.duration() - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_biggerhill_duration() {
        assert!((PowerUpKind::BiggerHill.duration() - 8.0).abs() < 0.01);
    }

    #[test]
    fn test_shield_duration() {
        assert!((PowerUpKind::Shield.duration() - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_powerup_new_is_active() {
        let pu = PowerUp::new(0.0, 1.0, 0.0, PowerUpKind::SpeedBoost);
        assert!(pu.active);
        assert!((pu.lifetime - 15.0).abs() < 0.01);
        assert!((pu.respawn_timer - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_powerup_despawns_after_lifetime() {
        let mut pu = PowerUp::new(0.0, 1.0, 0.0, PowerUpKind::SpeedBoost);
        pu.update(15.0);
        assert!(!pu.active);
        assert!((pu.respawn_timer - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_powerup_respawns_after_timer() {
        let mut pu = PowerUp::new(0.0, 1.0, 0.0, PowerUpKind::SpeedBoost);
        pu.update(15.0); // despawn
        pu.update(15.0); // respawn
        assert!(pu.active);
        assert!((pu.lifetime - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_powerup_collect_nearby() {
        let mut pu = PowerUp::new(0.0, 1.0, 0.0, PowerUpKind::SpeedBoost);
        let collected = pu.check_collect(0.0, 1.0, 0.0, 1.0);
        assert!(collected);
        assert!(!pu.active);
    }

    #[test]
    fn test_powerup_not_collect_far() {
        let mut pu = PowerUp::new(0.0, 1.0, 0.0, PowerUpKind::SpeedBoost);
        let collected = pu.check_collect(10.0, 1.0, 10.0, 1.0);
        assert!(!collected);
        assert!(pu.active);
    }

    #[test]
    fn test_powerup_not_collect_when_inactive() {
        let mut pu = PowerUp::new(0.0, 1.0, 0.0, PowerUpKind::SpeedBoost);
        pu.active = false;
        let collected = pu.check_collect(0.0, 1.0, 0.0, 1.0);
        assert!(!collected);
    }

    #[test]
    fn test_spawn_positions_count() {
        let positions = spawn_positions();
        assert_eq!(positions.len(), 4);
    }

    #[test]
    fn test_create_powerups_count() {
        let powerups = create_powerups();
        assert_eq!(powerups.len(), 4);
    }

    #[test]
    fn test_create_powerups_all_inactive() {
        let powerups = create_powerups();
        for pu in &powerups {
            assert!(!pu.active);
        }
    }

    #[test]
    fn test_aabb_bounds() {
        let pu = PowerUp::new(5.0, 2.0, 3.0, PowerUpKind::Shield);
        assert!((pu.min_x() - 4.5).abs() < 0.01);
        assert!((pu.max_x() - 5.5).abs() < 0.01);
        assert!((pu.min_y() - 1.5).abs() < 0.01);
        assert!((pu.max_y() - 2.5).abs() < 0.01);
        assert!((pu.min_z() - 2.5).abs() < 0.01);
        assert!((pu.max_z() - 3.5).abs() < 0.01);
    }

    #[test]
    fn test_color_speedboost_cyan() {
        let (r, g, b) = PowerUpKind::SpeedBoost.color();
        assert!((r - 0.0).abs() < 0.01);
        assert!((g - 1.0).abs() < 0.01);
        assert!((b - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_color_shield_white() {
        let (r, g, b) = PowerUpKind::Shield.color();
        assert!((r - 1.0).abs() < 0.01);
        assert!((g - 1.0).abs() < 0.01);
        assert!((b - 1.0).abs() < 0.01);
    }
}
