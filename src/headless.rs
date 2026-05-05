use crate::PhysicsEntity;
use crate::arena::create_arena;
use crate::input::PlayerInput;
use crate::physics::Physics;
use crate::player::Player;
use crate::world::World;
/// Parse an input string like "a3w3d2" into a list of (char, seconds).
/// Each char is followed by a number of seconds to hold it.
fn parse_input(s: &str) -> Vec<(char, u64)> {
    let mut result = Vec::new();
    let mut chars = s.chars().peekable(); // <-- peekable

    while let Some(key) = chars.next() {
        let mut num_str = String::new();

        // peek() checks the next char WITHOUT consuming it
        while let Some(&c) = chars.peek() {
            if c.is_ascii_digit() {
                num_str.push(c);
                chars.next(); // only consume confirmed digits
            } else {
                break; // non-digit stays in the iterator
            }
        }

        let seconds: u64 = if num_str.is_empty() {
            1
        } else {
            num_str.parse().unwrap_or(1)
        };

        result.push((key, seconds));
    }

    result
}
/// Convert current input keys to a PlayerInput struct
fn keys_to_player_input(keys: &[char]) -> PlayerInput {
    let move_x = if keys.contains(&'d') { 1.0 } else if keys.contains(&'a') { -1.0 } else { 0.0 };
    let move_z = if keys.contains(&'s') { 1.0 } else if keys.contains(&' ') { -1.0 } else { 0.0 };
    PlayerInput {
        move_x,
        move_z,
        jump: keys.contains(&'w'),
        shoot: keys.contains(&'f'),
    }
}
pub fn run_cli(input_str: &str) {
    let sequence = parse_input(input_str);
    let mut frame_count = 0;
    let mut seq_idx = 0;
    let mut elapsed = 0.0;
    let mut world = World::new();
    for obstacle in create_arena() {
        world.add_obstacle(obstacle);
    }
    world.add_entity(Player::default());
    let physics = Physics::new();
    println!("Running CLI simulation with input: {}", input_str);
    println!("Sequence: {:?}", sequence);
    let initial = &world.players[0];
    let physics_data = initial.physics_data();
    println!(
        "{{\"x\":{:.2},\"y\":{:.2},\"z\":{:.2},\"vel_x\":{:.2},\"vel_y\":{:.2},\"vel_z\":{:.2},\"grounded\":{}}}",
        physics_data.x,
        physics_data.y,
        physics_data.z,
        physics_data.vel_x,
        physics_data.vel_y,
        physics_data.vel_z,
        physics_data.is_grounded
    );
    loop {
        // Get current keys from sequence
        let current_keys: Vec<char> = if seq_idx < sequence.len() {
            vec![sequence[seq_idx].0]
        } else {
            vec![]
        };
        // Parse current frame input
        let player_input = keys_to_player_input(&current_keys);

        // Handle shooting
        if player_input.shoot {
            let projectile = world.players[0].fire();
            world.add_projectile(projectile);
        }

        // Update world with per-player inputs
        let inputs = [player_input];
        world.update_all(&inputs, &physics, 0.016);
        // Print debug info
        if frame_count % 20 == 0 {
            println!(
                "Frame {}: keys={:?} pos=({:.2}, {:.2}, {:.2}) projectiles={}",
                frame_count,
                current_keys,
                world.players[0].physics_data().x,
                world.players[0].physics_data().y,
                world.players[0].physics_data().z,
                world.projectile_count()
            );
        }
        // Advance sequence based on elapsed time
        elapsed += 0.016;
        if seq_idx < sequence.len() && elapsed >= sequence[seq_idx].1 as f64 {
            elapsed = 0.0;
            seq_idx += 1;
        }
        // Exit when sequence is done
        if seq_idx >= sequence.len() {
            println!(
                "Simulation complete. Final position: ({:.2}, {:.2}, {:.2})",
                world.players[0].physics_data().x,
                world.players[0].physics_data().y,
                world.players[0].physics_data().z
            );
            break;
        }
        frame_count += 1;
    }
}
