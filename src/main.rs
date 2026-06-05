use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::window::WindowMode;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};
use std::fs;

// --- STATES & RESOURCES ---
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Menu,
    LoadingLevel,
    InGame,
    PopUpMenu,
}

#[derive(Resource, Default)]
pub struct GridConfig {
    pub columns: i32,
    pub rows: i32,
    pub tile_size: f32,
    pub bottom_left: Vec2, // The exact pixel coordinate where the grid starts
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Resource, Default)]
pub struct OccupiedGrid(pub HashSet<GridPosition>);
// ! TODO: Create a serach algorithm for the winning condition

#[derive(Resource, Default)]
pub struct CurrentLevel(pub u32);

#[derive(Resource, Default)]
pub struct NextBlocks(pub VecDeque<ShapeType>);

// --- COMPONENTS ---
#[derive(Component)]
pub struct LevelEntity;

#[derive(Component)]
pub enum MenuButtonAction {
    Play,
    Quit,
}

#[derive(Component)]
pub enum PopUpAction {
    Resume,
    Restart,
    QuitToMenu,
    QuitToDesktop,
}

#[derive(Component)]
pub struct MenuUI;

#[derive(Component)]
pub struct PopUpMenuUI;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileType {
    Dirt,
    Rock,
    Bricks,
    Concrete,
    PlayerBlock,
}

#[derive(Component)]
pub struct NextBlocksPanel;

#[derive(Component)]
pub struct NextBlockPreview;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeType {
    LShape,
    JShape,
    OShape,
    FourLine,
    TShape,
    SShape,
    ZShape,
}

impl ShapeType {
    pub fn get_base_offsets(&self) -> Vec<(i32, i32)> {
        match self {
            ShapeType::LShape => vec![(0, 0), (0, 1), (0, 2), (1, 0)],
            ShapeType::JShape => vec![(0, 0), (1, 0), (-1, 0), (-2, 0)],
            ShapeType::OShape => vec![(0, 0), (0, 1), (1, 0), (1, 1)],
            ShapeType::FourLine => vec![(-1, 0), (0, 0), (0, 1), (0, 2)],
            ShapeType::TShape => vec![(-1, 0), (0, 0), (1, 0), (0, 1)],
            ShapeType::SShape => vec![(0, 0), (0, 1), (1, 1), (1, 2)],
            ShapeType::ZShape => vec![(1, 0), (1, 1), (0, 1), (0, 2)],
        }
    }
}

#[derive(Resource)]
pub struct ActivePlacement {
    pub shape: ShapeType,
    pub orientation: Orientation,
    pub material: TileType,
    pub current_grid_pos: GridPosition,
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

// Tag for the ghost tiles that follow the mouse
#[derive(Component)]
pub struct HoverTile;

#[derive(Component)]
pub struct MainCamera;

#[derive(Serialize, Deserialize, Debug)]
pub struct TileData {
    pub position: GridPosition,
    pub tile_type: TileType,
}

// 3. Update LevelData to hold an array of these tiles
#[derive(Serialize, Deserialize, Debug)]
pub struct LevelData {
    pub columns: i32,
    pub rows: i32,
    pub tiles: Vec<TileData>,
    pub background_img: String,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShapeId(pub u32);

#[derive(Resource, Default)]
pub struct ShapeCounter(pub u32);

#[derive(Resource)]
pub struct FallTimer(pub Timer);

impl Default for FallTimer {
    fn default() -> Self {
        // Blocks will drop 1 tile every 0.5 seconds. Adjust this to change speed!
        Self(Timer::from_seconds(0.15, TimerMode::Repeating))
    }
}

// --- MAIN ---
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Block Bridge!".into(),
                mode: WindowMode::BorderlessFullscreen(MonitorSelection::Index(0)),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .init_resource::<GridConfig>()
        .insert_resource(CurrentLevel(1))
        .init_resource::<ActivePlacement>()
        .init_resource::<OccupiedGrid>()
        .init_resource::<ShapeCounter>()
        .init_resource::<FallTimer>()
        .init_resource::<NextBlocks>()
        // Startup
        .add_systems(Startup, spawn_camera)
        // Universal UI Hover (Runs when either menu is active)
        .add_systems(
            Update,
            generic_button_hover.run_if(in_state(AppState::Menu).or(in_state(AppState::PopUpMenu))),
        )
        // Main Menu
        .add_systems(OnEnter(AppState::Menu), setup_main_menu)
        .add_systems(Update, main_menu_actions.run_if(in_state(AppState::Menu)))
        .add_systems(Update, snap_to_grid.run_if(in_state(AppState::InGame)))
        .add_systems(OnExit(AppState::Menu), cleanup_menu)
        // PopUp Menu
        .add_systems(OnEnter(AppState::PopUpMenu), setup_popup_menu)
        .add_systems(Update, popup_actions.run_if(in_state(AppState::PopUpMenu)))
        .add_systems(OnExit(AppState::PopUpMenu), cleanup_popup)
        // Level Management
        .add_systems(OnEnter(AppState::Menu), cleanup_level)
        .add_systems(
            OnEnter(AppState::LoadingLevel),
            (cleanup_level, spawn_level).chain(),
        )
        // Gameplay
        // The toggle button needs to run in BOTH the game and the menu to flip back and forth
        .add_systems(
            Update,
            toggle_popup_menu.run_if(in_state(AppState::InGame).or(in_state(AppState::PopUpMenu))),
        )
        .add_systems(
            Update,
            (
                snap_to_grid,           // Moves sprites to match grid coordinates
                check_level_completion, // Checks win condition
                update_placement_state, // Tracks mouse and rotation
                render_hover_block,     // Draws the ghost block
                place_block,            // Clicks to spawn permanent blocks
                apply_gravity,          // <-- ADD THIS so the blocks actually fall!
                update_next_blocks_ui,
            )
                .run_if(in_state(AppState::InGame)),
        )
        .run();
}

// --- SYSTEM DEFINITIONS ---

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

// --- GAMEPLAY SYSTEMS ---

/// Automatically translates integer grid coordinates into pixel screen coordinates
pub fn snap_to_grid(
    grid_config: Res<GridConfig>,
    mut query: Query<(&GridPosition, &mut Transform)>,
) {
    if grid_config.tile_size == 0.0 {
        return;
    }

    for (grid_pos, mut transform) in &mut query {
        transform.translation.x = grid_config.bottom_left.x
            + (grid_pos.x as f32 * grid_config.tile_size)
            + (grid_config.tile_size / 2.0);

        transform.translation.y = grid_config.bottom_left.y
            + (grid_pos.y as f32 * grid_config.tile_size)
            + (grid_config.tile_size / 2.0);
    }
}

fn spawn_level(
    mut commands: Commands,
    mut level_res: ResMut<CurrentLevel>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<AppState>>,
    mut occupied_grid: ResMut<OccupiedGrid>,
    mut next_blocks: ResMut<NextBlocks>,
    mut placement: ResMut<ActivePlacement>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };

    occupied_grid.0.clear();

    if level_res.0 > 3 {
        level_res.0 = 1;
        next_state.set(AppState::Menu); // ! maybe add an end screen
        return;
    }

    let file_path = format!("assets/levels/level_{}.json", level_res.0);
    let level_data: LevelData = match fs::read_to_string(&file_path) {
        Ok(json_str) => serde_json::from_str(&json_str).expect("Invalid JSON format"),
        Err(_) => return,
    };

    if !level_data.background_img.is_empty() {
        commands.spawn((
            Sprite {
                image: asset_server.load(format!("backgrounds/{}", level_data.background_img)),
                custom_size: Some(Vec2::new(window.width(), window.height())),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, -10.0),
            LevelEntity,
        ));
    }

    let ui_panel_width = 350.0;
    let available_width = window.width() - ui_panel_width;

    let tile_size =
        (available_width / level_data.columns as f32).min(window.height() / level_data.rows as f32);

    let grid_width = level_data.columns as f32 * tile_size;
    let grid_height = level_data.rows as f32 * tile_size;

    let bottom_left = Vec2::new(
        (-grid_width / 2.0) - (ui_panel_width / 2.0),
        -grid_height / 2.0,
    );

    commands.insert_resource(GridConfig {
        columns: level_data.columns,
        rows: level_data.rows,
        tile_size,
        bottom_left,
    });

    let line_color = Color::srgba(1.0, 1.0, 1.0, 0.15);
    let line_thickness = 1.0;

    // Vertical Lines
    for x in 0..=level_data.columns {
        let x_pos = bottom_left.x + (x as f32 * tile_size);
        commands.spawn((
            Sprite {
                color: line_color,
                custom_size: Some(Vec2::new(line_thickness, grid_height)),
                ..default()
            },
            Transform::from_xyz(x_pos, 0.0, -4.0),
            LevelEntity,
        ));
    }

    // We need the true center of the shifted grid to place the horizontal lines perfectly
    let grid_center_x = bottom_left.x + (grid_width / 2.0);

    // Horizontal Lines
    for y in 0..=level_data.rows {
        let y_pos = bottom_left.y + (y as f32 * tile_size);
        commands.spawn((
            Sprite {
                color: line_color,
                custom_size: Some(Vec2::new(grid_width, line_thickness)),
                ..default()
            },
            // Use the calculated grid_center_x instead of 0.0
            Transform::from_xyz(grid_center_x, y_pos, -6.0),
            LevelEntity,
        ));
    }

    for tile in level_data.tiles.iter() {
        let texture_path = match tile.tile_type {
            TileType::Rock => "textures/rock.png",
            TileType::Dirt => "textures/dirt.png",
            TileType::Bricks => "textures/bricks.png",
            TileType::Concrete => "textures/concrete.png",
            TileType::PlayerBlock => "textures/player_block.png",
        };

        let start_x = bottom_left.x + (tile.position.x as f32 * tile_size) + (tile_size / 2.0);
        let start_y = bottom_left.y + (tile.position.y as f32 * tile_size) + (tile_size / 2.0);

        commands.spawn((
            Sprite {
                image: asset_server.load(texture_path),
                custom_size: Some(Vec2::splat(tile_size)),
                ..default()
            },
            Transform::from_xyz(start_x, start_y, -5.0),
            tile.position,
            LevelEntity,
        ));

        occupied_grid.0.insert(tile.position);
    }

    commands.spawn((
        Node {
            width: Val::Px(ui_panel_width),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            right: Val::Px(0.0),
            top: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center, // Center the header text
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.08, 0.08, 0.95)),
        LevelEntity,
        NextBlocksPanel, // <-- Tag to locate the panel
        children![(
            // Header Container
            Node {
                margin: UiRect::top(Val::Px(40.0)),
                ..default()
            },
            children![(
                Text::new("NEXT BLOCKS"),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            )]
        )],
    ));

    next_blocks.0.clear();

    placement.shape = get_random_shape();

    for _ in 0..3 {
        next_blocks.0.push_back(get_random_shape());
    }

    next_state.set(AppState::InGame);
}

fn check_level_completion(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut level: ResMut<CurrentLevel>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // Simulating a win condition (e.g., reaching a door) by pressing Space
    if keyboard_input.just_pressed(KeyCode::Space) {
        level.0 += 1;
        next_state.set(AppState::LoadingLevel);
    }
}

pub fn update_next_blocks_ui(
    mut commands: Commands,
    next_blocks: Res<NextBlocks>,
    asset_server: Res<AssetServer>,
    panel_query: Query<Entity, With<NextBlocksPanel>>,
    preview_query: Query<Entity, With<NextBlockPreview>>,
) {
    if !next_blocks.is_changed() {
        return;
    }

    // 1. Delete all visual blocks from the previous frame
    for entity in &preview_query {
        commands.entity(entity).despawn();
    }

    let Ok(panel_entity) = panel_query.single() else {
        return;
    };

    let block_size = 30.0;

    // 2. Build the new shapes based on the queue
    for (i, shape) in next_blocks.0.iter().enumerate() {
        let offsets = shape.get_base_offsets();

        // Spawn an invisible 150x150 container for this specific shape
        let shape_container = commands
            .spawn((
                Node {
                    width: Val::Px(150.0),
                    height: Val::Px(150.0),
                    position_type: PositionType::Absolute,
                    top: Val::Px(120.0 + (i as f32 * 170.0)), // Space them vertically
                    // Center inside the 350px panel: (350 - 150) / 2 = 100
                    left: Val::Px(100.0),
                    ..default()
                },
                NextBlockPreview, // Tag the container so it can be deleted later
            ))
            .id();

        // Attach the container to the main dark panel
        commands.entity(panel_entity).add_child(shape_container);

        // Spawn the individual textured blocks inside the container
        for (x, y) in offsets {
            let tile = commands
                .spawn((
                    Node {
                        width: Val::Px(block_size),
                        height: Val::Px(block_size),
                        position_type: PositionType::Absolute,
                        // Center the blocks inside the 150x150 container mathematically
                        bottom: Val::Px((y + 1) as f32 * block_size),
                        left: Val::Px((x + 2) as f32 * block_size),
                        ..default()
                    },
                    // Use your exact game texture!
                    ImageNode::new(asset_server.load("textures/player_block.png")),
                ))
                .id();

            // Attach the tile to the shape container
            commands.entity(shape_container).add_child(tile);
        }
    }
}

/// 1. Tracks the mouse and handles Q/E rotation
fn update_placement_state(
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

/// 3. Commits the block to the level on Click or Spacebar
/// Commits the block to the level on Click or Spacebar
fn place_block(
    mut commands: Commands,
    mut placement: ResMut<ActivePlacement>,
    grid_config: Res<GridConfig>,
    mut occupied_grid: ResMut<OccupiedGrid>,
    asset_server: Res<AssetServer>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut shape_counter: ResMut<ShapeCounter>,
    mut next_blocks: ResMut<NextBlocks>,
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
}

pub fn apply_gravity(
    time: Res<Time>,
    mut fall_timer: ResMut<FallTimer>,
    mut occupied_grid: ResMut<OccupiedGrid>,
    // Only query entities that have a ShapeId (This automatically ignores your static level blocks!)
    mut query: Query<(&mut GridPosition, &ShapeId)>,
) {
    if !fall_timer.0.tick(time.delta()).just_finished() {
        return;
    }

    // 1. Map out all existing shapes
    let mut shapes: HashMap<u32, Vec<GridPosition>> = HashMap::new();
    for (pos, id) in &query {
        shapes.entry(id.0).or_default().push(*pos);
    }

    let mut shapes_to_move = Vec::new();

    // 2. Determine which shapes are allowed to fall
    for (id, tiles) in &shapes {
        let can_fall = tiles.iter().all(|pos| {
            let target_pos = GridPosition {
                x: pos.x,
                y: pos.y - 1,
            };

            // Rule B: Cannot fall if the tile below is occupied...
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
        for (mut pos, id) in &mut query {
            if shapes_to_move.contains(&id.0) {
                occupied_grid.0.remove(&pos);
            }
        }

        // Step B: Update the actual coordinates and write them back into the grid memory
        for (mut pos, id) in &mut query {
            if shapes_to_move.contains(&id.0) {
                pos.y -= 1;
                occupied_grid.0.insert(*pos);
            }
        }
    }
}

fn toggle_popup_menu(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        match state.get() {
            AppState::InGame => next_state.set(AppState::PopUpMenu),
            AppState::PopUpMenu => next_state.set(AppState::InGame),
            _ => {}
        }
    }
}

fn cleanup_level(mut commands: Commands, query: Query<Entity, With<LevelEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn menu_button(text: &str, action: impl Component) -> impl Bundle {
    (
        Button,
        Node {
            width: Val::Px(350.0),
            min_height: Val::Px(80.0),

            padding: UiRect::all(Val::Px(15.0)),

            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
        action,
        children![(
            Text::new(text.to_string()),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            TextColor(Color::WHITE),
            // 3. Forces the text to center-align if it spans multiple lines
            TextLayout {
                justify: Justify::Center,
                ..default()
            },
        )],
    )
}

pub fn setup_main_menu(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(20.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
        MenuUI,
        children![
            menu_button("Start Game", MenuButtonAction::Play),
            menu_button("Quit", MenuButtonAction::Quit),
        ],
    ));
}

pub fn setup_popup_menu(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(20.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        PopUpMenuUI,
        children![
            menu_button("Resume", PopUpAction::Resume),
            menu_button("Restart", PopUpAction::Restart),
            menu_button("Quit to Main Menu", PopUpAction::QuitToMenu),
            menu_button("Quit to Desktop", PopUpAction::QuitToDesktop),
        ],
    ));
}

pub fn generic_button_hover(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => *color = BackgroundColor(Color::srgb(0.35, 0.35, 0.35)),
            Interaction::Hovered => *color = BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
            Interaction::None => *color = BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
        }
    }
}

pub fn main_menu_actions(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit_writer: MessageWriter<AppExit>,
) {
    for (interaction, action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match action {
                MenuButtonAction::Play => {
                    next_state.set(AppState::LoadingLevel);
                }
                MenuButtonAction::Quit => {
                    exit_writer.write(AppExit::Success);
                }
            }
        }
    }
}

pub fn popup_actions(
    interaction_query: Query<(&Interaction, &PopUpAction), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit_writer: MessageWriter<AppExit>,
) {
    for (interaction, action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match action {
                PopUpAction::Resume => next_state.set(AppState::InGame),
                PopUpAction::Restart => next_state.set(AppState::LoadingLevel),
                PopUpAction::QuitToMenu => next_state.set(AppState::Menu),
                PopUpAction::QuitToDesktop => {
                    exit_writer.write(AppExit::Success);
                }
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn cleanup_popup(mut commands: Commands, query: Query<Entity, With<PopUpMenuUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn get_random_shape() -> ShapeType {
    let mut rng = rand::thread_rng();
    match rng.gen_range(0..7) {
        0 => ShapeType::LShape,
        1 => ShapeType::JShape,
        2 => ShapeType::OShape,
        3 => ShapeType::FourLine,
        4 => ShapeType::TShape,
        5 => ShapeType::SShape,
        _ => ShapeType::ZShape,
    }
}
