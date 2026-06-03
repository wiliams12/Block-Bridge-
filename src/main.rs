use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::window::WindowMode;

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
            (check_level_completion, toggle_popup_menu).run_if(in_state(AppState::InGame)),
        )
        .run();
}

// --- SYSTEM DEFINITIONS ---

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// --- GAMEPLAY SYSTEMS ---

fn spawn_level(
    mut commands: Commands,
    level: Res<CurrentLevel>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let background_color = match level.0 {
        1 => Color::srgb(0.2, 0.6, 0.2), // Grass green
        2 => Color::srgb(0.2, 0.2, 0.6), // Ocean blue
        3 => Color::srgb(0.6, 0.2, 0.2), // Lava red
        _ => Color::srgb(0.1, 0.1, 0.1), // Default dark grey
    };

    commands.spawn((
        Sprite {
            color: background_color,
            custom_size: Some(Vec2::new(5000.0, 5000.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -100.0),
        LevelEntity,
    ));

    println!("Level {} successfully loaded!", level.0);
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
