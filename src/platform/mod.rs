/// Platform entity: static or movable surfaces for players to stand on.
/// Axis-Aligned Bounding Box trait for generic collision detection.

pub trait Aabb {
    fn min_x(&self) -> f32;
    fn max_x(&self) -> f32;
    fn min_y(&self) -> f32;
    fn max_y(&self) -> f32;
    fn min_z(&self) -> f32;
    fn max_z(&self) -> f32;
}

/// Generic AABB collision check: returns true if two shapes overlap on all three axes.
pub fn is_colliding<T: Aabb, U: Aabb>(a: &T, b: &U) -> bool {
    a.max_x() >= b.min_x() && a.min_x() <= b.max_x()
        && a.max_y() >= b.min_y() && a.min_y() <= b.max_y()
        && a.max_z() >= b.min_z() && a.min_z() <= b.max_z()
}

#[derive(Clone, Debug)]
pub struct Platform {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub is_static: bool,
}

impl Platform {
    pub fn new(x: f32, y: f32, z: f32, width: f32, height: f32, depth: f32) -> Self {
        Self {
            x,
            y,
            z,
            width,
            height,
            depth,
            is_static: true,
        }
    }

    pub fn with_static(x: f32, y: f32, z: f32, width: f32, height: f32, depth: f32, static_: bool) -> Self {
        Self {
            x,
            y,
            z,
            width,
            height,
            depth,
            is_static: static_,
        }
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

impl Aabb for Platform {
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
    fn test_platform_creation() {
        let platform = Platform::new(0.0, 0.0, 0.0, 4.0, 0.5, 4.0);
        assert!((platform.x - 0.0).abs() < 0.01);
        assert!((platform.width - 4.0).abs() < 0.01);
        assert!(platform.is_static);
    }

    #[test]
    fn test_platform_bottom_and_top() {
        let platform = Platform::new(0.0, 2.0, 0.0, 4.0, 1.0, 4.0);
        assert!((platform.bottom() - 1.5).abs() < 0.01);
        assert!((platform.top() - 2.5).abs() < 0.01);
    }

    #[test]
    fn test_platform_left_and_right() {
        let platform = Platform::new(5.0, 0.0, 0.0, 4.0, 1.0, 4.0);
        assert!((platform.left() - 3.0).abs() < 0.01);
        assert!((platform.right() - 7.0).abs() < 0.01);
    }

    #[test]
    fn test_platform_front_and_back() {
        let platform = Platform::new(0.0, 0.0, 10.0, 4.0, 1.0, 4.0);
        assert!((platform.front() - 8.0).abs() < 0.01);
        assert!((platform.back() - 12.0).abs() < 0.01);
    }

    #[test]
    fn test_static_platform() {
        let platform = Platform::with_static(0.0, 0.0, 0.0, 4.0, 1.0, 4.0, true);
        assert!(platform.is_static);
    }

    #[test]
    fn test_movable_platform() {
        let platform = Platform::with_static(0.0, 0.0, 0.0, 4.0, 1.0, 4.0, false);
        assert!(!platform.is_static);
    }

    #[test]
    fn test_aabb_bounds() {
        let platform = Platform::new(0.0, 0.0, 0.0, 4.0, 1.0, 4.0);
        assert!((platform.min_x() - (-2.0)).abs() < 0.01);
        assert!((platform.max_x() - 2.0).abs() < 0.01);
        assert!((platform.min_y() - (-0.5)).abs() < 0.01);
        assert!((platform.max_y() - 0.5).abs() < 0.01);
        assert!((platform.min_z() - (-2.0)).abs() < 0.01);
        assert!((platform.max_z() - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_is_colliding_overlapping() {
        let p1 = Platform::new(0.0, 0.0, 0.0, 4.0, 1.0, 4.0);
        let p2 = Platform::new(1.0, 0.0, 0.0, 4.0, 1.0, 4.0);
        assert!(is_colliding(&p1, &p2));
    }

    #[test]
    fn test_is_colliding_not_overlapping_x() {
        let p1 = Platform::new(0.0, 0.0, 0.0, 2.0, 1.0, 4.0);
        let p2 = Platform::new(3.0, 0.0, 0.0, 2.0, 1.0, 4.0);
        assert!(!is_colliding(&p1, &p2));
    }

    #[test]
    fn test_is_colliding_not_overlapping_y() {
        let p1 = Platform::new(0.0, 0.0, 0.0, 4.0, 1.0, 4.0);
        let p2 = Platform::new(0.0, 2.0, 0.0, 4.0, 1.0, 4.0);
        assert!(!is_colliding(&p1, &p2));
    }

    #[test]
    fn test_is_colliding_not_overlapping_z() {
        let p1 = Platform::new(0.0, 0.0, 0.0, 4.0, 1.0, 4.0);
        let p2 = Platform::new(0.0, 0.0, 4.1, 4.0, 1.0, 4.0);
        assert!(!is_colliding(&p1, &p2));
    }
}
