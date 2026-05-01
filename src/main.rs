#[cfg(feature = "gui")]
use macroquad::prelude::*;
#[cfg(feature = "gui")]
use macroquad::camera::{Camera3D, set_camera, set_default_camera};
#[cfg(feature = "gui")]
use macroquad::camera::Projection;

#[cfg(feature = "gui")]
use spelletje_mac::physics::{Physics, PhysicsImpl, PhysicsConfig};
#[cfg(feature = "gui")]
use spelletje_mac::input::{Input, InputSource, MacroquadInput};
#[cfg(feature = "gui")]
use spelletje_mac::world::World;
#[cfg(feature = "gui")]
use spelletje_mac::player::Player;
#[cfg(feature = "gui")]
use spelletje_mac::platform::Platform;

use spelletje_mac::headless;

#[cfg(feature = "gui")]
fn get_game_input() -> Input {
    Input {
        left: is_key_down(KeyCode::A),
        right: is_key_down(KeyCode::D),
        jump: is_key_pressed(KeyCode::W),
        forward: is_key_down(KeyCode::S),
        backward: is_key_down(KeyCode::Space),
    }
}

#[cfg(feature = "gui")]
fn draw_platform(platform: &Platform, color: Color) {
    let size = vec3(platform.width, platform.height, platform.depth);
    let pos = vec3(platform.x, platform.y, platform.z);
    draw_cube(pos, size, None, color);
    draw_cube_wires(pos, size, DARKGRAY);
}

#[cfg(feature = "gui")]
async fn game_loop() {
    let mut world = World::new();

    world.add_platform(Platform::new(0.0, -0.25, 0.0, 100.0, 0.5, 100.0));
    world.add_platform(Platform::new(5.0, 1.5, 0.0, 4.0, 0.5, 4.0));
    world.add_platform(Platform::new(-5.0, 2.5, 0.0, 3.0, 0.5, 3.0));

    let player = Player::new();
    world.add_entity(player);

    let physics = PhysicsImpl::new();
    let input_source = MacroquadInput;

    loop {
        clear_background(BLACK);

        let dt = get_frame_time();
        let input = input_source.read();

        world.update_all(&input, &physics, dt);

        let player_ref = world.entities.get(0).unwrap();

        let screen_aspect = screen_width() / screen_height();
        let cam_height = 15.0;
        let cam_target = vec3(player_ref.x, player_ref.y + player_ref.size / 2.0, player_ref.z);
        let camera = Camera3D {
            position: vec3(cam_target.x, cam_target.y, cam_target.z + cam_height),
            target: cam_target,
            up: vec3(0.0, 1.0, 0.0),
            fovy: 10.0,
            aspect: Some(screen_aspect),
            projection: Projection::Orthographics,
            ..Default::default()
        };

        set_camera(&camera);

        draw_plane(vec3(0.0, 0.0, 0.0), vec2(100.0, 100.0), None, DARKGRAY);

        for i in -20..=20 {
            let c = DARKGRAY;
            draw_line_3d(vec3(-20.0, 0.001, i as f32), vec3(20.0, 0.001, i as f32), c);
            draw_line_3d(vec3(i as f32, 0.001, -20.0), vec3(i as f32, 0.001, 20.0), c);
        }

        for platform in &world.platforms {
            draw_platform(platform, GREEN);
        }

        for entity in &world.entities {
            let size = vec3(entity.size, entity.size, entity.size * 0.5);
            let pos = vec3(entity.x, entity.y + entity.size / 2.0, entity.z);
            draw_cube(pos, size, None, BLUE);
            draw_cube_wires(pos, size, DARKBLUE);
        }

        set_default_camera();
        draw_text("A/D to move | W to jump | S to move depth", 10.0, 30.0, 20.0, WHITE);
        draw_text(
            &format!("X: {:.1}  Y: {:.1}  Z: {:.1}", player_ref.x, player_ref.y, player_ref.z),
            10.0, 55.0, 20.0, YELLOW,
        );

        next_frame().await;
    }
}

#[cfg(all(feature = "gui", not(feature = "cli")))]
#[macroquad::main("Spelletje")]
async fn main() {
    game_loop().await;
}

#[cfg(any(not(feature = "gui"), feature = "cli"))]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input_str = args.get(1).map(|s| s.as_str()).unwrap_or("");
    headless::run_cli(input_str);
}
