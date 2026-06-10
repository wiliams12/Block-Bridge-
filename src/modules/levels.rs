use crate::modules::game::*;
use crate::modules::general::*;
use crate::modules::grid::*;
use crate::modules::helpers::*;
use crate::modules::ui_overlay::*;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use serde::{Deserialize, Serialize};

use std::fs;

#[derive(Resource, Default)]
pub struct CurrentLevel(pub u32);

#[derive(Component)]
pub struct LevelEntity;

#[derive(Serialize, Deserialize, Debug)]
pub struct TileData {
    pub position: GridPosition,
    pub tile_type: TileType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LevelData {
    pub columns: i32,
    pub rows: i32,
    pub tiles: Vec<TileData>,
    pub background_img: String,
}

pub fn spawn_level(
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

pub fn check_level_completion(
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

pub fn cleanup_level(mut commands: Commands, query: Query<Entity, With<LevelEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn levels_plugin(app: &mut App) {
    app.insert_resource(CurrentLevel(1));
    app.add_systems(OnEnter(AppState::Menu), cleanup_level);
    app.add_systems(
        OnEnter(AppState::LoadingLevel),
        (cleanup_level, spawn_level).chain(),
    );
    app.add_systems(
        Update,
        check_level_completion.run_if(in_state(AppState::InGame)),
    );
}
