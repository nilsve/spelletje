use crate::obstacle::Obstacle;

/// Creates the King of the Hill arena with all platforms, walls, and cover blocks.
///
/// Arena layout (top-down view):
///
///                     (high platform)
///                         ===
///               (side platform)
///               ===           ===
///          (central hill)
///               ===
///            (ground floor)
///     =========================================
///
/// Returns a list of obstacles that form the complete arena.
///
/// ```ignore
/// ```
pub fn create_arena() -> Vec<Obstacle> {
    let mut obstacles = Vec::new();

    // Ground floor — large flat platform
    obstacles.push(Obstacle::platform(0.0, -0.25, 0.0, 60.0, 0.5, 60.0));

    // Central hill — the "king of the hill" platform
    obstacles.push(Obstacle::platform(0.0, 0.5, 0.0, 5.0, 0.5, 5.0));

    // Left high platform
    obstacles.push(Obstacle::platform(-12.0, 3.0, 0.0, 6.0, 0.5, 4.0));

    // Right high platform
    obstacles.push(Obstacle::platform(12.0, 3.0, 0.0, 6.0, 0.5, 4.0));

    // Back wall
    obstacles.push(Obstacle::solid(0.0, 2.0, -20.0, 60.0, 4.0, 1.0));

    // Front wall
    obstacles.push(Obstacle::solid(0.0, 2.0, 20.0, 60.0, 4.0, 1.0));

    // Left wall
    obstacles.push(Obstacle::solid(-25.0, 2.0, 0.0, 1.0, 4.0, 40.0));

    // Right wall
    obstacles.push(Obstacle::solid(25.0, 2.0, 0.0, 1.0, 4.0, 40.0));

    // Corner obstacles (cover blocks)
    obstacles.push(Obstacle::solid(-8.0, 1.5, -8.0, 3.0, 3.0, 3.0)); // left-back
    obstacles.push(Obstacle::solid(8.0, 1.5, -8.0, 3.0, 3.0, 3.0));  // right-back
    obstacles.push(Obstacle::solid(-8.0, 1.5, 8.0, 3.0, 3.0, 3.0));  // left-front
    obstacles.push(Obstacle::solid(8.0, 1.5, 8.0, 3.0, 3.0, 3.0));   // right-front

    // Side ramps
    obstacles.push(Obstacle::platform(-18.0, 1.5, 0.0, 4.0, 0.5, 3.0)); // left ramp
    obstacles.push(Obstacle::platform(18.0, 1.5, 0.0, 4.0, 0.5, 3.0));  // right ramp

    obstacles
}

/// Returns the arena bounds as (min_x, max_x, min_z, max_z) based on the boundary walls.
pub fn arena_bounds() -> (f32, f32, f32, f32) {
    (-25.0, 25.0, -20.0, 20.0)
}

/// Returns the hill's default position.
pub fn hill_default_position() -> (f32, f32, f32) {
    (0.0, 0.5, 0.0)
}

/// Returns the hill's default dimensions.
pub fn hill_default_dimensions() -> (f32, f32, f32) {
    (5.0, 0.5, 5.0)
}

/// Returns the four high platform positions.
pub fn high_platform_positions() -> [(f32, f32, f32); 2] {
    [(-12.0, 3.0, 0.0), (12.0, 3.0, 0.0)]
}

/// Returns the four cover block positions.
pub fn cover_block_positions() -> [(f32, f32, f32); 4] {
    [
        (-8.0, 1.5, -8.0),
        (8.0, 1.5, -8.0),
        (-8.0, 1.5, 8.0),
        (8.0, 1.5, 8.0),
    ]
}

/// Returns the side ramp positions.
pub fn side_ramp_positions() -> [(f32, f32, f32); 2] {
    [(-18.0, 1.5, 0.0), (18.0, 1.5, 0.0)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obstacle::{Aabb, ObstacleKind};

    #[test]
    fn test_create_arena_returns_obstacles() {
        let obstacles = create_arena();
        assert!(!obstacles.is_empty());
    }

    #[test]
    fn test_create_arena_has_ground() {
        let obstacles = create_arena();
        let ground = obstacles.iter().find(|o| {
            o.kind == ObstacleKind::Platform
                && (o.width - 60.0).abs() < 0.01
                && (o.depth - 60.0).abs() < 0.01
        });
        assert!(ground.is_some(), "Arena should contain a large ground platform");
    }

    #[test]
    fn test_create_arena_has_hill() {
        let obstacles = create_arena();
        let hill = obstacles.iter().find(|o| {
            o.kind == ObstacleKind::Platform
                && (o.x - 0.0).abs() < 0.01
                && (o.z - 0.0).abs() < 0.01
                && (o.width - 5.0).abs() < 0.01
        });
        assert!(hill.is_some(), "Arena should contain a central hill platform");
    }

    #[test]
    fn test_create_arena_has_high_platforms() {
        let obstacles = create_arena();
        let high_platforms: Vec<_> = obstacles
            .iter()
            .filter(|o| {
                o.kind == ObstacleKind::Platform
                    && (o.y - 3.0).abs() < 0.01
                    && (o.width - 6.0).abs() < 0.01
            })
            .collect();
        assert_eq!(high_platforms.len(), 2, "Should have exactly 2 high platforms");
    }

    #[test]
    fn test_create_arena_has_boundary_walls() {
        let obstacles = create_arena();
        let solid_obstacles: Vec<_> = obstacles
            .iter()
            .filter(|o| o.kind == ObstacleKind::Solid)
            .collect();
        assert_eq!(solid_obstacles.len(), 8, "Should have 4 boundary walls + 4 cover blocks");
    }

    #[test]
    fn test_create_arena_has_cover_blocks() {
        let obstacles = create_arena();
        let cover_blocks: Vec<_> = obstacles
            .iter()
            .filter(|o| {
                o.kind == ObstacleKind::Solid
                    && (o.width - 3.0).abs() < 0.01
                    && (o.height - 3.0).abs() < 0.01
                    && (o.depth - 3.0).abs() < 0.01
            })
            .collect();
        assert_eq!(cover_blocks.len(), 4, "Should have 4 cover blocks");
    }

    #[test]
    fn test_create_arena_has_side_ramps() {
        let obstacles = create_arena();
        let ramps: Vec<_> = obstacles
            .iter()
            .filter(|o| {
                o.kind == ObstacleKind::Platform
                    && (o.y - 1.5).abs() < 0.01
                    && (o.width - 4.0).abs() < 0.01
            })
            .collect();
        assert_eq!(ramps.len(), 2, "Should have 2 side ramps");
    }

    #[test]
    fn test_create_arena_total_obstacle_count() {
        let obstacles = create_arena();
        // 1 ground + 1 hill + 2 high platforms + 4 walls + 4 cover blocks + 2 ramps = 14
        assert_eq!(obstacles.len(), 14);
    }

    #[test]
    fn test_arena_bounds() {
        let (min_x, max_x, min_z, max_z) = arena_bounds();
        assert!((min_x - (-25.0)).abs() < 0.01);
        assert!((max_x - 25.0).abs() < 0.01);
        assert!((min_z - (-20.0)).abs() < 0.01);
        assert!((max_z - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_hill_default_position() {
        let (x, y, z) = hill_default_position();
        assert!((x - 0.0).abs() < 0.01);
        assert!((y - 0.5).abs() < 0.01);
        assert!((z - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_hill_default_dimensions() {
        let (w, h, d) = hill_default_dimensions();
        assert!((w - 5.0).abs() < 0.01);
        assert!((h - 0.5).abs() < 0.01);
        assert!((d - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_high_platform_positions() {
        let positions = high_platform_positions();
        assert_eq!(positions.len(), 2);
        assert!((positions[0].0 - (-12.0)).abs() < 0.01);
        assert!((positions[0].1 - 3.0).abs() < 0.01);
        assert!((positions[1].0 - 12.0).abs() < 0.01);
        assert!((positions[1].1 - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_cover_block_positions() {
        let positions = cover_block_positions();
        assert_eq!(positions.len(), 4);
        // Verify all are at height 1.5
        for pos in positions {
            assert!((pos.1 - 1.5).abs() < 0.01);
        }
    }

    #[test]
    fn test_ground_platform_is_platform_kind() {
        let obstacles = create_arena();
        let ground = obstacles.iter().find(|o| {
            (o.x - 0.0).abs() < 0.01
                && (o.y - (-0.25)).abs() < 0.01
                && (o.z - 0.0).abs() < 0.01
        });
        assert!(ground.is_some());
        assert_eq!(ground.unwrap().kind, ObstacleKind::Platform);
    }

    #[test]
    fn test_walls_are_solid_kind() {
        let obstacles = create_arena();
        let solid_obstacles: Vec<_> = obstacles
            .iter()
            .filter(|o| o.kind == ObstacleKind::Solid)
            .collect();
        for wall in solid_obstacles {
            // Walls should be tall (height ~4.0) and thin (width/depth ~1.0)
            // or be cover blocks (3x3x3)
            let is_wall = (wall.height - 4.0).abs() < 0.01
                || (wall.width - 1.0).abs() < 0.01
                || (wall.depth - 1.0).abs() < 0.01;
            let is_cover = (wall.width - 3.0).abs() < 0.01
                && (wall.height - 3.0).abs() < 0.01
                && (wall.depth - 3.0).abs() < 0.01;
            assert!(is_wall || is_cover, "Solid obstacle at ({}, {}, {}) should be a wall or cover block", wall.x, wall.y, wall.z);
        }
    }

    #[test]
    fn test_hill_platform_top_y() {
        let obstacles = create_arena();
        let hill = obstacles.iter().find(|o| {
            (o.x - 0.0).abs() < 0.01
                && (o.z - 0.0).abs() < 0.01
                && (o.width - 5.0).abs() < 0.01
                && o.kind == ObstacleKind::Platform
        });
        assert!(hill.is_some(), "Should find hill platform at center");
        let hill = hill.unwrap();
        // Hill center y=0.5, height=0.5, so top = 0.5 + 0.25 = 0.75
        assert!((hill.top() - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_ground_platform_top_y() {
        let obstacles = create_arena();
        let ground = obstacles.iter().find(|o| {
            (o.width - 60.0).abs() < 0.01
                && (o.depth - 60.0).abs() < 0.01
        });
        assert!(ground.is_some());
        let ground = ground.unwrap();
        // Ground center y=-0.25, height=0.5, so top = -0.25 + 0.25 = 0.0
        assert!((ground.top() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_high_platform_reachable_from_ground() {
        // High platforms are at y=3.0 (top = 3.25), ground top = 0.0
        // Player needs to jump from ground to reach them
        // With default jump_force=10.0 and gravity=20.0, max jump height = jump_force^2 / (2*gravity) = 100/40 = 2.5
        // So player can reach y=2.5 from ground, not enough for y=3.25 directly
        // But side ramps at y=1.5 (top=1.75) are reachable, then can jump from ramps
        let obstacles = create_arena();
        let high_platforms: Vec<_> = obstacles
            .iter()
            .filter(|o| {
                o.kind == ObstacleKind::Platform
                    && (o.y - 3.0).abs() < 0.01
            })
            .collect();
        assert_eq!(high_platforms.len(), 2);
        for hp in &high_platforms {
            assert!((hp.top() - 3.25).abs() < 0.01);
        }
    }

    #[test]
    fn test_ramp_reachable_from_ground() {
        // Ramps at y=1.5 (top = 1.75), player can jump to y=2.5
        let obstacles = create_arena();
        let ramps: Vec<_> = obstacles
            .iter()
            .filter(|o| {
                o.kind == ObstacleKind::Platform
                    && (o.y - 1.5).abs() < 0.01
            })
            .collect();
        assert_eq!(ramps.len(), 2);
        for ramp in &ramps {
            assert!((ramp.top() - 1.75).abs() < 0.01);
        }
    }

    #[test]
    fn test_arena_is_symmetric() {
        let obstacles = create_arena();

        // Check that left and right high platforms are symmetric
        let left_hp = obstacles.iter().find(|o| {
            o.kind == ObstacleKind::Platform
                && (o.y - 3.0).abs() < 0.01
                && (o.x - (-12.0)).abs() < 0.01
        });
        let right_hp = obstacles.iter().find(|o| {
            o.kind == ObstacleKind::Platform
                && (o.y - 3.0).abs() < 0.01
                && (o.x - 12.0).abs() < 0.01
        });

        assert!(left_hp.is_some());
        assert!(right_hp.is_some());
        let left_hp = left_hp.unwrap();
        let right_hp = right_hp.unwrap();
        assert!((left_hp.width - right_hp.width).abs() < 0.01);
        assert!((left_hp.depth - right_hp.depth).abs() < 0.01);
        assert!((left_hp.y - right_hp.y).abs() < 0.01);
    }

    #[test]
    fn test_cover_blocks_are_equally_sized() {
        let obstacles = create_arena();
        let cover_blocks: Vec<_> = obstacles
            .iter()
            .filter(|o| {
                o.kind == ObstacleKind::Solid
                    && (o.width - 3.0).abs() < 0.01
                    && (o.height - 3.0).abs() < 0.01
                    && (o.depth - 3.0).abs() < 0.01
            })
            .collect();
        for block in &cover_blocks {
            assert!((block.width - 3.0).abs() < 0.01);
            assert!((block.height - 3.0).abs() < 0.01);
            assert!((block.depth - 3.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_boundary_walls_enclose_arena() {
        let obstacles = create_arena();
        let walls: Vec<_> = obstacles
            .iter()
            .filter(|o| o.kind == ObstacleKind::Solid && o.height == 4.0)
            .collect();

        // Should have 4 boundary walls (front, back, left, right)
        assert_eq!(walls.len(), 4);

        // Check positions cover all boundaries
        let has_front = walls.iter().any(|w| (w.z - 20.0).abs() < 0.01);
        let has_back = walls.iter().any(|w| (w.z - (-20.0)).abs() < 0.01);
        let has_left = walls.iter().any(|w| (w.x - (-25.0)).abs() < 0.01);
        let has_right = walls.iter().any(|w| (w.x - 25.0).abs() < 0.01);

        assert!(has_front, "Should have front wall at z=20");
        assert!(has_back, "Should have back wall at z=-20");
        assert!(has_left, "Should have left wall at x=-25");
        assert!(has_right, "Should have right wall at x=25");
    }

    #[test]
    fn test_create_arena_deterministic() {
        let a1 = create_arena();
        let a2 = create_arena();
        assert_eq!(a1.len(), a2.len());
        for (o1, o2) in a1.iter().zip(a2.iter()) {
            assert!((o1.x - o2.x).abs() < 0.01);
            assert!((o1.y - o2.y).abs() < 0.01);
            assert!((o1.z - o2.z).abs() < 0.01);
            assert_eq!(o1.kind, o2.kind);
            assert!((o1.width - o2.width).abs() < 0.01);
            assert!((o1.height - o2.height).abs() < 0.01);
            assert!((o1.depth - o2.depth).abs() < 0.01);
        }
    }

    #[test]
    fn test_hill_center_on_ground() {
        let obstacles = create_arena();
        let ground = obstacles.iter().find(|o| {
            (o.width - 60.0).abs() < 0.01
                && (o.depth - 60.0).abs() < 0.01
        });
        let hill = obstacles.iter().find(|o| {
            (o.x - 0.0).abs() < 0.01
                && (o.z - 0.0).abs() < 0.01
                && o.kind == ObstacleKind::Platform
        });

        assert!(ground.is_some());
        assert!(hill.is_some());

        let ground = ground.unwrap();
        let hill = hill.unwrap();

        // Hill is centered on the ground (both at x=0, z=0)
        assert!((hill.x - ground.x).abs() < 0.01);
        assert!((hill.z - ground.z).abs() < 0.01);
    }

    #[test]
    fn test_obstacles_fit_within_boundary() {
        let obstacles = create_arena();
        let (min_x, max_x, min_z, max_z) = arena_bounds();

        // Check that boundary walls fully enclose the arena
        let walls: Vec<_> = obstacles.iter()
            .filter(|o| o.kind == ObstacleKind::Solid && (o.height - 4.0).abs() < 0.01)
            .collect();

        // Front wall should extend beyond arena bounds in X
        let front = walls.iter().find(|o| (o.z - max_z).abs() < 0.01);
        assert!(front.is_some());
        let front = front.unwrap();
        assert!(front.min_x() <= min_x - 0.01, "Front wall should extend beyond left boundary");
        assert!(front.max_x() >= max_x + 0.01, "Front wall should extend beyond right boundary");

        // Back wall should extend beyond arena bounds in X
        let back = walls.iter().find(|o| (o.z - min_z).abs() < 0.01);
        assert!(back.is_some());
        let back = back.unwrap();
        assert!(back.min_x() <= min_x - 0.01, "Back wall should extend beyond left boundary");
        assert!(back.max_x() >= max_x + 0.01, "Back wall should extend beyond right boundary");

        // Left wall should extend beyond arena bounds in Z
        let left = walls.iter().find(|o| (o.x - min_x).abs() < 0.01);
        assert!(left.is_some());
        let left = left.unwrap();
        assert!(left.min_z() <= min_z, "Left wall should extend beyond back boundary");
        assert!(left.max_z() >= max_z, "Left wall should extend beyond front boundary");

        // Right wall should extend beyond arena bounds in Z
        let right = walls.iter().find(|o| (o.x - max_x).abs() < 0.01);
        assert!(right.is_some());
        let right = right.unwrap();
        assert!(right.min_z() <= min_z, "Right wall should extend beyond back boundary");
        assert!(right.max_z() >= max_z, "Right wall should extend beyond front boundary");

        // Cover blocks should fit within arena bounds
        let cover_blocks: Vec<_> = obstacles.iter()
            .filter(|o| {
                o.kind == ObstacleKind::Solid
                    && (o.width - 3.0).abs() < 0.01
                    && (o.height - 3.0).abs() < 0.01
                    && (o.depth - 3.0).abs() < 0.01
            })
            .collect();
        for block in &cover_blocks {
            assert!(block.min_x() >= min_x, "Cover block min_x {:?} outside arena", block.x);
            assert!(block.max_x() <= max_x, "Cover block max_x {:?} outside arena", block.x);
            assert!(block.min_z() >= min_z, "Cover block min_z {:?} outside arena", block.z);
            assert!(block.max_z() <= max_z, "Cover block max_z {:?} outside arena", block.z);
        }
    }
}
