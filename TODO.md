# King of the Hill — Implementation Plan

## Design Overview

- **2-4 players**, first to **10 points** wins
- **King of the Hill** mode: stand on the hill platform to earn 1 point/second
- **Hill teleports** to a new position every 15 seconds
- **Power-ups** spawn at arena edges on a timer
- **Mario Kart-style split-screen** camera (1P=full, 2P=left/right, 3P=top/bottom, 4P=2x2 grid)
- **Dual input**: gamepad (primary) + keyboard/mouse fallback
- **GUI-only** (headless mode removed)

---

## Phase 1: Arena Redesign ✅ DONE

Compact arena with 14 obstacles via `create_arena()`.

**Arena layout:**
- Ground floor: 60x0.5x60 flat ground
- Central hill: 5x0.5x5 (teleports every 15s)
- 2 high platforms (left/right at y=3)
- 4 boundary walls (red)
- 4 cover blocks (gray, 3x3x3)
- 2 side ramps

**Helper functions:** `arena_bounds()`, `hill_default_position()`, `hill_default_dimensions()`, `high_platform_positions()`, `cover_block_positions()`, `side_ramp_positions()`

---

## Phase 2: Input System ✅ DONE

- `PlayerInput` struct with analog movement (`move_x`, `move_z`), `jump`, `shoot`, camera controls
- `GamepadInput` reads from macroquad's built-in gamepad module (SDL2/Steam Input transparent)
- Keyboard fallback for P0: WASD + Space (jump) + mouse right-click drag (camera)
- Deadzone: 0.2 for analog sticks
- Per-player camera controls: gamepad right stick → camera yaw/pitch

**`PlayerInput` fields:**
```rust
pub move_x: f32,      // -1.0 to 1.0
pub move_z: f32,      // -1.0 to 1.0
pub jump: bool,
pub shoot: bool,
pub camera_yaw_speed: f32,
pub camera_pitch_speed: f32,
```

---

## Phase 3: Hill System ✅ DONE

- `Hill` struct: position, dimensions, `timer` (15s teleport interval), `score_timer`, `player_on_hill`
- Teleports to random valid position within arena bounds every 15s
- Player standing on hill earns **1 point/second** (`POINT_TIME = 1.0`)
- `WINNING_SCORE = 10`
- Hill obstacle added to world for collision; lifecycle managed in `World::update_all()`
- Hill indicator: pulsing yellow circle on ground plane

---

## Phase 4: Power-ups ✅ DONE

**`PowerUpKind` variants:**

| Kind | Effect | Duration | Color |
|------|--------|----------|-------|
| SpeedBoost | 2x movement speed | 5s | Cyan |
| DoubleJump | Can jump while airborne | 5s | Green |
| BiggerHill | Hill platform width x2 | 8s | Gold |
| Shield | Absorbs one hit | 10s | White |

**Spawning:**
- 4 spawn points at arena corners
- Spawn timer: 10s between spawns
- Only one power-up active at a time
- Respawn after collection: 15s
- Visual: color-coded pulsing cubes (`sin(time * 4.0)`)

**Player state:**
```rust
pub active_powerup: Option<PowerUpKind>,
pub powerup_timer: f32,
pub double_jump_used: bool,
```

---

## Phase 5: Game States ✅ DONE

- `GameState` enum: `Menu`, `Playing`, `GameOver`
- `GameStateManager` handles transitions
- Menu: "King of the Hill" title, "Press Space to start", gamepad count, win condition
- Game Over: winner announcement, final score, "Press Space to restart"
- Jump to start/restart: Space (keyboard) or A button (gamepad)

---

## Phase 6: UI / HUD ✅ DONE

- `draw_menu_hud()`: title, start prompt, gamepad count
- `draw_playing_hud()`: per-player scores, hill teleport timer, active power-up status
- `draw_game_over_hud()`: winner, final score, restart prompt
- HUD renders in 2D overlay (not per-camera)

---

## Phase 7: Camera ✅ DONE

- `PlayerCamera` struct: `target_x`, `target_z`, `yaw`, `pitch`, smooth lerp follow
- `build_camera(players, camera, viewport)` — Mario Kart-style split-screen
- Layouts: 1P=full screen, 2P=left/right split, 3P=top/bottom split, 4P=2x2 grid
- Uses `Camera3D.viewport` for per-player viewports
- Pitch clamped to `[-1.2, 1.2]`
- Per-player camera controls: gamepad right stick, mouse right-click drag (P0)

---

## Phase 8: Visual Polish ✅ DONE

- Boundary walls: **red** (`[1.0, 0.2, 0.2]`)
- Cover blocks: **gray** (`[0.5, 0.5, 0.5]`)
- Hill platform: **gold** (`[1.0, 0.84, 0.0]`)
- Hill indicator: pulsing yellow circle on ground (24 segments, `sin(time * 3.0)`)
- Power-up cubes: kind-specific colors with pulsing animation
- Platforms: **green** (`[0.2, 0.8, 0.2]`)
- `draw_world_in_camera(camera, world, total_time)` — `total_time` drives animations

---

## Phase 9: E2E Tests ✅ DONE

18 integration tests in `src/e2e/mod.rs`:
- Movement: forward/backward/left/right
- Jumping: basic jump, double jump, grounded check
- Hill scoring: stand on hill earns points
- Hill teleport: position changes after timer expires
- Power-up collection: pickup, effect applied, expiration
- Game state transitions: menu → playing → game over → restart
- Winning: first player to 10 points wins

Helper: `build_game_world()`, `simulate(frames, input_fn)` frame runner

---

## Implementation Order

1. **Arena redesign** — build the arena, test it ✅
2. **Input system** — gamepad + keyboard, dual player ✅
3. **Hill system** — teleporting hill, scoring ✅
4. **Power-ups** — spawn, collect, effects ✅
5. **Game states** — menu, playing, game over ✅
6. **Camera** — split-screen, per-player tracking ✅
7. **UI/HUD** — scores, indicators ✅
8. **Visual polish** — colors, effects ✅
9. **E2E tests** — integration test suite ✅

**All phases complete. 203 tests pass. Build clean.**

---

## Key Files

| File | Purpose |
|------|---------|
| `src/main.rs` | Game loop, split-screen rendering, HUD, visual polish |
| `src/camera/mod.rs` | `PlayerCamera`, `build_camera()` with split-screen layouts |
| `src/gamestate/mod.rs` | `GameState`, `GameStateManager` |
| `src/powerup/mod.rs` | `PowerUpKind`, `PowerUp`, spawn/collection logic |
| `src/hill/mod.rs` | `Hill` struct, scoring, teleportation |
| `src/arena/mod.rs` | `create_arena()`, helper functions |
| `src/world/mod.rs` | `World`, `update_all()`, hill/powerup integration |
| `src/input/mod.rs` | `PlayerInput`, `GamepadInput`, `MacroquadInput` |
| `src/player/mod.rs` | `Player`, power-up state, double jump |
| `src/e2e/mod.rs` | 18 integration tests |
| `src/obstacle/mod.rs` | `Obstacle`, `ObstacleKind`, `Aabb` trait |
