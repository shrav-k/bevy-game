// System definitions - where all the game logic lives
// Systems are just functions that operate on components and resources

use bevy::prelude::*;

use crate::components::{GridPosition, Tile};
use crate::constants::*;
use crate::resources::GridMap;
use crate::AppState;

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
                Text::new("Phase 2: State Management"),
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
