/// Per-player third-person camera with split-screen support (Mario Kart style).
///
/// Split-screen layouts (1-4 players):
///   1 player: full screen
///   2 players: left / right split
///   3 players: top full-width, two bottom halves
///   4 players: 2x2 grid

use macroquad::prelude::*;

/// Per-player camera state.
#[derive(Clone, Debug)]
pub struct PlayerCamera {
    pub yaw: f32,
    pub pitch: f32,
    /// Distance behind player.
    pub distance: f32,
    /// Height above player.
    pub height: f32,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.3,
            distance: 12.0,
            height: 8.0,
        }
    }
}

/// Build a Camera3D for one player in a split-screen layout.
///
/// `split_count` is the total number of players (determines viewport layout).
/// `split_index` is this player's slot (0..split_count).
/// `viewport` is the full-screen rect (x, y, w, h) that contains all splits.
pub fn build_camera(
    player_pos: Vec3,
    cam: &PlayerCamera,
    split_count: usize,
    split_index: usize,
    viewport: Rect,
) -> Camera3D {
    let vp = viewport_rect(split_count, split_index, viewport);

    let cos_yaw = cam.yaw.cos();
    let sin_yaw = cam.yaw.sin();
    let cos_pitch = cam.pitch.cos();
    let sin_pitch = cam.pitch.sin();

    let offset_x = sin_yaw * cos_pitch * cam.distance;
    let offset_y = sin_pitch * cam.distance + cam.height;
    let offset_z = cos_yaw * cos_pitch * cam.distance;

    let cam_pos = vec3(
        player_pos.x + offset_x,
        player_pos.y + offset_y,
        player_pos.z + offset_z,
    );

    let player_h = player_pos.y + 0.5;
    let cam_target = vec3(player_pos.x, player_h, player_pos.z);

    let aspect = vp.w / vp.h;

    Camera3D {
        position: cam_pos,
        target: cam_target,
        up: vec3(0.0, -1.0, 0.0),
        fovy: 60.0,
        aspect: Some(aspect),
        projection: macroquad::camera::Projection::Perspective,
        viewport: Some((vp.x as i32, vp.y as i32, vp.w as i32, vp.h as i32)),
        ..Default::default()
    }
}

/// Public: return the viewport rect for a given split slot.
/// Re-exported for use in main.rs.
pub fn viewport_rect_for_split(split_count: usize, split_index: usize, full: Rect) -> Rect {
    viewport_rect(split_count, split_index, full)
}

/// Return the viewport rect for a given split slot.
fn viewport_rect(split_count: usize, split_index: usize, full: Rect) -> Rect {
    match split_count {
        1 => full,
        2 => {
            // Left / right split.
            if split_index == 0 {
                Rect::new(full.x, full.y, full.w * 0.5, full.h)
            } else {
                Rect::new(full.x + full.w * 0.5, full.y, full.w * 0.5, full.h)
            }
        }
        3 => {
            // Top full-width, two bottom halves.
            if split_index == 0 {
                // Top half, full width.
                Rect::new(full.x, full.y + full.h * 0.5, full.w, full.h * 0.5)
            } else if split_index == 1 {
                // Bottom-left.
                Rect::new(full.x, full.y, full.w * 0.5, full.h * 0.5)
            } else {
                // Bottom-right.
                Rect::new(full.x + full.w * 0.5, full.y, full.w * 0.5, full.h * 0.5)
            }
        }
        4 => {
            // 2x2 grid.
            let half_w = full.w * 0.5;
            let half_h = full.h * 0.5;
            match split_index {
                0 => Rect::new(full.x, full.y + half_h, half_w, half_h), // top-left
                1 => Rect::new(full.x + half_w, full.y + half_h, half_w, half_h), // top-right
                2 => Rect::new(full.x, full.y, half_w, half_h), // bottom-left
                _ => Rect::new(full.x + half_w, full.y, half_w, half_h), // bottom-right
            }
        }
        _ => full,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_camera() {
        let cam = PlayerCamera::default();
        assert_eq!(cam.yaw, 0.0);
        assert_eq!(cam.pitch, 0.3);
        assert_eq!(cam.distance, 12.0);
        assert_eq!(cam.height, 8.0);
    }

    #[test]
    fn test_viewport_1_player_full_screen() {
        let vp = Rect::new(0.0, 0.0, 1920.0, 1080.0);
        let r = viewport_rect(1, 0, vp);
        assert!((r.x - 0.0).abs() < 0.01);
        assert!((r.y - 0.0).abs() < 0.01);
        assert!((r.w - 1920.0).abs() < 0.01);
        assert!((r.h - 1080.0).abs() < 0.01);
    }

    #[test]
    fn test_viewport_2_players_left_right() {
        let vp = Rect::new(0.0, 0.0, 1920.0, 1080.0);
        let r0 = viewport_rect(2, 0, vp);
        let r1 = viewport_rect(2, 1, vp);
        assert!((r0.w - 960.0).abs() < 0.01);
        assert!((r0.h - 1080.0).abs() < 0.01);
        assert!((r1.w - 960.0).abs() < 0.01);
        assert!((r1.h - 1080.0).abs() < 0.01);
        assert!((r0.x + r0.w - r1.x).abs() < 0.01); // r0 right edge = r1 left edge
    }

    #[test]
    fn test_viewport_3_players_top_bottom() {
        let vp = Rect::new(0.0, 0.0, 1920.0, 1080.0);
        let r0 = viewport_rect(3, 0, vp); // top
        let r1 = viewport_rect(3, 1, vp); // bottom-left
        let r2 = viewport_rect(3, 2, vp); // bottom-right
        assert!((r0.w - 1920.0).abs() < 0.01);
        assert!((r0.h - 540.0).abs() < 0.01);
        assert!((r0.y - 540.0).abs() < 0.01);
        assert!((r1.w - 960.0).abs() < 0.01);
        assert!((r1.h - 540.0).abs() < 0.01);
        assert!((r2.w - 960.0).abs() < 0.01);
        assert!((r2.h - 540.0).abs() < 0.01);
    }

    #[test]
    fn test_viewport_4_players_2x2() {
        let vp = Rect::new(0.0, 0.0, 1920.0, 1080.0);
        let r0 = viewport_rect(4, 0, vp);
        let r1 = viewport_rect(4, 1, vp);
        let r2 = viewport_rect(4, 2, vp);
        let r3 = viewport_rect(4, 3, vp);
        assert!((r0.w - 960.0).abs() < 0.01);
        assert!((r0.h - 540.0).abs() < 0.01);
        assert!((r1.w - 960.0).abs() < 0.01);
        assert!((r1.h - 540.0).abs() < 0.01);
        assert!((r2.w - 960.0).abs() < 0.01);
        assert!((r2.h - 540.0).abs() < 0.01);
        assert!((r3.w - 960.0).abs() < 0.01);
        assert!((r3.h - 540.0).abs() < 0.01);
    }

    #[test]
    fn test_camera_build_basic() {
        let mut cam = PlayerCamera::default();
        cam.yaw = 0.0;
        cam.pitch = 0.3;
        cam.distance = 12.0;
        cam.height = 8.0;

        let vp = Rect::new(0.0, 0.0, 1920.0, 1080.0);
        let c = build_camera(vec3(0.0, 0.0, 0.0), &cam, 1, 0, vp);

        // Camera should be behind player (positive z when yaw=0).
        assert!(c.position.z > 0.0);
        // Camera should be above player.
        assert!(c.position.y > 0.0);
        // Target should be near player center.
        assert!((c.target.y - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_camera_yaw_rotates_around_player() {
        let mut cam = PlayerCamera::default();
        cam.distance = 10.0;
        cam.height = 5.0;
        cam.pitch = 0.0;

        let vp = Rect::new(0.0, 0.0, 1920.0, 1080.0);

        // yaw=0: camera behind player (+z).
        cam.yaw = 0.0;
        let c0 = build_camera(vec3(0.0, 0.0, 0.0), &cam, 1, 0, vp);
        assert!(c0.position.z > 0.0);
        assert!((c0.position.x).abs() < 0.1);

        // yaw=PI/2: camera to the right of player (+x).
        cam.yaw = std::f32::consts::FRAC_PI_2;
        let c90 = build_camera(vec3(0.0, 0.0, 0.0), &cam, 1, 0, vp);
        assert!(c90.position.x > 0.0);
        assert!((c90.position.z).abs() < 0.1);
    }
}
