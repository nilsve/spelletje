#[cfg(feature = "gui")]
use macroquad::prelude::*;
#[cfg(feature = "gui")]
use macroquad::camera::{Camera3D, set_camera, set_default_camera};
#[cfg(feature = "gui")]
use macroquad::camera::Projection;

#[cfg(feature = "gui")]
use spelletje_mac::physics::PhysicsImpl;
#[cfg(feature = "gui")]
use spelletje_mac::input::{InputSource, MacroquadInput};
#[cfg(feature = "gui")]
use spelletje_mac::world::World;
#[cfg(feature = "gui")]
use spelletje_mac::player::Player;
#[cfg(feature = "gui")]
use spelletje_mac::obstacle::Obstacle;
#[cfg(feature = "gui")]
use spelletje_mac::projectile::Projectile;
#[cfg(feature = "gui")]
use spelletje_mac::enemy::{Enemy, EnemyConfig};

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
    let pos = vec3(projectile.x, projectile.y, projectile.z);
    draw_cube(pos, size, None, YELLOW);
    draw_cube_wires(pos, size, ORANGE);
}

#[cfg(feature = "gui")]
fn draw_gun(player: &Player) {
    let gun_start = vec3(player.x, player.y + player.size / 2.0, player.z);
    let gun_end = player.gun_end();
    let gun_pos = vec3(gun_end.0, gun_end.1, gun_end.2);
    draw_line_3d(gun_start, gun_pos, WHITE);
}

#[cfg(feature = "gui")]
fn draw_enemy(enemy: &Enemy) {
    let size = vec3(enemy.size, enemy.size, enemy.size * 0.8);
    let pos = vec3(enemy.x, enemy.y + enemy.size / 2.0, enemy.z);
    let color = if enemy.config.health > 30.0 {
        RED
    } else if enemy.config.health > 15.0 {
        ORANGE
    } else {
        Color::new(0.4, 0.0, 0.0, 1.0)
    };
    draw_cube(pos, size, None, color);
    draw_cube_wires(pos, size, Color::new(0.2, 0.0, 0.0, 1.0));
}

#[cfg(feature = "gui")]
async fn game_loop() {
    let mut world = World::new();

    world.add_platform(0.0, -0.25, 0.0, 100.0, 0.5, 100.0);
    world.add_obstacle(Obstacle::solid(5.0, 1.5, 0.0, 4.0, 0.5, 4.0));
    world.add_obstacle(Obstacle::solid(-5.0, 2.5, 0.0, 3.0, 0.5, 3.0));

    let player = Player::new();
    world.add_entity(player);

    // Spawn enemies
    world.add_enemy(Enemy::new(EnemyConfig::default()));
    world.enemies[0].x = 10.0;
    world.enemies[0].z = 5.0;
    
    let mut enemy2 = Enemy::new(EnemyConfig::default());
    enemy2.x = -8.0;
    enemy2.z = 8.0;
    enemy2.config.health = 80.0;
    enemy2.config.damage = 15.0;
    enemy2.config.shoot_interval = 1.5;
    world.add_enemy(enemy2);

    let physics = PhysicsImpl::new();
    let input_source = MacroquadInput;

    loop {
        clear_background(BLACK);

        let dt = get_frame_time();

        // Mouse aiming: map mouse X/Y to gun angle and pitch
        let (mx, my) = mouse_position();
        let (sw, sh) = (screen_width(), screen_height());
        if sw > 0.0 && sh > 0.0 {
            world.entities[0].gun_angle = (mx / sw - 0.5) * std::f32::consts::PI;
            world.entities[0].gun_pitch = (0.5 - my / sh) * std::f32::consts::PI;
        }

        let mut input = input_source.read();

        // Handle shooting
        if is_mouse_button_pressed(MouseButton::Left) {
            let projectile = world.entities[0].fire();
            world.add_projectile(projectile);
        }

        world.update_all(&input, &physics, dt);

        let player_ref = world.entities.first().unwrap();

        let screen_aspect = sw / screen_height();
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

        for obstacle in &world.obstacles {
            draw_obstacle(obstacle, GREEN);
        }

        for entity in &world.entities {
            let size = vec3(entity.size, entity.size, entity.size * 0.5);
            let pos = vec3(entity.x, entity.y + entity.size / 2.0, entity.z);
            draw_cube(pos, size, None, BLUE);
            draw_cube_wires(pos, size, DARKBLUE);
        }

        for enemy in &world.enemies {
            draw_enemy(enemy);
        }

        for projectile in &world.projectiles {
            draw_projectile(projectile);
        }

        for entity in &world.entities {
            draw_gun(entity);
        }

        set_default_camera();
        draw_text("A/D to move | W to jump | S to move depth | Mouse to aim | Click to shoot", 10.0, 30.0, 20.0, WHITE);
        draw_text(
            &format!("X: {:.1}  Y: {:.1}  Z: {:.1}", player_ref.x, player_ref.y, player_ref.z),
            10.0, 55.0, 20.0, YELLOW,
        );
        draw_text(
            &format!("Projectiles: {}", world.projectile_count()),
            10.0, 80.0, 20.0, YELLOW,
        );
        draw_text(
            &format!("Enemies: {}", world.enemy_count()),
            10.0, 105.0, 20.0, YELLOW,
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
