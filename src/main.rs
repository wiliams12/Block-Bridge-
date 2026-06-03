use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::window::WindowMode;

use serde::{Deserialize, Serialize};
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

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Resource, Default)]
pub struct CurrentLevel(pub u32);

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
}

pub enum BlockType {
    // TODO: Add different types
}

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
        .add_systems(
            Update,
            check_level_completion.run_if(in_state(AppState::InGame)),
        )
        // The toggle button needs to run in BOTH the game and the menu to flip back and forth
        .add_systems(
            Update,
            toggle_popup_menu.run_if(in_state(AppState::InGame).or(in_state(AppState::PopUpMenu))),
        )
        .run();
}

// --- SYSTEM DEFINITIONS ---

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
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
) {
    let Ok(window) = window_query.single() else {
        return;
    };

    if level_res.0 > 3 {
        // ! Maybe change into an END SCREEN
        level_res.0 = 1;
        next_state.set(AppState::Menu);
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

    let tile_size = ((window.width()) / level_data.columns as f32)
        .min((window.height()) / level_data.rows as f32);

    let grid_width = level_data.columns as f32 * tile_size;
    let grid_height = level_data.rows as f32 * tile_size;

    let bottom_left = Vec2::new(-grid_width / 2.0, -grid_height / 2.0);

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

    // Horizontal Lines
    for y in 0..=level_data.rows {
        let y_pos = bottom_left.y + (y as f32 * tile_size);
        commands.spawn((
            Sprite {
                color: line_color,
                custom_size: Some(Vec2::new(grid_width, line_thickness)),
                ..default()
            },
            Transform::from_xyz(0.0, y_pos, -6.0),
            LevelEntity,
        ));
    }

    for tile in level_data.tiles {
        let texture_path = match tile.tile_type {
            TileType::Rock => "textures/rock.png",
            TileType::Dirt => "textures/dirt.png",
            TileType::Bricks => "textures/bricks.png",
            TileType::Concrete => "textures/concrete.png",
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
