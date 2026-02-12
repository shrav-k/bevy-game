//! Turn-Based Grid Game - Learning Bevy ECS
//!
//! This is a simple turn-based tactical game built to learn Bevy's
//! Entity Component System (ECS) architecture.
//!
//! Phase 4: Turn-based movement

use bevy::prelude::*;

// Module declarations
mod components;
mod constants;
mod resources;
mod systems;

// Import the items we need
use resources::{EnemyTurnTimer, GridMap, SelectionState, TurnManager};
use systems::*;

// ===== STATE DEFINITIONS =====

/// Main application states - controls the overall game flow
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,   // Main menu screen (press Enter to start)
    GamePlay,   // Active gameplay
}

/// Turn states - controls whose turn it is
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum TurnState {
    #[default]
    PlayerTurn,  // Player's turn to move units
    EnemyTurn,   // Enemy's turn (AI controlled)
}

fn main() {
    App::new()
        // Add default Bevy plugins (rendering, input, windowing, etc.)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Turn-Based Tactics - Phase 4: Turns".to_string(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        // Initialize state machines
        .init_state::<AppState>()
        .init_state::<TurnState>()
        // Initialize resources (global state)
        .init_resource::<GridMap>()
        .init_resource::<SelectionState>()
        .init_resource::<TurnManager>()
        .init_resource::<EnemyTurnTimer>()
        // Startup systems (run once at the beginning, regardless of state)
        .add_systems(Startup, setup_camera)
        // Systems that run when entering MainMenu state
        .add_systems(OnEnter(AppState::MainMenu), setup_main_menu)
        // Systems that run when exiting MainMenu state
        .add_systems(OnExit(AppState::MainMenu), cleanup_main_menu)
        // Systems that run when entering GamePlay state
        .add_systems(
            OnEnter(AppState::GamePlay),
            (setup_grid, center_camera, spawn_units, setup_turn_ui).chain(),
        )
        // Systems for turn initialization
        .add_systems(OnEnter(TurnState::PlayerTurn), start_player_turn)
        .add_systems(OnEnter(TurnState::EnemyTurn), start_enemy_turn)
        // Systems that run every frame during MainMenu
        .add_systems(Update, menu_input_system.run_if(in_state(AppState::MainMenu)))
        // Systems that run every frame during GamePlay
        .add_systems(
            Update,
            (
                unit_selection_system,
                highlight_selected_system,
                highlight_movement_system,
                movement_system,
                check_turn_end_system,
                update_turn_ui_system,
                camera_pan_system,
            )
                .run_if(in_state(AppState::GamePlay)),
        )
        // Run the app!
        .run();
}
