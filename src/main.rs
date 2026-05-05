#[cfg(feature = "gui")]
use macroquad::camera::Projection;
#[cfg(feature = "gui")]
use macroquad::camera::{Camera3D, set_camera, set_default_camera};
#[cfg(feature = "gui")]
use macroquad::prelude::*;

#[cfg(any(not(feature = "gui"), feature = "cli"))]
use spelletje_mac::headless;
#[cfg(feature = "gui")]
use spelletje_mac::PhysicsEntity;
#[cfg(feature = "gui")]
use spelletje_mac::enemy::Enemy;
#[cfg(feature = "gui")]
use spelletje_mac::input::{GamepadInputImpl, Input, InputSource, MacroquadInput, PlayerInput};
#[cfg(feature = "gui")]
use spelletje_mac::obstacle::Obstacle;
#[cfg(feature = "gui")]
use spelletje_mac::physics::Physics;
#[cfg(feature = "gui")]
use spelletje_mac::player::Player;
#[cfg(feature = "gui")]
use spelletje_mac::projectile::Projectile;
#[cfg(feature = "gui")]
use spelletje_mac::world::World;
#[cfg(feature = "gui")]
use spelletje_mac::arena::create_arena;

#[cfg(feature = "gui")]
fn draw_obstacle(obstacle: &Obstacle, color: Color) {
    let size = vec3(obstacle.width, obstacle.height, obstacle.depth);
    let pos = vec3(obstacle.x, obstacle.y, obstacle.z);
    draw_cube(pos, size, None, color);
    draw_cube_wires(pos, size, DARKGRAY);
}

#[cfg(feature = "gui")]
fn draw_projectile(projectile: &Projectile) {
    let size = vec3(0.15, 0.15, 0.15);
    let pos = vec3(
        projectile.position().0,
        projectile.position().1,
        projectile.position().2,
    );
    draw_cube(pos, size, None, YELLOW);
    draw_cube_wires(pos, size, ORANGE);
}

#[cfg(feature = "gui")]
fn draw_gun(player: &Player) {
    let pos = player.physics_data().pos();
    let gun_start = vec3(pos.0, pos.1 + player.physics_data().size() / 2.0, pos.2);
    let gun_end = player.gun_end();
    let gun_pos = vec3(gun_end.0, gun_end.1, gun_end.2);
    draw_line_3d(gun_start, gun_pos, WHITE);
}

#[cfg(feature = "gui")]
fn draw_enemy(enemy: &Enemy) {
    let pos = enemy.pos();
    let s = enemy.size();
    let size = vec3(s, s, s * 0.8);
    let pos_vec = vec3(pos.0, pos.1 + s / 2.0, pos.2);
    let color = if enemy.health > 30.0 {
        RED
    } else if enemy.health > 15.0 {
        ORANGE
    } else {
        Color::new(0.4, 0.0, 0.0, 1.0)
    };
    draw_cube(pos_vec, size, None, color);
    draw_cube_wires(pos_vec, size, Color::new(0.2, 0.0, 0.0, 1.0));
}

#[cfg(feature = "gui")]
async fn game_loop() {
    let mut world = World::new();
    for obstacle in create_arena() {
        world.add_obstacle(obstacle);
    }
    world.add_obstacle(Obstacle::solid(5.0, 1.5, 0.0, 4.0, 1., 100.0));
    world.add_platform(-5.0, 2.5, 0.0, 3.0, 0.5, 3.0);

    for _ in 0..4 {
        world.add_entity(Player::default());
    }

    world.add_enemy(Enemy::default());
    world.enemies[0].physics_data.x = 10.0;
    world.enemies[0].physics_data.z = 5.0;

    let mut enemy2 = Enemy::default();
    enemy2.physics_data.x = -8.0;
    enemy2.physics_data.z = 0.0;
    enemy2.health = 80.0;
    enemy2.damage = 15.0;
    enemy2.shoot_interval = 1.5;
    world.add_enemy(enemy2);

    let physics = Physics::new();

    let mut camera_yaw = 0.0f32;
    let mut camera_pitch = 0.0f32;
    let mut is_panning = false;
    let mut last_mouse_pos = (0.0f32, 0.0f32);

    let gamepad_impl = GamepadInputImpl::new();

    loop {
        clear_background(BLACK);

        let dt = get_frame_time().min(1. * 0.1);

        gamepad_impl.poll();

        let gamepad_count = gamepad_impl.read_all().player_count();

        let (input, input_count) = if gamepad_count > 0 {
            (Input::default(), gamepad_count)
        } else {
            let input_source = MacroquadInput;
            let input = input_source.read();
            (input, 1)
        };

        let player_inputs: Vec<PlayerInput> = if gamepad_count > 0 {
            gamepad_impl.read_all().players
        } else {
            vec![(&input).into()]
        };

        let right_pressed = is_mouse_button_down(MouseButton::Right);
        if right_pressed && !is_panning {
            is_panning = true;
            last_mouse_pos = mouse_position();
        } else if !right_pressed && is_panning {
            is_panning = false;
        }

        if is_panning {
            let (cx, cy) = mouse_position();
            let (dx, dy) = (cx - last_mouse_pos.0, cy - last_mouse_pos.1);
            camera_yaw += dx * 0.005;
            camera_pitch -= dy * 0.003;
            camera_pitch = camera_pitch.max(-1.2).min(1.2);
            last_mouse_pos = (cx, cy);
        }

        let shoot_pressed = if gamepad_count > 0 {
            gamepad_impl
                .read_all()
                .players
                .first()
                .map(|p| p.shoot)
                .unwrap_or(false)
        } else {
            is_mouse_button_pressed(MouseButton::Left)
        };

        if shoot_pressed && !world.players.is_empty() {
            let projectile = world.players[0].fire();
            world.add_projectile(projectile);
        }

        world.update_all(&player_inputs, &physics, dt);

        let screen_aspect = screen_width() / screen_height();
        let cam_height = 15.0;
        let player_ref = world.players.first().unwrap();
        let p = player_ref.physics_data().pos();
        let s = player_ref.physics_data().size();
        let cam_target = vec3(p.0, p.1 + s / 2.0, p.2);

        let cos_yaw = camera_yaw.cos();
        let sin_yaw = camera_yaw.sin();
        let cam_dist = cam_height;
        let cam_x = cam_target.x + sin_yaw * cam_dist;
        let cam_z = cam_target.z + cos_yaw * cam_dist;
        let cam_y = cam_target.y + camera_pitch.sin().abs() * cam_dist * 0.5;

        let camera = Camera3D {
            position: vec3(cam_x, cam_y, cam_z),
            target: cam_target,
            up: vec3(0.0, 1.0, 0.0),
            fovy: 10.0,
            aspect: Some(screen_aspect),
            projection: Projection::Orthographics,
            ..Default::default()
        };

        set_camera(&camera);

        draw_plane(vec3(0.0, -1.0, 0.0), vec2(200.0, 200.0), None, DARKGRAY);

        for i in -20..=20 {
            let c = DARKGRAY;
            draw_line_3d(vec3(-20.0, 0.001, i as f32), vec3(20.0, 0.001, i as f32), c);
            draw_line_3d(vec3(i as f32, 0.001, -20.0), vec3(i as f32, 0.001, 20.0), c);
        }

        for obstacle in &world.obstacles {
            draw_obstacle(obstacle, GREEN);
        }

        let player_colors = [
            Color::new(0.2, 0.2, 1.0, 1.0),
            Color::new(1.0, 0.2, 0.2, 1.0),
            Color::new(0.2, 1.0, 0.2, 1.0),
            Color::new(1.0, 1.0, 0.2, 1.0),
        ];

        for (i, entity) in world.players.iter().enumerate() {
            let s = entity.physics_data().size();
            let e = entity.physics_data().pos();
            let size = vec3(s, s, s * 0.5);
            let pos = vec3(e.0, e.1 + s / 2.0, e.2);
            let color = player_colors[i % player_colors.len()];
            draw_cube(pos, size, None, color);
        }

        for enemy in &world.enemies {
            draw_enemy(enemy);
        }

        for projectile in &world.projectiles {
            draw_projectile(projectile);
        }

        for entity in &world.players {
            draw_gun(entity);
        }

        set_default_camera();
        draw_text(
            "A/D to move | W to jump | S to move depth | Mouse to aim | Click to shoot",
            10.0,
            30.0,
            20.0,
            WHITE,
        );

        for (i, entity) in world.players.iter().enumerate() {
            let e = entity.physics_data().pos();
            draw_text(
                &format!("P{i}: X: {:.1}  Y: {:.1}  Z: {:.1}", e.0, e.1, e.2),
                10.0,
                55.0 + (i as f32) * 25.0,
                20.0,
                player_colors[i % player_colors.len()],
            );
        }

        draw_text(
            &format!("Projectiles: {}", world.projectile_count()),
            10.0,
            55.0 + (input_count as f32) * 25.0 + 25.0,
            20.0,
            YELLOW,
        );
        draw_text(
            &format!("Enemies: {}", world.enemy_count()),
            10.0,
            55.0 + (input_count as f32) * 25.0 + 50.0,
            20.0,
            YELLOW,
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
