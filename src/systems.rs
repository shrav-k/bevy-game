// System definitions - where all the game logic lives
// Systems are just functions that operate on components and resources

use bevy::prelude::*;

use crate::components::{Faction, GridPosition, Hoverable, Selected, Tile, TurnStatus, Unit};
use crate::constants::*;
use crate::resources::{EnemyTurnTimer, GridMap, SelectionState, TurnManager};
use crate::{AppState, TurnState};

// ===== SETUP SYSTEMS =====

/// Sets up the 2D camera for the game
pub fn setup_camera(mut commands: Commands) {
    // Spawn a 2D camera
    // The camera will be centered at (0, 0) by default
    commands.spawn(Camera2d);

    info!("Camera spawned");
}

/// Creates the grid of tiles
/// This system runs once at startup to initialize the game board
pub fn setup_grid(mut commands: Commands, mut grid_map: ResMut<GridMap>) {
    info!(
        "Setting up grid: {}x{} tiles of size {}",
        GRID_WIDTH, GRID_HEIGHT, TILE_SIZE
    );

    // Iterate through all grid positions and spawn tile entities
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let grid_pos = GridPosition::new(x, y);

            // Calculate world position for this tile
            let world_pos = grid_map.grid_to_world(&grid_pos);

            // Checkerboard pattern for tile colors
            let color = if (x + y) % 2 == 0 {
                TILE_COLOR_LIGHT
            } else {
                TILE_COLOR_DARK
            };

            // Spawn the tile entity with all its components
            let tile_entity = commands
                .spawn((
                    Tile::new_grass(),
                    grid_pos,
                    Sprite {
                        color,
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        ..default()
                    },
                    Transform::from_xyz(world_pos.x, world_pos.y, Z_TILE),
                ))
                .id();

            // Register this tile in the grid map
            grid_map.register_tile(grid_pos, tile_entity);
        }
    }

    info!("Grid setup complete: {} tiles spawned", grid_map.tiles.len());
}

/// Centers the camera on the grid
/// Runs after grid setup to position camera properly
pub fn center_camera(
    grid_map: Res<GridMap>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        // Calculate the center of the grid in world coordinates
        let grid_center_x = (grid_map.width as f32 * grid_map.tile_size) / 2.0;
        let grid_center_y = (grid_map.height as f32 * grid_map.tile_size) / 2.0;

        // Position camera at grid center
        camera_transform.translation.x = grid_center_x;
        camera_transform.translation.y = grid_center_y;

        info!(
            "Camera centered at ({}, {})",
            grid_center_x, grid_center_y
        );
    }
}

// ===== CAMERA CONTROL SYSTEMS (for Phase 2) =====

/// Allows panning the camera with WASD or arrow keys
pub fn camera_pan_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    time: Res<Time>,
) {
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        let camera_speed = 300.0; // pixels per second
        let delta = camera_speed * time.delta_secs();

        // WASD or arrow keys for panning
        if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
            camera_transform.translation.y += delta;
        }
        if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
            camera_transform.translation.y -= delta;
        }
        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            camera_transform.translation.x -= delta;
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            camera_transform.translation.x += delta;
        }
    }
}

// ===== INPUT SYSTEMS (for Phase 2) =====

/// Detects mouse clicks and converts to grid coordinates
/// This will be expanded in Phase 2 to handle unit selection
pub fn mouse_input_system(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    grid_map: Res<GridMap>,
) {
    // Only process if left mouse button was just pressed
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    // Get the primary window
    let Ok(window) = windows.single() else {
        return;
    };

    // Get camera and its transform
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    // Get cursor position in window
    if let Some(cursor_pos) = window.cursor_position() {
        // Convert cursor position to world position
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            // Convert world position to grid coordinates
            let grid_pos = grid_map.world_to_grid(world_pos);

            // Check if click is within grid bounds
            if grid_map.is_in_bounds(&grid_pos) {
                info!(
                    "Clicked tile at grid position ({}, {}), world position ({:.1}, {:.1})",
                    grid_pos.x, grid_pos.y, world_pos.x, world_pos.y
                );
            } else {
                info!(
                    "Clicked outside grid at ({}, {})",
                    grid_pos.x, grid_pos.y
                );
            }
        }
    }
}

// ===== MAIN MENU SYSTEMS (Phase 2) =====

/// Marker component for main menu UI entities
#[derive(Component)]
pub struct MainMenuUI;

/// Sets up the main menu UI
/// Runs when entering MainMenu state
pub fn setup_main_menu(mut commands: Commands) {
    info!("Setting up main menu");

    // Spawn UI container
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            MainMenuUI,
        ))
        .with_children(|parent| {
            // Title text
            parent.spawn((
                Text::new("Turn-Based Tactics"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Instruction text
            parent.spawn((
                Text::new("Press ENTER to start"),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
            ));

            // Phase info text
            parent.spawn((
                Text::new("Phase 4: Turn-Based Movement"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.8, 0.5)),
                Node {
                    margin: UiRect::top(Val::Px(40.0)),
                    ..default()
                },
            ));
        });
}

/// Cleans up the main menu UI
/// Runs when exiting MainMenu state
pub fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuUI>>) {
    info!("Cleaning up main menu");

    // Despawn all menu UI entities (and their children)
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Handles input in the main menu
/// Pressing Enter transitions to GamePlay state
pub fn menu_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Enter) {
        info!("Starting game...");
        next_state.set(AppState::GamePlay);
    }
}

// ===== UNIT SYSTEMS (Phase 3) =====

/// Spawns initial units on the grid
/// Runs when entering GamePlay state
pub fn spawn_units(mut commands: Commands, grid_map: Res<GridMap>) {
    info!("Spawning units");

    // Spawn 2 player units (blue circles)
    let player_positions = vec![GridPosition::new(2, 2), GridPosition::new(3, 2)];

    for grid_pos in player_positions {
        let world_pos = grid_map.grid_to_world(&grid_pos);

        commands.spawn((
            Unit {
                faction: Faction::Player,
            },
            grid_pos,
            TurnStatus::default(), // Track if unit has acted this turn
            Sprite {
                color: PLAYER_COLOR,
                custom_size: Some(Vec2::new(UNIT_RADIUS * 2.0, UNIT_RADIUS * 2.0)),
                ..default()
            },
            Transform::from_xyz(world_pos.x, world_pos.y, Z_UNIT),
            Hoverable, // Can be hovered over with mouse
        ));
    }

    // Spawn 2 enemy units (red circles)
    let enemy_positions = vec![GridPosition::new(6, 7), GridPosition::new(7, 7)];

    for grid_pos in enemy_positions {
        let world_pos = grid_map.grid_to_world(&grid_pos);

        commands.spawn((
            Unit {
                faction: Faction::Enemy,
            },
            grid_pos,
            TurnStatus::default(), // Track if unit has acted this turn
            Sprite {
                color: ENEMY_COLOR,
                custom_size: Some(Vec2::new(UNIT_RADIUS * 2.0, UNIT_RADIUS * 2.0)),
                ..default()
            },
            Transform::from_xyz(world_pos.x, world_pos.y, Z_UNIT),
            Hoverable,
        ));
    }

    info!("Spawned 4 units (2 player, 2 enemy)");
}

/// Handles unit selection with mouse clicks
/// Only player units can be selected
pub fn unit_selection_system(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    grid_map: Res<GridMap>,
    unit_query: Query<(Entity, &GridPosition, &Unit), With<Hoverable>>,
    selected_query: Query<Entity, With<Selected>>,
    mut commands: Commands,
    mut selection_state: ResMut<SelectionState>,
) {
    // Only process if left mouse button was just pressed
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            let clicked_grid_pos = grid_map.world_to_grid(world_pos);

            // Find if there's a unit at the clicked position
            let mut clicked_unit: Option<(Entity, &Unit)> = None;
            for (entity, unit_grid_pos, unit) in &unit_query {
                if unit_grid_pos.x == clicked_grid_pos.x && unit_grid_pos.y == clicked_grid_pos.y
                {
                    clicked_unit = Some((entity, unit));
                    break;
                }
            }

            // If we clicked on a unit
            if let Some((entity, unit)) = clicked_unit {
                // Only allow selecting player units
                if unit.faction == Faction::Player {
                    // Deselect previously selected unit
                    for selected_entity in &selected_query {
                        commands.entity(selected_entity).remove::<Selected>();
                    }

                    // Select the new unit
                    commands.entity(entity).insert(Selected);
                    selection_state.select_unit(entity);

                    info!(
                        "Selected player unit at ({}, {})",
                        clicked_grid_pos.x, clicked_grid_pos.y
                    );
                } else {
                    info!(
                        "Clicked enemy unit at ({}, {}) - cannot select",
                        clicked_grid_pos.x, clicked_grid_pos.y
                    );
                }
            } else {
                // Clicked empty tile - deselect all
                for selected_entity in &selected_query {
                    commands.entity(selected_entity).remove::<Selected>();
                }
                selection_state.clear_selection();

                info!(
                    "Clicked empty tile at ({}, {}) - deselected all",
                    clicked_grid_pos.x, clicked_grid_pos.y
                );
            }
        }
    }
}

/// Marker component for selection visual indicators
#[derive(Component)]
pub struct SelectionRing;

/// Adds visual feedback for selected units
/// Spawns a yellow ring around selected units
/// Only updates when selection changes
pub fn highlight_selected_system(
    mut commands: Commands,
    selected_query: Query<Entity, (With<Unit>, With<Selected>)>,
    ring_query: Query<Entity, With<SelectionRing>>,
    selection_state: Res<SelectionState>,
) {
    // Only update if selection state changed
    if !selection_state.is_changed() {
        return;
    }

    // Remove old selection rings
    for ring_entity in &ring_query {
        commands.entity(ring_entity).despawn();
    }

    // Add selection ring to currently selected unit
    if let Some(selected_entity) = selection_state.selected_unit {
        // Check if unit still has Selected component
        if selected_query.get(selected_entity).is_ok() {
            // Spawn a selection ring as a child of the unit
            commands.entity(selected_entity).with_children(|parent| {
                parent.spawn((
                    Sprite {
                        color: SELECTED_COLOR,
                        custom_size: Some(Vec2::new(
                            SELECTION_RING_RADIUS * 2.0,
                            SELECTION_RING_RADIUS * 2.0,
                        )),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, Z_SELECTION - Z_UNIT), // Relative to parent
                    SelectionRing,
                ));
            });
        }
    }
}

// ===== TURN-BASED MOVEMENT SYSTEMS (Phase 4) =====

/// Marker component for movement highlight overlays
#[derive(Component)]
pub struct MovementHighlight;

/// Highlights valid movement tiles for the selected unit
/// Shows green overlay on adjacent tiles
pub fn highlight_movement_system(
    mut commands: Commands,
    selected_query: Query<(&GridPosition, &Unit), With<Selected>>,
    highlight_query: Query<Entity, With<MovementHighlight>>,
    selection_state: Res<SelectionState>,
    grid_map: Res<GridMap>,
    turn_state: Res<State<TurnState>>,
) {
    // Only update if selection changed
    if !selection_state.is_changed() {
        return;
    }

    // Remove old highlights
    for highlight_entity in &highlight_query {
        commands.entity(highlight_entity).despawn();
    }

    // Only show movement during player turn
    if *turn_state.get() != TurnState::PlayerTurn {
        return;
    }

    // Highlight valid moves for selected unit
    if let Some(selected_entity) = selection_state.selected_unit {
        if let Ok((grid_pos, _)) = selected_query.get(selected_entity) {
            // Get adjacent tiles (4-directional movement)
            let adjacent_positions = grid_pos.adjacent();

            for adj_pos in adjacent_positions {
                // Check if position is in bounds
                if grid_map.is_in_bounds(&adj_pos) {
                    let world_pos = grid_map.grid_to_world(&adj_pos);

                    // Spawn highlight overlay
                    commands.spawn((
                        Sprite {
                            color: MOVEMENT_HIGHLIGHT,
                            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                            ..default()
                        },
                        Transform::from_xyz(world_pos.x, world_pos.y, Z_OVERLAY),
                        MovementHighlight,
                    ));
                }
            }
        }
    }
}

/// Handles unit movement on click
/// Moves selected unit to adjacent tile if valid
pub fn movement_system(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    grid_map: Res<GridMap>,
    mut unit_query: Query<(&mut GridPosition, &mut Transform, &mut TurnStatus, &Unit), With<Selected>>,
    selection_state: Res<SelectionState>,
    turn_state: Res<State<TurnState>>,
) {
    // Only allow movement during player turn
    if *turn_state.get() != TurnState::PlayerTurn {
        return;
    }

    // Only process if left mouse button was just pressed
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            let clicked_grid_pos = grid_map.world_to_grid(world_pos);

            // Get the selected unit
            if let Some(selected_entity) = selection_state.selected_unit {
                if let Ok((mut unit_grid_pos, mut unit_transform, mut turn_status, _)) =
                    unit_query.get_mut(selected_entity)
                {
                    // Check if clicked position is adjacent
                    let adjacent_positions = unit_grid_pos.adjacent();
                    let is_adjacent = adjacent_positions
                        .iter()
                        .any(|pos| pos.x == clicked_grid_pos.x && pos.y == clicked_grid_pos.y);

                    if is_adjacent && grid_map.is_in_bounds(&clicked_grid_pos) {
                        // Move the unit
                        let new_world_pos = grid_map.grid_to_world(&clicked_grid_pos);

                        // Update grid position
                        *unit_grid_pos = clicked_grid_pos;

                        // Update world position
                        unit_transform.translation.x = new_world_pos.x;
                        unit_transform.translation.y = new_world_pos.y;

                        // Mark as acted
                        turn_status.has_acted = true;

                        info!(
                            "Unit moved to ({}, {})",
                            clicked_grid_pos.x, clicked_grid_pos.y
                        );
                    }
                }
            }
        }
    }
}

/// Checks if all units have acted and transitions turn
pub fn check_turn_end_system(
    unit_query: Query<(&Unit, &TurnStatus)>,
    turn_state: Res<State<TurnState>>,
    mut next_turn_state: ResMut<NextState<TurnState>>,
    time: Res<Time>,
    mut enemy_timer: ResMut<EnemyTurnTimer>,
) {
    match turn_state.get() {
        TurnState::PlayerTurn => {
            // Check if all player units have acted
            let all_player_acted = unit_query
                .iter()
                .filter(|(unit, _)| unit.faction == Faction::Player)
                .all(|(_, status)| status.has_acted);

            if all_player_acted {
                info!("All player units have acted - switching to enemy turn");
                next_turn_state.set(TurnState::EnemyTurn);
            }
        }
        TurnState::EnemyTurn => {
            // Tick the timer
            enemy_timer.timer.tick(time.delta());

            // Only check for turn end after timer finishes
            if enemy_timer.timer.just_finished() {
                // Check if all enemy units have acted
                let all_enemy_acted = unit_query
                    .iter()
                    .filter(|(unit, _)| unit.faction == Faction::Enemy)
                    .all(|(_, status)| status.has_acted);

                if all_enemy_acted {
                    info!("All enemy units have acted - switching to player turn");
                    next_turn_state.set(TurnState::PlayerTurn);
                }
            }
        }
    }
}

/// Resets turn status for player units at start of player turn
pub fn start_player_turn(mut unit_query: Query<(&Unit, &mut TurnStatus)>) {
    info!("Starting player turn");

    for (unit, mut status) in &mut unit_query {
        if unit.faction == Faction::Player {
            status.has_acted = false;
        }
    }
}

/// Resets turn status for enemy units at start of enemy turn
/// Also triggers AI to act
pub fn start_enemy_turn(
    mut unit_query: Query<(&Unit, &mut TurnStatus)>,
    mut enemy_timer: ResMut<EnemyTurnTimer>,
) {
    info!("Starting enemy turn");

    // Reset the timer
    enemy_timer.timer.reset();

    for (unit, mut status) in &mut unit_query {
        if unit.faction == Faction::Enemy {
            status.has_acted = false;
        }
    }

    // For Phase 4, enemy units just pass their turn after a delay
    // Phase 5 will add actual AI movement
    for (unit, mut status) in &mut unit_query {
        if unit.faction == Faction::Enemy {
            status.has_acted = true;
        }
    }

    info!("Enemy units will pass after delay (AI not implemented yet)");
}

// ===== TURN UI SYSTEMS (Phase 4) =====

/// Marker component for turn indicator UI
#[derive(Component)]
pub struct TurnIndicatorUI;

/// Sets up the turn indicator UI
pub fn setup_turn_ui(mut commands: Commands) {
    info!("Setting up turn UI");

    // Spawn turn indicator in top-left corner
    commands.spawn((
        Text::new("Player Turn"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(PLAYER_COLOR),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            ..default()
        },
        TurnIndicatorUI,
    ));
}

/// Updates turn indicator text based on current turn
pub fn update_turn_ui_system(
    mut query: Query<(&mut Text, &mut TextColor), With<TurnIndicatorUI>>,
    turn_state: Res<State<TurnState>>,
) {
    if !turn_state.is_changed() {
        return;
    }

    for (mut text, mut color) in &mut query {
        match turn_state.get() {
            TurnState::PlayerTurn => {
                **text = "Player Turn".to_string();
                *color = TextColor(PLAYER_COLOR);
            }
            TurnState::EnemyTurn => {
                **text = "Enemy Turn".to_string();
                *color = TextColor(ENEMY_COLOR);
            }
        }
    }
}
