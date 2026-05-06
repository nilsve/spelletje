# spelletje-mac — King of the Hill Game

Rust 3D multiplayer arena game. King of the Hill mode, 2-4 players, first to 10 points wins. GUI-only (macroquad 0.4). Split-screen camera.

## Architecture

```
main.rs ── GUI game loop (macroquad async)
           ├─ Menu → Playing → GameOver state machine
           ├─ Split-screen rendering (per-player Camera3D viewports)
           └─ HUD overlay (scores, timers, power-up status)

src/
  ├── lib.rs           — Re-exports all modules
  ├── main.rs          — Entry point, game loop, rendering, HUD
  ├── arena/mod.rs     — Arena generation: create_arena(), helper functions
  ├── camera/mod.rs    — PlayerCamera, split-screen build_camera()
  ├── e2e/mod.rs       — 18 integration tests (test-only module)
  ├── enemy/mod.rs     — Enemy AI (not currently active in arena)
  ├── gamestate/mod.rs — GameState (Menu/Playing/GameOver), GameStateManager
  ├── hill/mod.rs      — Hill: scoring, teleportation, WINNING_SCORE=10
  ├── input/mod.rs     — PlayerInput, GameInput, GamepadInput, MacroquadInput
  ├── obstacle/mod.rs  — Obstacle, ObstacleKind (Solid/Platform), Aabb trait, is_colliding()
  ├── physics/mod.rs   — Physics, EntityPhysicsData, PhysicsEntity trait, collision resolution
  ├── player/mod.rs    — Player entity, gun/shooting, power-up state
  ├── powerup/mod.rs   — PowerUpKind, PowerUp, spawn/respawn/collection
  ├── projectile/mod.rs — Projectile (lifetime, collision, not auto-integrated into hill)
  ├── shooter/mod.rs   — Shooter trait
  └── world/mod.rs     — World: players, obstacles, projectiles, enemies, hill, powerups
```

## Running

```bash
cargo run          # Launch game (GUI)
cargo test         # Run all tests (203 tests)
```

No feature flags. No headless mode.

---

## Physics System

### EntityPhysicsData
Shared positioning data embedded in all physics entities:
```rust
pub struct EntityPhysicsData {
    pub x, y, z: f32,
    pub vel_x, vel_y, vel_z: f32,
    pub size: f32,
    pub is_grounded: bool,
}
```

### PhysicsEntity Trait
Implemented by `Player`, `Enemy`, `Projectile`. Provides `physics_data()`, `physics_data_mut()`, `update_position(dt)`.

### Physics Struct
Wraps `GlobalPhysicsConfig`. Methods:
- `apply_gravity(vel_y, dt)` — decreases vel_y
- `apply_jump(vel_y)` — sets vel_y = jump_force
- `apply_friction(vel, dt)` — reduces velocity toward zero
- `apply_acceleration_x/y/z(entity_data, input, dt)` — adds velocity
- `clamp_speed(vel)` — caps at max_speed
- `update(entity, obstacles, dt)` — full collision resolution pass
- `resolve_platform_collision(player, vel_y, obstacle, kind)` → `CollisionResult` (Bottom/Top/None)
- `resolve_horizontal_collision(player, vel_x, obstacle, kind)` → `CollisionResult` (Left/Right/None)

### GlobalPhysicsConfig
| Field | Default | Purpose |
|-------|---------|---------|
| gravity | 20.0 | Downward acceleration/s |
| jump_force | 10.0 | Upward velocity on jump |
| max_speed | 5.0 | Horizontal speed cap |
| acceleration | 20.0 | Acceleration rate/s |
| friction | 5.0 | Velocity reduction rate |
| friction_threshold | 0.01 | Below this, velocity = 0 |
| landing_tolerance | 0.2 | Platform landing tolerance (0.5 for Solid) |

### Collision
- **Platform (one-way):** Landing only when falling (`vel_y < 0`), within `landing_tolerance`. No horizontal collision.
- **Solid (full):** Landing, head bump (Top), horizontal (Left/Right). Full AABB.
- `is_colliding(a, b)` — generic AABB overlap check for any `Aabb` types.

---

## Player Module

### State
```rust
pub struct Player {
    pub jump_force: f32,
    physics_data: EntityPhysicsData,
    pub angle: f32,
    pub gun_pitch: f32,
    pub is_dead: bool,
    pub active_powerup: Option<PowerUpKind>,
    pub powerup_timer: f32,
    pub double_jump_used: bool,
}
```

### Update Order (`update(input, physics, dt)`)
1. Power-up timer countdown
2. Horizontal acceleration (or friction if no input) — X axis
3. Depth movement (or friction) — Z axis
4. Gravity on vel_y
5. Jump (if pressed AND grounded, OR double jump active)
6. Reset double_jump_used when grounded
7. Clamp speed on X and Z

**Key:** SpeedBoost doubles movement. DoubleJump allows mid-air jump. `double_jump_used` resets on ground.

### Shooting
- Implements `Shooter` trait: `shoot_origin()`, `projectile_speed()`, `damage()`
- `gun_end()` → 3D position at gun tip
- `shoot_direction()` → normalized aim vector
- `fire()` → creates `Projectile` at gun tip

---

## World Module

### State
```rust
pub struct World {
    pub players: Vec<Player>,
    pub obstacles: Vec<Obstacle>,
    pub projectiles: Vec<Projectile>,
    pub enemies: Vec<Enemy>,
    pub hill: Hill,
    pub powerups: Vec<PowerUp>,
}
```

### Update Loop (`update_all(player_inputs, physics, dt)`)
1. Update each player with corresponding input
2. Update enemies (AI + shooting)
3. `update_position(dt)` for all entities
4. `physics.update()` for all entities against obstacles
5. `update_hill(dt)` — check holder, score, teleport
6. `update_powerups(dt)` — timers, collection
7. Remove dead enemies
8. `update_projectiles(dt)` — move, collision, cleanup

### Hill Update
- Checks which player stands on hill platform (AABB overlap + Y tolerance)
- `hill.register_holder(idx, dt)` — accumulates timer, awards points at POINT_TIME (1s)
- `hill.update_teleport(dt)` — returns true every 15s
- On teleport: removes old hill obstacle, picks new position, adds new obstacle

### Power-up Collection
- Checks each player against each active power-up
- On collect: sets `player.active_powerup`, `player.powerup_timer`
- BiggerHill doubles `hill.width` and `hill.depth`

---

## Input System

### PlayerInput
```rust
pub struct PlayerInput {
    pub move_x: f32,          // -1.0 to 1.0 (left stick / A-D keys)
    pub move_z: f32,          // -1.0 to 1.0 (left stick / W-S keys)
    pub jump: bool,           // A button / Space key
    pub shoot: bool,          // Left trigger / Left mouse
    pub camera_yaw_speed: f32,   // Right stick X
    pub camera_pitch_speed: f32, // Right stick Y
}
```

### GameInput
```rust
pub struct GameInput {
    pub players: Vec<PlayerInput>,  // Up to MAX_GAMEPAD_PLAYERS (8)
}
```

### Keyboard Mapping (P0 fallback)
- A/D: left/right (`move_x`)
- W/S: forward/backward (`move_z`)
- Space: jump (pressed, not held)
- Left mouse: shoot
- Right mouse drag: camera yaw/pitch

### Gamepad
- Left stick: movement
- Right stick: camera rotation
- South button (A): jump
- Left trigger: shoot
- Deadzone: 0.2
- Uses `gilrs` crate for gamepad polling

### Legacy Input
Old `Input` struct (bool-based) still exists for backward compatibility. `From<&Input> for PlayerInput` converts it.

---

## Obstacle Module

### ObstacleKind
- `Solid` — full AABB collision (walls, floors, ceilings)
- `Platform` — one-way: land on top when falling, no horizontal collision

### Obstacle
```rust
pub struct Obstacle {
    pub x, y, z: f32,
    pub width, height, depth: f32,
    pub kind: ObstacleKind,
}
```
Center-positioned. `Aabb` bounds computed from center ± half-dimensions.

### Aabb Trait
```rust
pub trait Aabb {
    fn min_x(&self) -> f32;  fn max_x(&self) -> f32;
    fn min_y(&self) -> f32;  fn max_y(&self) -> f32;
    fn min_z(&self) -> f32;  fn max_z(&self) -> f32;
}
```
Implemented by: `Obstacle`, `Player`, `PowerUp`, `Projectile`, test doubles.

---

## Hill System

- `WINNING_SCORE = 10`, `POINT_TIME = 1.0` (seconds)
- Hill teleports every 15 seconds to random position within arena bounds
- Player standing on hill earns 1 point per second
- `hill.holder` tracks current player; resets on teleport or player switch
- `hill.to_obstacle()` creates `Obstacle::platform()` for collision
- `hill.init_scores(n)` / `ensure_scores(n)` manage score vector

---

## Power-ups

| Kind | Effect | Duration | Color |
|------|--------|----------|-------|
| SpeedBoost | 2x movement speed | 5s | Cyan |
| DoubleJump | Mid-air jump | 5s | Green |
| BiggerHill | Hill width/depth x2 | 8s | Gold |
| Shield | Absorbs one hit | 10s | White |

4 spawn points at arena corners. Staggered initial spawn (2.5s intervals). 15s lifetime, 15s respawn.

---

## Camera

### PlayerCamera
Per-player camera with smooth follow:
```rust
pub struct PlayerCamera {
    target_x: f32, target_z: f32,
    yaw: f32, pitch: f32,
}
```

### Split-Screen
`build_camera(players, camera, viewport)` — Mario Kart-style:
- 1P: full screen
- 2P: left/right split
- 3P: top/bottom split
- 4P: 2x2 grid

Uses `Camera3D.viewport` field. Pitch clamped to `[-1.2, 1.2]`.

---

## Game States

`GameState` enum: `Menu`, `Playing`, `GameOver`
`GameStateManager` handles transitions. Space/A button starts game or restarts.

---

## Arena

`create_arena()` returns 14 obstacles:
- 1 ground floor (60x0.5x60 platform)
- 1 central hill (5x0.5x5 platform)
- 2 high platforms (y=3)
- 4 boundary walls (Solid)
- 4 cover blocks (3x3x3 Solid)
- 2 side ramps (platform)

`arena_bounds()` → `(min_x, max_x, min_z, max_z)` = `(-25, 25, -20, 20)`.

---

## Visual Rendering (main.rs)

`draw_world_in_camera(camera, world, total_time)` — `total_time` drives animations:
- Walls: **red** `[1.0, 0.2, 0.2]`
- Cover blocks: **gray** `[0.5, 0.5, 0.5]`
- Hill: **gold** `[1.0, 0.84, 0.0]`
- Hill indicator: pulsing yellow circle on ground (24 segments, `sin(time * 3.0)`)
- Power-ups: kind-specific colors with pulsing (`sin(time * 4.0)`)
- Platforms: **green** `[0.2, 0.8, 0.2]`

---

## Testing

### Unit Tests
All modules have `#[cfg(test)]` modules:
- **physics:** 19 tests — gravity, jump, friction, acceleration, clamping, collisions
- **player:** 18+ tests — movement, jumping, config, grounded, gun, shooting, respawn
- **obstacle:** 11 tests — creation, bounds, collision, AABB
- **powerup:** 16 tests — duration, lifecycle, collection, spawn, colors
- **hill:** 19 tests — scoring, teleport, holder, win condition
- **world:** 16 tests — entity/obstacle management, collision, projectiles
- **input:** 16 tests — PlayerInput, GameInput, deadzone, gamepad constants
- **projectile:** 10+ tests — lifecycle, movement, collision

### E2E Tests (`src/e2e/mod.rs`)
18 integration tests: movement, jumping, hill scoring, teleport, power-up collection, game state transitions, winning.

Helpers: `build_game_world()`, `simulate(frames, input_fn)`.

### Running
```bash
cargo test          # All 203 tests
cargo test e2e      # E2E tests only
```

---

## Common Pitfalls

1. **Jump only works when grounded** — `grounded` becomes false after jump. No double-jump unless DoubleJump power-up active.
2. **Platform vs Solid collision** — Platforms are one-way (land on top only). Solids have full AABB.
3. **Horizontal collision snaps to edge** — `vel_x = 0`, position clamped. No sliding.
4. **Hill obstacle lifecycle** — World removes old hill obstacle and adds new one on teleport. Don't add hill manually.
5. **Power-up BiggerHill** — doubles `hill.width`/`hill.depth` directly. Effect persists until power-up expires (no auto-reset in current code).
6. **Projectile not auto-integrated into hill** — `Projectile::update()` called via `world.update_projectiles()`, not in main `update_all` player loop.
7. **Player uses `EntityPhysicsData`** — Access position via `player.physics_data().x`, not `player.x`.
8. **Camera pitch clamped** — `[-1.2, 1.2]` to prevent upside-down camera.

---

## Dependencies

```toml
macroquad = "0.4"    # Game engine (rendering, input, audio)
gilrs = "0.11.1"     # Gamepad support
```

No feature flags. No optional dependencies.
