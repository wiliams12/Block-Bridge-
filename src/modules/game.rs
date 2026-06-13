use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::collections::HashMap;
use std::collections::VecDeque;

use crate::modules::general::*;
use crate::modules::grid::*;
use crate::modules::helpers::*;
use crate::modules::levels::*;

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShapeId(pub u32);

#[derive(Resource, Default)]
pub struct NextBlocks(pub VecDeque<ShapeType>);

#[derive(Resource, Default)]
pub struct Falling(pub bool);

#[derive(Resource, Default)]
pub struct ShapeCounter(pub u32);

#[derive(Resource)]
pub struct ActivePlacement {
    pub shape: ShapeType,
    pub orientation: Orientation,
    pub material: TileType,
    pub current_grid_pos: GridPosition,
}

#[derive(Component)]
pub struct HoverTile;

#[derive(Resource)]
pub struct FallTimer(pub Timer);

impl Default for FallTimer {
    fn default() -> Self {
        // Blocks will drop 1 tile every 0.5 seconds. Adjust this to change speed!
        Self(Timer::from_seconds(0.15, TimerMode::Repeating))
    }
}

impl Default for ActivePlacement {
    fn default() -> Self {
        Self {
            shape: ShapeType::LShape,
            orientation: Orientation::default(),
            material: TileType::PlayerBlock,
            current_grid_pos: GridPosition { x: 0, y: 0 },
        }
    }
}

impl ActivePlacement {
    pub fn get_absolute_tiles(&self) -> Vec<GridPosition> {
        self.shape
            .get_base_offsets()
            .into_iter()
            .map(|(x, y)| {
                let (rot_x, rot_y) = match self.orientation {
                    Orientation::North => (x, y),
                    Orientation::East => (y, -x),
                    Orientation::South => (-x, -y),
                    Orientation::West => (-y, x),
                };
                GridPosition {
                    x: self.current_grid_pos.x + rot_x,
                    y: self.current_grid_pos.y + rot_y,
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Orientation {
    #[default]
    North,
    East,
    South,
    West,
}

impl Orientation {
    pub fn rotate_clockwise(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }
    pub fn rotate_counter_clockwise(self) -> Self {
        match self {
            Self::North => Self::West,
            Self::West => Self::South,
            Self::South => Self::East,
            Self::East => Self::North,
        }
    }
}

pub fn update_placement_state(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    grid_config: Res<GridConfig>,
    mut placement: ResMut<ActivePlacement>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if grid_config.tile_size == 0.0 {
        return;
    }

    let Ok(window) = window_query.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    // Update Grid Position based on mouse
    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            // Translate the world coordinates back into our integer grid system
            let grid_x =
                ((world_pos.x - grid_config.bottom_left.x) / grid_config.tile_size).floor() as i32;
            let grid_y =
                ((world_pos.y - grid_config.bottom_left.y) / grid_config.tile_size).floor() as i32;

            placement.current_grid_pos = GridPosition {
                x: grid_x,
                y: grid_y,
            };
        }
    }

    // Handle Rotation
    if keyboard_input.just_pressed(KeyCode::KeyE) {
        placement.orientation = placement.orientation.rotate_clockwise();
    }
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        placement.orientation = placement.orientation.rotate_counter_clockwise();
    }
}

/// Draws the ghost shape on the mouse, showing RED if invalid
pub fn render_hover_block(
    mut commands: Commands,
    placement: Res<ActivePlacement>,
    grid_config: Res<GridConfig>,
    occupied_grid: Res<OccupiedGrid>,
    asset_server: Res<AssetServer>,
    hover_query: Query<Entity, With<HoverTile>>,
) {
    // 1. Destroy the old ghost tiles from the previous frame
    for entity in &hover_query {
        commands.entity(entity).despawn();
    }

    if grid_config.tile_size == 0.0 {
        return;
    }

    let tiles = placement.get_absolute_tiles();

    let is_valid = tiles.iter().all(|pos| {
        let in_bounds =
            pos.x >= 0 && pos.x < grid_config.columns && pos.y >= 0 && pos.y < grid_config.rows;
        let is_free = !occupied_grid.0.contains(pos);

        in_bounds && is_free
    });

    let mut color = Color::srgba(1.0, 1.0, 1.0, 0.4);
    if !is_valid {
        color = Color::srgba(1.0, 0.2, 0.2, 0.6);
    }

    // Draw the new ghost tiles
    for pos in tiles {
        if pos.x < grid_config.columns {
            // Calculate the exact pixel center immediately
            let start_x = grid_config.bottom_left.x
                + (pos.x as f32 * grid_config.tile_size)
                + (grid_config.tile_size / 2.0);
            let start_y = grid_config.bottom_left.y
                + (pos.y as f32 * grid_config.tile_size)
                + (grid_config.tile_size / 2.0);

            commands.spawn((
                Sprite {
                    image: asset_server.load("textures/player_block.png"),
                    color,
                    custom_size: Some(Vec2::splat(grid_config.tile_size)),
                    ..default()
                },
                // Inject the coordinates instantly with Z = 0.0 (Above background)
                Transform::from_xyz(start_x, start_y, 0.0),
                pos,
                HoverTile,
                LevelEntity,
            ));
        }
    }
}

pub fn place_block(
    mut commands: Commands,
    mut placement: ResMut<ActivePlacement>,
    grid_config: Res<GridConfig>,
    mut occupied_grid: ResMut<OccupiedGrid>,
    asset_server: Res<AssetServer>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut shape_counter: ResMut<ShapeCounter>,
    mut next_blocks: ResMut<NextBlocks>,
    mut score: ResMut<Score>,
    mut falling: ResMut<Falling>,
) {
    if !mouse_input.just_pressed(MouseButton::Left) && !keyboard_input.just_pressed(KeyCode::Space)
    {
        return;
    }

    let tiles = placement.get_absolute_tiles();

    // Strict Bounds Check: Do nothing if placing off-screen
    let is_valid = tiles.iter().all(|pos| {
        let in_bounds =
            pos.x >= 0 && pos.x < grid_config.columns && pos.y >= 0 && pos.y < grid_config.rows;
        let is_free = !occupied_grid.0.contains(pos);

        in_bounds && is_free
    });

    if !is_valid {
        return;
    }

    let texture_path = match placement.material {
        TileType::Rock => "textures/rock.png",
        TileType::Dirt => "textures/dirt.png",
        TileType::Bricks => "textures/bricks.png",
        TileType::Concrete => "textures/concrete.png",
        TileType::PlayerBlock => "textures/player_block.png",
    };

    // Spawn permanent tiles
    for pos in tiles {
        occupied_grid.0.insert(pos);
        // Calculate the exact pixel center immediately
        let start_x = grid_config.bottom_left.x
            + (pos.x as f32 * grid_config.tile_size)
            + (grid_config.tile_size / 2.0);
        let start_y = grid_config.bottom_left.y
            + (pos.y as f32 * grid_config.tile_size)
            + (grid_config.tile_size / 2.0);

        commands.spawn((
            Sprite {
                image: asset_server.load(texture_path),
                custom_size: Some(Vec2::splat(grid_config.tile_size)),
                ..default()
            },
            // Inject the calculated coordinates instead of (0.0, 0.0, -3.0)
            Transform::from_xyz(start_x, start_y, -3.0),
            pos,
            LevelEntity,
            ShapeId(shape_counter.0),
        ));
    }
    shape_counter.0 += 1;
    if let Some(next_shape) = next_blocks.0.pop_front() {
        placement.shape = next_shape;
        placement.orientation = Orientation::North;
    }

    next_blocks.0.push_back(get_random_shape());
    score.0 += 1;
    falling.0 = true;
}

pub fn apply_gravity(
    mut commands: Commands,
    time: Res<Time>,
    mut fall_timer: ResMut<FallTimer>,
    mut occupied_grid: ResMut<OccupiedGrid>,
    mut query: Query<(Entity, &mut GridPosition, &ShapeId)>,
    mut falling: ResMut<Falling>,
) {
    if !fall_timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let mut shapes: HashMap<u32, Vec<GridPosition>> = HashMap::new();

    // 1. Map shapes and cull entities that fell below 0
    for (entity, pos, id) in &query {
        if pos.y < 0 {
            // Delete it from memory and despawn the physical sprite
            occupied_grid.0.remove(&pos);
            commands.entity(entity).despawn();
        } else {
            shapes.entry(id.0).or_default().push(*pos);
        }
    }

    let mut shapes_to_move = Vec::new();

    // 2. Determine which shapes are allowed to fall
    for (id, tiles) in &shapes {
        let can_fall = tiles.iter().all(|pos| {
            let target_pos = GridPosition {
                x: pos.x,
                y: pos.y - 1,
            };

            // RESTORED: Cannot fall if the tile below is occupied...
            // UNLESS the tile occupying it belongs to this exact same shape.
            if occupied_grid.0.contains(&target_pos) && !tiles.contains(&target_pos) {
                return false;
            }

            true
        });

        if can_fall {
            shapes_to_move.push(*id);
        }
    }

    // 3. Move the valid shapes
    if !shapes_to_move.is_empty() {
        // Step A: We MUST erase all old positions from the grid memory first
        for (_, pos, id) in &mut query {
            if shapes_to_move.contains(&id.0) {
                occupied_grid.0.remove(&pos);
            }
        }

        // Step B: Update the actual coordinates and write them back into the grid memory
        for (_, mut pos, id) in &mut query {
            if shapes_to_move.contains(&id.0) {
                pos.y -= 1;
                occupied_grid.0.insert(*pos);
            }
        }
    } else {
        falling.0 = false;
    }
}

#[derive(Resource, Default)]
pub struct Score(pub u32);

pub fn score_level(num_of_shapes: u32, current_level: u32) -> u32 {
    (100 / num_of_shapes).pow(current_level)
}

pub fn game_plugin(app: &mut App) {
    app.init_resource::<FallTimer>();
    app.init_resource::<Falling>();
    app.init_resource::<Score>();
    app.init_state::<AppState>();
    app.init_resource::<ActivePlacement>();
    app.init_resource::<ShapeCounter>();
    app.init_resource::<NextBlocks>();
    app.add_systems(
        Update,
        (
            update_placement_state,
            render_hover_block,
            place_block,
            apply_gravity,
        )
            .run_if(in_state(AppState::InGame)),
    );
}
