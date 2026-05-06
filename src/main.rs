use macroquad::camera::{Camera3D, set_camera, set_default_camera};
use macroquad::prelude::*;

use spelletje_mac::camera::{build_camera, PlayerCamera};
use spelletje_mac::gamestate::{GameState, GameStateManager};
use spelletje_mac::PhysicsEntity;
use spelletje_mac::enemy::Enemy;
use spelletje_mac::input::{GamepadInputImpl, Input, InputSource, MacroquadInput, PlayerInput};
use spelletje_mac::obstacle::Obstacle;
use spelletje_mac::physics::Physics;
use spelletje_mac::player::Player;
use spelletje_mac::powerup::create_powerups;
use spelletje_mac::projectile::Projectile;
use spelletje_mac::world::World;
use spelletje_mac::arena::create_arena;

fn draw_obstacle(obstacle: &Obstacle, color: Color) {
    let size = vec3(obstacle.width, obstacle.height, obstacle.depth);
    let pos = vec3(obstacle.x, obstacle.y, obstacle.z);
    draw_cube(pos, size, None, color);
    draw_cube_wires(pos, size, DARKGRAY);
}

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

fn draw_gun(player: &Player) {
    let pos = player.physics_data().pos();
    let gun_start = vec3(pos.0, pos.1 + player.physics_data().size() / 2.0, pos.2);
    let gun_end = player.gun_end();
    let gun_pos = vec3(gun_end.0, gun_end.1, gun_end.2);
    draw_line_3d(gun_start, gun_pos, WHITE);
}

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

fn draw_world_in_camera(camera: &Camera3D, world: &World, time: f32) {
    set_camera(camera);

    draw_plane(vec3(0.0, -1.0, 0.0), vec2(200.0, 200.0), None, DARKGRAY);

    for i in -20..=20 {
        let c = DARKGRAY;
        draw_line_3d(vec3(-20.0, 0.001, i as f32), vec3(20.0, 0.001, i as f32), c);
        draw_line_3d(vec3(i as f32, 0.001, -20.0), vec3(i as f32, 0.001, 20.0), c);
    }

    // Hill indicator circle on ground
    let hx = world.hill.x;
    let hz = world.hill.z;
    let pulse = (time * 3.0).sin() * 0.1 + 1.0;
    let indicator_radius = world.hill.width / 2.0 * pulse + 0.5;
    for j in 0..24 {
        let a1 = j as f32 * std::f32::consts::PI * 2.0 / 24.0;
        let a2 = (j + 1) as f32 * std::f32::consts::PI * 2.0 / 24.0;
        let p1 = vec3(hx + a1.cos() * indicator_radius, 0.01, hz + a1.sin() * indicator_radius);
        let p2 = vec3(hx + a2.cos() * indicator_radius, 0.01, hz + a2.sin() * indicator_radius);
        draw_line_3d(p1, p2, YELLOW);
    }

    // Color obstacles by type
    use spelletje_mac::obstacle::ObstacleKind;
    for obstacle in &world.obstacles {
        let is_hill = (obstacle.x - world.hill.x).abs() < 0.1
            && (obstacle.y - world.hill.y).abs() < 0.1
            && (obstacle.z - world.hill.z).abs() < 0.1;
        let is_wall = obstacle.kind == ObstacleKind::Solid && (obstacle.height - 4.0).abs() < 0.01;
        let is_cover = obstacle.kind == ObstacleKind::Solid
            && (obstacle.width - 3.0).abs() < 0.01
            && (obstacle.height - 3.0).abs() < 0.01;
        let color = if is_hill {
            YELLOW
        } else if is_wall {
            RED
        } else if is_cover {
            GRAY
        } else {
            GREEN
        };
        draw_obstacle(obstacle, color);
    }

    // Draw active power-ups
    for pu in &world.powerups {
        if pu.active {
            let (r, g, b) = pu.kind.color();
            let pu_pulse = (time * 4.0).sin() * 0.15 + 1.0;
            let sz = 0.5 * pu_pulse;
            let size = vec3(sz, sz, sz);
            let pos = vec3(pu.x, pu.y, pu.z);
            let color = Color::new(r, g, b, 1.0);
            draw_cube(pos, size, None, color);
            draw_cube_wires(pos, size, WHITE);
        }
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
}

fn draw_menu_hud(gamepad_count: usize) {
    let cx = screen_width() / 2.0;
    draw_text("King of the Hill", cx - 100.0, 100.0, 48.0, YELLOW);
    draw_text("First to 10 points wins!", cx - 120.0, 160.0, 24.0, WHITE);
    draw_text(
        &format!("Gamepads connected: {}", gamepad_count),
        cx - 80.0,
        220.0,
        20.0,
        GREEN,
    );
       draw_text("Press Space to start", cx - 90.0, 280.0, 24.0, WHITE);
}

fn draw_playing_hud(world: &World) {
    let player_colors = [
        Color::new(0.2, 0.2, 1.0, 1.0),
        Color::new(1.0, 0.2, 0.2, 1.0),
        Color::new(0.2, 1.0, 0.2, 1.0),
        Color::new(1.0, 1.0, 0.2, 1.0),
    ];

    // Score display at top
    let score_text = world.hill.scores.iter().enumerate().fold(
        String::new(),
        |mut s, (i, &score)| {
            if i > 0 {
                s.push_str(" | ");
            }
            s.push_str(&format!("P{}: {}", i, score));
            s
        },
    );
    draw_text(&score_text, screen_width() / 2.0 - 100.0, 20.0, 28.0, YELLOW);

    // Hill teleport timer
    let remaining = (world.hill.teleport_interval - world.hill.teleport_timer).max(0.0);
    draw_text(
        &format!("Hill teleport: {:.0}s", remaining),
        screen_width() / 2.0 - 60.0,
        50.0,
        18.0,
        ORANGE,
    );

    // Player info with power-up status
    for (i, entity) in world.players.iter().enumerate() {
        let e = entity.physics_data().pos();
        let mut info = format!("P{}: X:{:.0} Y:{:.0} Z:{:.0}", i, e.0, e.1, e.2);
        if let Some(ref pu) = entity.active_powerup {
            info.push_str(" [");
            match pu {
                spelletje_mac::powerup::PowerUpKind::SpeedBoost => info.push_str("SPEED"),
                spelletje_mac::powerup::PowerUpKind::DoubleJump => info.push_str("2JUMP"),
                spelletje_mac::powerup::PowerUpKind::BiggerHill => info.push_str("BIG"),
                spelletje_mac::powerup::PowerUpKind::Shield => info.push_str("SHIELD"),
            }
            info.push_str(&format!(" {:.0}s]", entity.powerup_timer));
        }
        draw_text(
            &info,
            10.0,
            80.0 + (i as f32) * 25.0,
            18.0,
            player_colors[i % player_colors.len()],
        );
    }
}

fn draw_game_over_hud(game_state: &GameStateManager, world: &World) {
    let cx = screen_width() / 2.0;

    // Dim overlay
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.5));

    if let Some(winner) = game_state.winner {
        draw_text("GAME OVER", cx - 80.0, 100.0, 48.0, RED);
        draw_text(
            &format!("Player {} Wins!", winner),
            cx - 80.0,
            170.0,
            36.0,
            YELLOW,
        );
    }

    // Final scores
    let score_text = world.hill.scores.iter().enumerate().fold(
        String::new(),
        |mut s, (i, &score)| {
            if i > 0 {
                s.push_str(" | ");
            }
            s.push_str(&format!("P{}: {}", i, score));
            s
        },
    );
    draw_text(&score_text, cx - 80.0, 240.0, 24.0, WHITE);

      draw_text("Press Space to restart", cx - 100.0, 310.0, 24.0, WHITE);
}

async fn game_loop() {
    let mut world = World::new();
    for obstacle in create_arena() {
        world.add_obstacle(obstacle);
    }

    for _ in 0..4 {
        world.add_entity(Player::default());
    }
    world.hill.init_scores(4);

    // Power-ups
    world.powerups = create_powerups();

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

    let mut player_cameras: Vec<PlayerCamera> = (0..4)
        .map(|i| {
            let mut cam = PlayerCamera::default();
            cam.yaw = (i as f32) * std::f32::consts::PI / 2.0;
            cam
        })
        .collect();

    let mut gamepad_impl = GamepadInputImpl::new();
    let mut game_state = GameStateManager::new();
    let mut total_time = 0.0f32;

    let mut camera_yaw = 0.0f32;
    let mut camera_pitch = 0.0f32;
    let mut is_panning = false;
    let mut last_mouse_pos = (0.0f32, 0.0f32);

    loop {
        clear_background(BLACK);

        let dt = get_frame_time().min(1. * 0.1);
        total_time += dt;

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

        // Game state transitions
        game_state.update(&player_inputs);

        if game_state.is_playing() {
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

            // Check win condition
            if let Some(winner) = world.winning_player() {
                game_state.set_game_over(winner);
            }
        }

        let screen_w = screen_width();
        let screen_h = screen_height();
        let full_viewport = Rect::new(0.0, 0.0, screen_w, screen_h);

        let active_players = if gamepad_count > 0 {
            gamepad_count
        } else {
            1
        };
        let active_players = active_players.min(world.players.len()).max(1);

        // Mouse camera control for player 0 (when no gamepad)
        if gamepad_count == 0 {
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

            player_cameras[0].yaw = camera_yaw;
            player_cameras[0].pitch = camera_pitch;
        }

        // Per-player camera control from input
        for i in 0..active_players {
            if let Some(input) = player_inputs.get(i) {
                player_cameras[i].yaw += input.camera_yaw_speed * 2.0 * dt;
                player_cameras[i].pitch -= input.camera_pitch_speed * 1.5 * dt;
                player_cameras[i].pitch = player_cameras[i].pitch.max(-1.2).min(1.2);
            }
        }

        // Render 3D world
        if game_state.state != GameState::Menu {
            for i in 0..active_players {
                let p = world.players[i].physics_data().pos();
                let player_pos = vec3(p.0, p.1, p.2);
                let camera = build_camera(player_pos, &player_cameras[i], active_players, i, full_viewport);
                draw_world_in_camera(&camera, &world, total_time);
            }
        }

        set_default_camera();

        // HUD based on game state
        match game_state.state {
            GameState::Menu => draw_menu_hud(gamepad_count),
            GameState::Playing => draw_playing_hud(&world),
            GameState::GameOver => draw_game_over_hud(&game_state, &world),
        }

        next_frame().await;
    }
}

#[macroquad::main("Spelletje")]
async fn main() {
    game_loop().await;
}
