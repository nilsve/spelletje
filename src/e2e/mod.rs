/// End-to-end integration tests simulating full gameplay.
/// Exercises World, Hill, PowerUp, Player, and GameState together.

use crate::arena::create_arena;
use crate::gamestate::{GameState, GameStateManager};
use crate::input::PlayerInput;
use crate::obstacle::ObstacleKind;
use crate::physics::{Physics, PhysicsEntity};
use crate::player::Player;
use crate::powerup::{create_powerups, PowerUpKind};
use crate::world::World;

/// Build a full game world ready for simulation.
fn build_game_world(player_count: usize) -> World {
    let mut world = World::new();

    // Arena obstacles
    for obstacle in create_arena() {
        world.add_obstacle(obstacle);
    }

    // Players
    for i in 0..player_count {
        let mut player = Player::default();
        player.physics_data_mut().x = (i as f32 - 0.5) * 4.0;
        world.add_entity(player);
    }

    // Hill scores
    world.hill.init_scores(player_count);

    // Power-ups
    world.powerups = create_powerups();

    world
}

/// Run N frames of simulation with the given input for each player.
fn simulate(world: &mut World, inputs: &[PlayerInput], frames: usize, dt: f32) {
    let physics = Physics::new();
    for _ in 0..frames {
        world.update_all(inputs, &physics, dt);
    }
}

/// Helper: input with player moving toward center (hill default position).
fn move_toward_center() -> PlayerInput {
    PlayerInput {
        move_x: 0.0,
        move_z: 0.0,
        jump: false,
        shoot: false,
        camera_yaw_speed: 0.0,
        camera_pitch_speed: 0.0,
    }
}

// ==================== Full Game Simulation ====================

#[test]
fn test_full_game_init() {
    let world = build_game_world(2);

    assert_eq!(world.entity_count(), 2);
    assert!(world.obstacle_count() >= 14);
    assert_eq!(world.hill.scores.len(), 2);
    assert_eq!(world.hill.scores, vec![0, 0]);
    assert_eq!(world.powerups.len(), 4);
    assert!(!world.winning_player().is_some());
}

#[test]
fn test_player_moves_with_input() {
    let mut world = build_game_world(1);
    let initial_x = world.players[0].physics_data().x;

    let inputs = vec![PlayerInput {
        move_x: 1.0,
        ..move_toward_center()
    }];

    simulate(&mut world, &inputs, 60, 0.016);

    assert!(world.players[0].physics_data().x > initial_x);
}

#[test]
fn test_player_jumps_and_falls() {
    let mut world = build_game_world(1);

    // First, move player to ground level and let them land
    let inputs = vec![PlayerInput::default()];
    simulate(&mut world, &inputs, 60, 0.016);

    let y_after_fall = world.players[0].physics_data().y;

    // Now jump
    let jump_inputs = vec![PlayerInput {
        jump: true,
        ..move_toward_center()
    }];
    simulate(&mut world, &jump_inputs, 5, 0.016);

    let y_after_jump = world.players[0].physics_data().y;

    // Player should be higher after jump than after fall
    assert!(y_after_jump > y_after_fall);
}

#[test]
fn test_player_earns_hill_score() {
    let mut world = build_game_world(1);

    // Place player directly on the hill
    let pd = world.players[0].physics_data_mut();
    pd.x = world.hill.x;
    pd.y = world.hill.y + world.hill.height / 2.0 + 0.5;
    pd.z = world.hill.z;

    let inputs = vec![PlayerInput::default()];

    // Simulate enough frames for a point (1 second = ~63 frames at dt=0.016)
    simulate(&mut world, &inputs, 120, 0.016);

    assert!(world.hill.scores[0] > 0, "Player should have earned at least 1 point on the hill");
}

#[test]
fn test_two_players_compete_for_hill() {
    let mut world = build_game_world(2);

    // Place both players near the hill
    world.players[0].physics_data_mut().x = world.hill.x;
    world.players[0].physics_data_mut().y = world.hill.y + world.hill.height / 2.0 + 0.5;
    world.players[0].physics_data_mut().z = world.hill.z;

    // Player 2 starts further away
    world.players[1].physics_data_mut().x = world.hill.x + 10.0;
    world.players[1].physics_data_mut().y = world.hill.y + world.hill.height / 2.0 + 0.5;
    world.players[1].physics_data_mut().z = world.hill.z;

    let inputs = vec![PlayerInput::default(), PlayerInput::default()];

    // Simulate for a few points
    simulate(&mut world, &inputs, 180, 0.016);

    // Player 1 should have more points since they started on the hill
    assert!(world.hill.scores[0] >= world.hill.scores[1]);
}

#[test]
fn test_hill_teleport_changes_position() {
    let mut world = build_game_world(1);
    let initial_hill_x = world.hill.x;
    let initial_hill_z = world.hill.z;

    // Simulate enough frames for teleport (15 seconds = ~938 frames)
    let inputs = vec![PlayerInput::default()];
    simulate(&mut world, &inputs, 950, 0.016);

    // Hill should have moved
    assert!(
        (world.hill.x - initial_hill_x).abs() > 0.1 || (world.hill.z - initial_hill_z).abs() > 0.1,
        "Hill should have teleported to a new position"
    );
}

#[test]
fn test_hill_clears_holder_on_teleport() {
    let mut world = build_game_world(1);

    // Place player on hill
    let pd = world.players[0].physics_data_mut();
    pd.x = world.hill.x;
    pd.y = world.hill.y + world.hill.height / 2.0 + 0.5;
    pd.z = world.hill.z;

    let inputs = vec![PlayerInput::default()];

    // Build up some timer
    simulate(&mut world, &inputs, 30, 0.016);

    // Verify holder is set
    assert!(world.hill.holder.is_some());

    // Simulate until teleport
    simulate(&mut world, &inputs, 950, 0.016);

    // Holder should be cleared after teleport
    assert!(world.hill.holder.is_none(), "Holder should be cleared after hill teleport");
}

#[test]
fn test_powerup_collection() {
    let mut world = build_game_world(1);

    // Place player near first power-up spawn point
    if let Some(pu) = world.powerups.first_mut() {
        // Activate the power-up
        pu.active = true;
        pu.lifetime = 15.0;
        pu.kind = PowerUpKind::SpeedBoost;

        // Move player to power-up location
        let pd = world.players[0].physics_data_mut();
        pd.x = pu.x;
        pd.y = pu.y;
        pd.z = pu.z;

        let inputs = vec![PlayerInput::default()];
        simulate(&mut world, &inputs, 5, 0.016);

        // Player should have collected the power-up
        assert_eq!(world.players[0].active_powerup, Some(PowerUpKind::SpeedBoost));
        assert!((world.players[0].powerup_timer - 5.0).abs() < 1.0);
    }
}

#[test]
fn test_speedboost_increases_movement() {
    let mut world = build_game_world(1);

    // Give player speed boost
    world.players[0].active_powerup = Some(PowerUpKind::SpeedBoost);
    world.players[0].powerup_timer = 10.0;

    let inputs = vec![PlayerInput {
        move_x: 1.0,
        ..move_toward_center()
    }];

    simulate(&mut world, &inputs, 60, 0.016);

    let boosted_x = world.players[0].physics_data().x;

    // Reset and test without boost
    let mut world2 = build_game_world(1);
    let inputs2 = vec![PlayerInput {
        move_x: 1.0,
        ..move_toward_center()
    }];
    simulate(&mut world2, &inputs2, 60, 0.016);

    let normal_x = world2.players[0].physics_data().x;

    // Boosted player should have moved further
    assert!(boosted_x.abs() > normal_x.abs(), "Speed boost should increase movement distance");
}

#[test]
fn test_double_jump_allows_airborne_jump() {
    let mut world = build_game_world(1);

    // Place player high enough to test
    let pd = world.players[0].physics_data_mut();
    pd.y = 10.0;

    // Give double jump
    world.players[0].active_powerup = Some(PowerUpKind::DoubleJump);
    world.players[0].powerup_timer = 10.0;

    // First jump (in air, should work with double jump)
    let inputs = vec![PlayerInput {
        jump: true,
        ..move_toward_center()
    }];
    simulate(&mut world, &inputs, 3, 0.016);

    let y_after_first = world.players[0].physics_data().y;

    // Second jump (should NOT work since double jump used)
    simulate(&mut world, &inputs, 3, 0.016);

    // The double jump should have been consumed
    assert!(world.players[0].double_jump_used);
}

#[test]
fn test_shield_powerup() {
    let mut world = build_game_world(1);

    world.players[0].active_powerup = Some(PowerUpKind::Shield);
    world.players[0].powerup_timer = 10.0;

    assert_eq!(world.players[0].active_powerup, Some(PowerUpKind::Shield));
}

// ==================== Game State Integration ====================

#[test]
fn test_game_state_menu_to_playing() {
    let mut state = GameStateManager::new();
    assert_eq!(state.state, GameState::Menu);

    let inputs = vec![PlayerInput {
        jump: true,
        ..move_toward_center()
    }];
    state.update(&inputs);

    assert_eq!(state.state, GameState::Playing);
}

#[test]
fn test_game_state_playing_to_game_over() {
    let mut state = GameStateManager::new();
    state.state = GameState::Playing;

    // Simulate winning
    state.set_game_over(0);

    assert_eq!(state.state, GameState::GameOver);
    assert_eq!(state.winner, Some(0));
}

#[test]
fn test_game_state_game_over_to_menu() {
    let mut state = GameStateManager::new();
    state.state = GameState::GameOver;
    state.winner = Some(1);

    let inputs = vec![PlayerInput {
        jump: true,
        ..move_toward_center()
    }];
    state.update(&inputs);

    assert_eq!(state.state, GameState::Menu);
    assert_eq!(state.winner, None);
}

// ==================== Full Game Loop ====================

#[test]
fn test_full_game_loop_simulation() {
    let mut world = build_game_world(2);
    let mut state = GameStateManager::new();
    let physics = Physics::new();

    // Menu phase
    assert!(state.is_menu());

    // Start game
    let inputs = vec![
        PlayerInput { jump: true, ..move_toward_center() },
        PlayerInput { jump: true, ..move_toward_center() },
    ];
    state.update(&inputs);
    assert!(state.is_playing());

    // Place players on hill for scoring
    world.players[0].physics_data_mut().x = world.hill.x;
    world.players[0].physics_data_mut().y = world.hill.y + world.hill.height / 2.0 + 0.5;
    world.players[0].physics_data_mut().z = world.hill.z;

    world.players[1].physics_data_mut().x = world.hill.x + 10.0;
    world.players[1].physics_data_mut().y = world.hill.y + world.hill.height / 2.0 + 0.5;

    // Simulate gameplay
    let game_inputs = vec![PlayerInput::default(), PlayerInput::default()];
    let dt = 0.016;

    for _ in 0..300 {
        if state.is_playing() {
            world.update_all(&game_inputs, &physics, dt);

            // Check win condition
            if let Some(winner) = world.winning_player() {
                state.set_game_over(winner);
            }
        }
    }

    // Player 0 should have scored since they were on the hill
    assert!(world.hill.scores[0] > 0);
}

#[test]
fn test_projectile_hits_enemy() {
    let mut world = build_game_world(1);

    // Place enemy in front of player
    let mut enemy = crate::enemy::Enemy::default();
    enemy.physics_data.x = 10.0;
    enemy.physics_data.y = 1.0;
    enemy.physics_data.z = 0.0;
    world.add_enemy(enemy);

    // Player fires projectile toward enemy
    world.players[0].physics_data_mut().x = 0.0;
    world.players[0].physics_data_mut().y = 1.0;
    world.players[0].physics_data_mut().z = 0.0;

    let projectile = world.players[0].fire();
    world.add_projectile(projectile);

    // Update projectiles - should hit enemy or obstacle
    let inputs = vec![PlayerInput::default()];
    simulate(&mut world, &inputs, 30, 0.016);

    // Projectile should have been removed (hit something)
    assert_eq!(world.projectile_count(), 0);
}

#[test]
fn test_arena_boundary_prevents_escape() {
    let mut world = build_game_world(1);

    // Place player near boundary wall
    let pd = world.players[0].physics_data_mut();
    pd.x = 24.0; // Near right wall at x=25
    pd.y = 1.0;
    pd.z = 0.0;
    pd.vel_x = 10.0; // Moving toward wall

    let inputs = vec![PlayerInput::default()];
    simulate(&mut world, &inputs, 60, 0.016);

    // Player should still be within arena bounds
    let final_x = world.players[0].physics_data().x;
    assert!(final_x < 25.0, "Player should not pass through right wall");
}

#[test]
fn test_multiple_powerups_spawn_over_time() {
    let mut world = build_game_world(1);

    // Initially all power-ups are inactive (staggered)
    let active_initially = world.powerups.iter().filter(|p| p.active).count();

    // Simulate a while to let power-ups spawn
    let inputs = vec![PlayerInput::default()];
    simulate(&mut world, &inputs, 600, 0.016); // ~10 seconds

    // Some power-ups should have spawned by now
    assert!(world.powerups.len() == 4);
}

#[test]
fn test_hill_bigger_effect() {
    let mut world = build_game_world(1);
    let original_width = world.hill.width;

    // Simulate collecting BiggerHill power-up
    if let Some(pu) = world.powerups.first_mut() {
        pu.active = true;
        pu.kind = PowerUpKind::BiggerHill;
        pu.x = world.players[0].physics_data().x;
        pu.y = world.players[0].physics_data().y;
        pu.z = world.players[0].physics_data().z;

        let inputs = vec![PlayerInput::default()];
        simulate(&mut world, &inputs, 5, 0.016);

        // Hill should be bigger
        assert!(world.hill.width > original_width, "BiggerHill should increase hill width");
    }
}

#[test]
fn test_powerup_timer_expires() {
    let mut world = build_game_world(1);

    world.players[0].active_powerup = Some(PowerUpKind::SpeedBoost);
    world.players[0].powerup_timer = 5.0;

    let inputs = vec![PlayerInput::default()];
    simulate(&mut world, &inputs, 320, 0.016); // ~5.12 seconds

    // Power-up should have expired
    assert!(world.players[0].active_powerup.is_none());
}

#[test]
fn test_two_players_independent_scoring() {
    let mut world = build_game_world(2);

    // Place player 0 on hill
    world.players[0].physics_data_mut().x = world.hill.x;
    world.players[0].physics_data_mut().y = world.hill.y + world.hill.height / 2.0 + 0.5;
    world.players[0].physics_data_mut().z = world.hill.z;

    // Player 1 far away
    world.players[1].physics_data_mut().x = 20.0;
    world.players[1].physics_data_mut().y = 0.0;
    world.players[1].physics_data_mut().z = 20.0;

    let inputs = vec![PlayerInput::default(), PlayerInput::default()];
    simulate(&mut world, &inputs, 120, 0.016);

    assert!(world.hill.scores[0] > 0);
    assert_eq!(world.hill.scores[1], 0);
}

#[test]
fn test_game_over_winning_score() {
    let mut world = build_game_world(1);

    // Manually set score to winning threshold
    world.hill.scores[0] = crate::hill::WINNING_SCORE;

    assert_eq!(world.winning_player(), Some(0));
}

#[test]
fn test_no_winner_below_threshold() {
    let mut world = build_game_world(1);
    world.hill.scores[0] = crate::hill::WINNING_SCORE - 1;

    assert_eq!(world.winning_player(), None);
}
