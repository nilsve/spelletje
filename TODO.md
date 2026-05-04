# King of the Hill — Implementation Plan

## Design Overview

- **2 players**, first to **10 points** wins
- **King of the Hill** mode: stand on the hill platform to earn 1 point/second
- **Hill teleports** to a new position every 15 seconds
- **Power-ups** spawn at arena edges on a timer
- **Single camera** tracking the center of both players
- **Controller only** (no keyboard/mouse)

---

## Phase 1: Arena Redesign (Testing Foundation)

Build a compact, interesting arena using the existing obstacle/platform system.

### Arena Layout

```
                    (high platform)
                        ===
              (side platform)
              ===           ===
         (central hill)
              ===
           (ground floor)
    =========================================
```

**Specific arena design:**

- **Ground floor:** `Platform::new(0, -0.25, 0, 60, 0.5, 60)` — large flat ground
- **Central hill:** `Platform::new(0, 0.5, 0, 5, 0.5, 5)` — the hill platform (will move)
- **Left high platform:** `Platform::new(-12, 3, 0, 6, 0.5, 4)` — elevated, accessible by jumping
- **Right high platform:** `Platform::new(12, 3, 0, 6, 0.5, 4)` — elevated, accessible by jumping
- **Back wall:** `Obstacle::solid(0, 2, -20, 60, 4, 1)` — boundary wall
- **Front wall:** `Obstacle::solid(0, 2, 20, 60, 4, 1)` — boundary wall
- **Left wall:** `Obstacle::solid(-25, 2, 0, 1, 4, 40)` — boundary wall
- **Right wall:** `Obstacle::solid(25, 2, 0, 1, 4, 40)` — boundary wall
- **Corner obstacles (cover):**
  - `Obstacle::solid(-8, 1.5, -8, 3, 3, 3)` — left-back cover block
  - `Obstacle::solid(8, 1.5, -8, 3, 3, 3)` — right-back cover block
  - `Obstacle::solid(-8, 1.5, 8, 3, 3, 3)` — left-front cover block
  - `Obstacle::solid(8, 1.5, 8, 3, 3, 3)` — right-front cover block
- **Side ramps (optional):**
  - `Platform::new(-18, 1.5, 0, 4, 0.5, 3)` — left side ramp (one-way)
  - `Platform::new(18, 1.5, 0, 4, 0.5, 3)` — right side ramp (one-way)

### Arena Testing Checklist

- [ ] All platforms render correctly
- [ ] Players can jump between all levels
- [ ] Boundary walls prevent players from leaving arena
- [ ] Cover blocks provide meaningful cover (not too small, not too big)
- [ ] High platforms are reachable by jumping from ground
- [ ] Arena fits comfortably on one screen (camera at ~15 units height)
- [ ] Arena is not too cramped or too empty

---

## Phase 2: Input System (Controller Only)

### 2a. Gamepad Support

**Leverage macroquad's built-in gamepad module** (not Steam Input directly). Steam Input works transparently with SDL2, which macroquad uses under the hood.

- macroquad has `gamepad::gamepad_is_connected()`, `gamepad::gamepad_is_button_down()`, `gamepad::gamepad_left_stick()`
- Steam Input maps controllers to standard gamepad inputs automatically — no extra setup needed
- On Steam Deck / Steam Big Picture: controllers work out of the box
- On non-Steam: standard Xbox/PS controllers work via SDL2 gamepad API

### 2b. Extend Input System

**Current `Input` struct:**
```rust
pub struct Input {
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub forward: bool,
    pub backward: bool,
    pub shoot: bool,
}
```

**New structure:**
```rust
pub struct PlayerInput {
    pub move_x: f32,      // -1.0 to 1.0 (left stick or D-pad)
    pub move_z: f32,      // -1.0 to 1.0 (left stick or D-pad)
    pub jump: bool,
    pub shoot: bool,
}

pub struct GameInput {
    pub player1: PlayerInput,
    pub player2: PlayerInput,
}
```

### 2c. Input Mappings

**Player 1 (Gamepad 1):**
- Left stick: movement (X and Z axis)
- A button: jump
- Left trigger / B button: shoot

**Player 2 (Gamepad 2):**
- Right stick (or D-pad): movement
- X button: jump
- Right trigger: shoot

**Deadzone:** 0.2 for analog sticks (ignore small drift)

### 2d. Input Source Implementation

- `GamepadInputSource` struct implementing `InputSource` trait
- `read()` returns `GameInput` with both players' inputs
- Deadzone handling for analog sticks
- Button polling for both connected gamepads

---

## Phase 3: Hill System

### 3a. Hill Struct

```rust
pub struct Hill {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub timer: f32,           // seconds until next teleport
    pub interval: f32,        // default 15.0 seconds
    pub score_timer: f32,     // seconds spent on hill
    pub player_on_hill: Option<usize>,  // player index
    pub next_position: Option<(f32, f32, f32)>,  // for visual indicator
}
```

### 3b. Hill Behavior

- Hill teleports every 15 seconds to a random valid position
- Valid positions: within arena bounds, on top of existing platforms, not inside obstacles
- When hill teleports, show a visual indicator of where it will appear
- Player standing on hill earns 1 point per second
- If no player on hill, score timer resets
- If player leaves hill, that player's score is locked until they reclaim it

### 3c. World Integration

- `World` tracks `scores: [f32; 2]` (player scores as floats, display as ints)
- `update_all` checks which player is on hill each frame
- Add hill platform to obstacle list for collision
- `winning_player()` returns `Option<usize>` when someone reaches 10 points

---

## Phase 4: Power-ups

### 4a. Power-up Types

| Type | Effect | Duration | Icon/Color |
|------|--------|----------|------------|
| Speed Boost | 2x movement speed | 5s | Cyan cube |
| Double Jump | Can jump while airborne | 5s | Green cube |
| Bigger Hill | Hill platform width x2 | 8s | Gold cube |
| Shield | Absorbs one hit | 10s | White cube |

### 4b. PowerUp Struct

```rust
pub struct PowerUp {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub kind: PowerUpKind,
    pub active: bool,
    pub lifetime: f32,      // seconds until despawn
    pub respawn_timer: f32, // seconds until next spawn at this location
}
```

### 4c. Power-up Spawning

- 4 spawn points at arena edges (corners)
- Spawn timer: 10 seconds between spawns
- Only one power-up active at a time
- Visual indicator: spinning cube at spawn point
- When collected, power-up respawns after 15 seconds

### 4d. Player Power-up State

```rust
// Add to Player struct:
pub active_powerup: Option<PowerUpKind>,
pub powerup_timer: f32,
```

---

## Phase 5: Game States

### 5a. State Enum

```rust
pub enum GameState {
    Menu,
    Playing,
    GameOver,
}
```

### 5b. Menu Screen

- Title: "King of the Hill"
- "Press A to start" for each player
- Show connected gamepad count
- Show win condition (first to 10)

### 5c. Game Over Screen

- Winner announcement
- Final score
- "Press A to restart"

---

## Phase 6: UI / HUD

### 6a. In-Game HUD

- Score display: "P1: X | P2: Y" (large text, top of screen)
- Hill position indicator (arrow showing direction to hill)
- Active power-up icons for each player
- Timer until next hill teleport

### 6b. Hill Indicator

- Arrow pointing toward hill position
- Distance text
- Dashed line to hill

---

## Phase 7: Camera

### 7a. Camera Behavior

- Track the center point between both players
- Smooth follow (lerp) to avoid jitter
- Fixed height at ~15 units above ground
- Orthographic projection (current setting)
- FOV wide enough to show entire arena

---

## Phase 8: Arena Visual Polish

### 8a. Visual Improvements

- Grid lines on ground for depth perception
- Colored boundary walls (red)
- Hill platform highlighted (gold color)
- Hill indicator on ground (circle showing hill position)
- Power-up spawn points (pulsing ring)
- Arena floor color (darker gray for contrast)

---

## Implementation Order

1. **Arena redesign** — build the arena, test it
2. **Input system** — gamepad support, dual player
3. **Hill system** — teleporting hill, scoring
4. **Game states** — menu, playing, game over
5. **Power-ups** — spawn, collect, effects
6. **Camera** — dual player tracking
7. **UI/HUD** — scores, indicators
8. **Visual polish** — colors, effects
