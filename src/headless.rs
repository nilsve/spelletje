use crate::physics::PhysicsImpl;
use crate::input::Input;
use crate::world::World;
use crate::player::Player;
/// Parse an input string like "a3w3d2" into a list of (char, seconds).
/// Each char is followed by a number of seconds to hold it.
fn parse_input(s: &str) -> Vec<(char, u64)> {
    let mut result = Vec::new();
    let mut chars = s.chars().peekable();  // <-- peekable

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
/// Convert current input keys to an Input struct
fn keys_to_input(keys: &[char]) -> Input {
    Input {
        left: keys.contains(&'a'),
        right: keys.contains(&'d'),
        jump: keys.contains(&'w'),
        forward: keys.contains(&'s'),
        backward: keys.contains(&' '),
        shoot: keys.contains(&'f'),
    }
}
pub fn run_cli(input_str: &str) {
    let sequence = parse_input(input_str);
    let mut frame_count = 0;
    let mut seq_idx = 0;
    let mut elapsed = 0.0;
    let mut world = World::new();
    world.add_entity(Player::new());
    world.add_platform(0.0, -0.25, 0.0, 100.0, 0.5, 100.0);
    let physics = PhysicsImpl::new();
    println!("Running CLI simulation with input: {}", input_str);
    println!("Sequence: {:?}", sequence);
    let initial = &world.entities[0];
    println!("{{\"x\":{:.2},\"y\":{:.2},\"z\":{:.2},\"vel_x\":{:.2},\"vel_y\":{:.2},\"vel_z\":{:.2},\"grounded\":{}}}", 
        initial.physics_data.x, initial.physics_data.y, initial.physics_data.z, initial.physics_data.vel_x, initial.physics_data.vel_y, initial.physics_data.vel_z, initial.grounded);
    loop {
        // Get current keys from sequence
        let current_keys: Vec<char> = if seq_idx < sequence.len() {
            vec![sequence[seq_idx].0]
        } else {
            vec![]
        };
        // Parse current frame input
        let input = keys_to_input(&current_keys);
        
        // Handle shooting
        if input.shoot {
            let projectile = world.entities[0].fire();
            world.add_projectile(projectile);
        }

        // Update world
        world.update_all(&input, &physics, 0.016);
        // Print debug info
        if frame_count % 20 == 0 {
            println!("Frame {}: keys={:?} pos=({:.2}, {:.2}, {:.2}) projectiles={}", 
                frame_count, current_keys, world.entities[0].physics_data.x, world.entities[0].physics_data.y, world.entities[0].physics_data.z, world.projectile_count());
        }
        // Advance sequence based on elapsed time
        elapsed += 0.016;
        if seq_idx < sequence.len() && elapsed >= sequence[seq_idx].1 as f64 {
            elapsed = 0.0;
            seq_idx += 1;
        }
        // Exit when sequence is done
        if seq_idx >= sequence.len() {
            println!("Simulation complete. Final position: ({:.2}, {:.2}, {:.2})", 
                world.entities[0].physics_data.x, world.entities[0].physics_data.y, world.entities[0].physics_data.z);
            break;
        }
        frame_count += 1;
    }
}

