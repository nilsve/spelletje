/// Obstacle: a static world object that entities collide with.
/// Can be a solid obstacle (wall/floor/ceiling) or a one-way platform.

pub trait Aabb {
    fn min_x(&self) -> f32;
    fn max_x(&self) -> f32;
    fn min_y(&self) -> f32;
    fn max_y(&self) -> f32;
    fn min_z(&self) -> f32;
    fn max_z(&self) -> f32;
}

/// Determines how entities interact with an obstacle.
#[derive(Clone, Debug, PartialEq)]
pub enum ObstacleKind {
    /// Solid: full AABB collision from all sides.
    /// Used for walls, floors, ceilings.
    Solid,
    /// Platform: one-way collision.
    /// Entity can land on top (when falling), jump from below.
    /// No horizontal collision.
    Platform,
}

/// Generic AABB collision check: returns true if two shapes overlap on all three axes.
pub fn is_colliding<T: Aabb, U: Aabb>(a: &T, b: &U) -> bool {
    a.max_x() >= b.min_x() && a.min_x() <= b.max_x()
        && a.max_y() >= b.min_y() && a.min_y() <= b.max_y()
        && a.max_z() >= b.min_z() && a.min_z() <= b.max_z()
}

#[derive(Clone, Debug)]
pub struct Obstacle {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub kind: ObstacleKind,
}

impl Obstacle {
    pub fn new(x: f32, y: f32, z: f32, width: f32, height: f32, depth: f32, kind: ObstacleKind) -> Self {
        Self { x, y, z, width, height, depth, kind }
    }

    pub fn solid(x: f32, y: f32, z: f32, width: f32, height: f32, depth: f32) -> Self {
        Self { x, y, z, width, height, depth, kind: ObstacleKind::Solid }
    }

    pub fn platform(x: f32, y: f32, z: f32, width: f32, height: f32, depth: f32) -> Self {
        Self { x, y, z, width, height, depth, kind: ObstacleKind::Platform }
    }

    pub fn bottom(&self) -> f32 {
        self.y - self.height / 2.0
    }

    pub fn top(&self) -> f32 {
        self.y + self.height / 2.0
    }

    pub fn left(&self) -> f32 {
        self.x - self.width / 2.0
    }

    pub fn right(&self) -> f32 {
        self.x + self.width / 2.0
    }

    pub fn front(&self) -> f32 {
        self.z - self.depth / 2.0
    }

    pub fn back(&self) -> f32 {
        self.z + self.depth / 2.0
    }
}

impl Aabb for Obstacle {
    fn min_x(&self) -> f32 {
        self.x - self.width / 2.0
    }
    fn max_x(&self) -> f32 {
        self.x + self.width / 2.0
    }
    fn min_y(&self) -> f32 {
        self.y - self.height / 2.0
    }
    fn max_y(&self) -> f32 {
        self.y + self.height / 2.0
    }
    fn min_z(&self) -> f32 {
        self.z - self.depth / 2.0
    }
    fn max_z(&self) -> f32 {
        self.z + self.depth / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solid_obstacle_creation() {
        let obstacle = Obstacle::solid(0.0, 0.0, 0.0, 4.0, 0.5, 4.0);
        assert_eq!(obstacle.kind, ObstacleKind::Solid);
        assert!((obstacle.x - 0.0).abs() < 0.01);
        assert!((obstacle.width - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_platform_obstacle_creation() {
        let obstacle = Obstacle::platform(0.0, 2.0, 0.0, 4.0, 0.5, 4.0);
        assert_eq!(obstacle.kind, ObstacleKind::Platform);
        assert!((obstacle.y - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_generic_new_with_kind() {
        let solid = Obstacle::new(0.0, 0.0, 0.0, 4.0, 1.0, 4.0, ObstacleKind::Solid);
        assert_eq!(solid.kind, ObstacleKind::Solid);

        let plat = Obstacle::new(0.0, 0.0, 0.0, 4.0, 1.0, 4.0, ObstacleKind::Platform);
        assert_eq!(plat.kind, ObstacleKind::Platform);
    }

    #[test]
    fn test_obstacle_bounds() {
        let obstacle = Obstacle::solid(0.0, 2.0, 0.0, 4.0, 1.0, 4.0);
        assert!((obstacle.min_x() - (-2.0)).abs() < 0.01);
        assert!((obstacle.max_x() - 2.0).abs() < 0.01);
        assert!((obstacle.min_y() - 1.5).abs() < 0.01);
        assert!((obstacle.max_y() - 2.5).abs() < 0.01);
        assert!((obstacle.min_z() - (-2.0)).abs() < 0.01);
        assert!((obstacle.max_z() - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_obstacle_top_and_bottom() {
        let obstacle = Obstacle::solid(0.0, 2.0, 0.0, 4.0, 1.0, 4.0);
        assert!((obstacle.bottom() - 1.5).abs() < 0.01);
        assert!((obstacle.top() - 2.5).abs() < 0.01);
    }

    #[test]
    fn test_is_colliding_overlapping() {
        let o1 = Obstacle::solid(0.0, 0.0, 0.0, 4.0, 1.0, 4.0);
        let o2 = Obstacle::solid(1.0, 0.0, 0.0, 4.0, 1.0, 4.0);
        assert!(is_colliding(&o1, &o2));
    }

    #[test]
    fn test_is_colliding_not_overlapping_x() {
        let o1 = Obstacle::solid(0.0, 0.0, 0.0, 2.0, 1.0, 4.0);
        let o2 = Obstacle::solid(3.0, 0.0, 0.0, 2.0, 1.0, 4.0);
        assert!(!is_colliding(&o1, &o2));
    }

    #[test]
    fn test_is_colliding_not_overlapping_y() {
        let o1 = Obstacle::solid(0.0, 0.0, 0.0, 4.0, 1.0, 4.0);
        let o2 = Obstacle::solid(0.0, 2.0, 0.0, 4.0, 1.0, 4.0);
        assert!(!is_colliding(&o1, &o2));
    }

    #[test]
    fn test_is_colliding_platform_and_solid() {
        let solid = Obstacle::solid(0.0, 0.0, 0.0, 4.0, 1.0, 4.0);
        let platform = Obstacle::platform(1.0, 0.0, 0.0, 2.0, 1.0, 4.0);
        assert!(is_colliding(&solid, &platform));
    }

    #[test]
    fn test_clone_obstacle() {
        let o1 = Obstacle::solid(0.0, 0.0, 0.0, 4.0, 1.0, 4.0);
        let o2 = o1.clone();
        assert_eq!(o1.kind, o2.kind);
        assert!((o1.x - o2.x).abs() < 0.01);
    }

    #[test]
    fn test_obstacle_debug() {
        let o = Obstacle::solid(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        let debug_str = format!("{:?}", o);
        assert!(debug_str.contains("Solid"));
    }
}
