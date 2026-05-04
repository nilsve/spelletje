# spelletje-mac — Game Engine

Rust game engine with 3D cube world. Player moves on platforms (jump, move). Dual-mode: GUI (macroquad) and headless/CLI simulation.

## Architecture

```
main.rs ─┬─ GUI mode (macroquad async loop)
         └─ CLI/headless mode (deterministic simulation)

src/
  ├── lib.rs          — Re-exports all modules
  ├── main.rs         — Entry point, dual-mode routing
  ├── headless.rs      — Headless simulation engine
  ├── input/mod.rs     — Input abstraction (trait + GUI impl)
  ├── physics/mod.rs   — Physics trait + impl, collision resolution
  ├── player/mod.rs    — Player entity, config, update loop
  ├── world/mod.rs     — World manager: entities + platforms
  ├── platform/mod.rs  — AABB trait, Platform struct
  └── projectile/mod.rs— Projectile (not yet integrated into world)
```

## Headless Testing (Key Workflow)

The headless wrapper enables deterministic, replayable simulation without GUI. Used for testing physics behavior with string-based input sequences.

### Input Parsing: `parse_input("d2a3")`
- Each char = key press; digits following = seconds held
- `"w3d5"` → `[('w', 3), ('d', 5)]` — hold W for 3s, then D for 5s
- `"d2a2"` → `[('d', 2), ('a', 2)]`
- Each key in the sequence is pressed **alone** (not combined)

### Running Headless Simulation
```bash
# With GUI feature enabled (requires --cli flag to skip macroquad init):
cargo run --features "gui,cli" -- d3w2
# OR:
cargo run --no-default-features -- d3w2

# Without GUI feature (always headless):
cargo run -- d3w2
```

### Simulation Loop (`headless::run_cli`)
1. Creates minimal world: 1 player + 1 ground platform
2. Runs at fixed dt=0.016s (~60fps)
3. For each frame:
   - Determine current key from sequence based on elapsed time
   - Call `world.update_all()` with single-key Input
   - Print debug info every 60 frames
4. Exits when all sequence events consumed

### Creating Headless Worlds
```rust
use spelletje_mac::headless;
use spelletje_mac::{World, Player, Platform};
use spelletje_mac::physics::PhysicsImpl;
use spelletje_mac::input::Input;

// Create world with custom platforms
let mut world = World::new();
world.add_platform(Platform::new(0.0, -0.25, 0.0, 100.0, 0.5, 100.0)); // ground
world.add_platform(Platform::new(5.0, 2.0, 0.0, 4.0, 0.5, 4.0));        // elevated platform

// Add player with custom config
let mut player = Player::default();
player.config.acceleration = 15.0;
player.config.max_speed = 8.0;
world.add_entity(player);

// Run a single frame
let input = Input { left: true, ..Default::default() };
let physics = PhysicsImpl::new();
world.update_all(&input, &physics, 0.016);
```

## Physics System

### Core Trait (`Physics`)
All movement and collision methods live on this trait for testability:
- `apply_gravity(vel_y, dt, config)` — decreases vel_y
- `apply_jump(vel_y, config)` — sets vel_y to jump_force
- `apply_friction(vel, dt, config)` — reduces velocity toward zero
- `apply_acceleration(vel, input, dt, config)` — adds velocity based on input
- `clamp_speed(vel, config)` — caps at max_speed
- `resolve_ground_collision(pos_y, vel_y, config)` — ground plane check (y=0)
- `resolve_platform_collision(player, vel_y, platform)` — AABB vertical collision → Bottom/None
- `resolve_horizontal_collision(player, vel_x, platform)` — AABB side collision → Left/Right/None

### Collision Detection
- **Platform collision:** Checks X/Z overlap first, then if player is falling and near platform top (0.5 unit tolerance)
- **Horizontal collision:** Checks Y/Z overlap, then direction of motion toward platform edge
- Both use the `Aabb` trait for generic bounding box checks

### Config (`PhysicsConfig`)
| Field | Default | Purpose |
|-------|---------|---------|
| gravity | 20.0 | Downward acceleration per second |
| jump_force | 10.0 | Upward velocity on jump |
| max_speed | 5.0 | Horizontal speed cap |
| acceleration | 20.0 | Acceleration rate per second |
| friction | 5.0 | Velocity reduction rate |
| friction_threshold | 0.01 | Below this, velocity = 0 |

## Player Module

### State
```rust
pub struct Player {
    pub x, y, z: f32;       // Position (world coordinates)
    pub size: f32;            // Collision half-size
    pub vel_x, vel_y, vel_z: f32;
    pub grounded: bool;       // Standing on a platform?
    pub config: PlayerConfig; // Tunable physics params
}
```

### Update Order (Critical!)
1. Horizontal acceleration from input (or friction if no input)
2. Gravity applied to vel_y
3. Ground check (y < 0 → snap to 0, grounded=true)
4. Jump (if pressed AND grounded): set vel_y = jump_force, grounded=false
5. Depth movement (Z axis) — acceleration or friction
6. Position update: x += vel_x*dt, y += vel_y*dt, z += vel_z*dt
7. Speed clamping on X and Z

**Key rule:** Jump only works when `grounded=true`. Friction applies whenever there's no horizontal input.

## World Module

### Update Loop (`update_all`)
Two-pass approach:
1. **Move all players** — calls `entity.update()` for each
2. **Resolve collisions** — for each entity against every platform:
   - Vertical collision first (platform landing) → snap to top, zero vel_y, grounded=true
   - Horizontal collision next (wall hitting) → snap to edge, zero vel_x

### Entity Management
```rust
world.add_entity(player);     // Add player
world.remove_entity(index);   // Remove by index
world.entities[0].clone();   // Access player state for display/debug
```

## Input System

### Input Struct
```rust
pub struct Input {
    pub left: bool;      // A key — move left
    pub right: bool;     // D key — move right
    pub jump: bool;      // W key — jump (pressed only)
    pub forward: bool;   // S key — move toward camera (+Z)
    pub backward: bool;  // Space — move away from camera (-Z)
}
```

### GUI Implementation
Uses macroquad's `is_key_down()` / `is_key_pressed()`. Only compiled with `gui` feature.

## Platform Module

### AABB Trait
```rust
pub trait Aabb {
    fn min_x(&self) -> f32;  fn max_x(&self) -> f32;
    fn min_y(&self) -> f32;  fn max_y(&self) -> f32;
    fn min_z(&self) -> f32;  fn max_z(&self) -> f32;
}
```

Implemented by: `Platform`, `Player`, and test doubles.

### Platform Struct
```rust
pub struct Platform {
    pub x, y, z: f32;           // Center position
    pub width, height, depth: f32;  // Dimensions
    pub is_static: bool;          // Unused (always immovable)
}
```

## Projectiles (Not Integrated Yet)

- Exists in `projectile/mod.rs` but **not wired into World update loop**
- Has lifetime-based expiry and collision detection
- Can be tested independently using `Projectile::update()` and `is_alive()`

## Testing Patterns

### Unit Tests
All modules have comprehensive tests under `#[cfg(test)]`:
- **physics:** 14+ tests — gravity, jump, friction, acceleration, clamping, collisions
- **player:** 12+ tests — movement, jumping, config, grounded state
- **platform:** 9+ tests — AABB bounds, collision on all axes
- **projectile:** 10+ tests — lifecycle, movement, collision detection
- **world:** 6+ tests — entity management, multi-entity updates

### Testing with Headless Mode
```rust
// Test a specific sequence
let mut world = World::new();
world.add_platform(Platform::new(0.0, -0.25, 0.0, 100.0, 0.5, 100.0));
world.add_entity(Player::default());

// Simulate frames
for _ in 0..60 { // ~1 second
    let input = Input { forward: true, ..Default::default() };
    world.update_all(&input, &PhysicsImpl::new(), 0.016);
}
assert!(world.entities[0].z > 0.0); // Should have moved forward
```

## Common Pitfalls

### 1. Input parsing expects digits after each key
`"d2a3"` means d held 2s, a held 3s. `"d"` alone means 1 second (default).

### 2. Jump only works when grounded
After jumping, `grounded` becomes false until landing on a platform again. No double-jump.

### 3. Horizontal collision snaps player to edge
When hitting a wall: `vel_x = 0` and position is clamped to platform edge. Player doesn't slide.

### 4. Collision detection uses 0.5 unit tolerance for landing
Player must be within 0.5 units of platform top to land. Adjust in `resolve_platform_collision`.

### 5. Projectile not integrated into world loop
Don't expect projectiles to update automatically. Must call `projectile.update()` manually or integrate into World.

## Debugging Tips

1. **Use headless mode for deterministic testing** — same input always produces same output
2. **Print positions every frame** during development: add debug prints in `world::update_all`
3. **Check collision bounds** — verify AABB coordinates with `player.min_x()`, `platform.max_y()` etc.
4. **Friction threshold** — if velocity doesn't reach zero, check `friction_threshold` setting

## Feature Flags

- `gui` (default) — macroquad GUI mode
- `cli` — enables CLI headless mode with runtime `--cli` flag check

```bash
# GUI mode:
cargo run

# Headless mode:
cargo run --features "gui,cli" -- --cli d3w2
cargo run --no-default-features -- d3w2
```
