use bevy::prelude::*;
use serde::Deserialize;
use std::fs;

use crate::modules::general::*;

#[derive(Component)]
pub struct MenuUI;

#[derive(Component)]
pub enum MenuButtonAction {
    Play,
    Quit,
}

// Ensure this matches the struct you used to save the data in the end screen!
#[derive(Deserialize)]
pub struct LeaderboardEntry {
    pub initials: String,
    pub score: u32,
}

const LEADERBOARD_FILE: &str = "leaderboard.json";

pub fn setup_main_menu(mut commands: Commands) {
    // 1. Load the leaderboard data
    let mut leaderboard: Vec<LeaderboardEntry> = Vec::new();
    if let Ok(data) = fs::read_to_string(LEADERBOARD_FILE) {
        if let Ok(entries) = serde_json::from_str(&data) {
            leaderboard = entries;
        }
    }

    // 2. Spawn the Main UI
    commands
        .spawn((
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
        ))
        .with_children(|parent| {
            // TITLE
            parent.spawn((
                Text::new("MAIN MENU"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // BUTTONS
            // Since you previously used children![menu_button(...)], we can
            // directly spawn whatever bundle that helper function returns!
            parent.spawn(menu_button("Start Game", MenuButtonAction::Play));
            parent.spawn(menu_button("Quit", MenuButtonAction::Quit));

            // LEADERBOARD HEADER
            parent.spawn((
                Text::new("TOP SCORES"),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.0)),
                Node {
                    margin: UiRect::top(Val::Px(40.0)),
                    ..default()
                },
            ));

            // DYNAMIC LEADERBOARD ENTRIES
            if leaderboard.is_empty() {
                parent.spawn((
                    Text::new("No scores yet!"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                ));
            } else {
                for (i, entry) in leaderboard.iter().take(5).enumerate() {
                    parent.spawn((
                        Text::new(format!("{}. {} - {}", i + 1, entry.initials, entry.score)),
                        TextFont {
                            font_size: 25.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(10.0)),
                            ..default()
                        },
                    ));
                }
            }
        });
}

pub fn main_menu_actions(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<AppState>>,
    // 1. Revert this back to what you originally had:
    mut exit_writer: MessageWriter<AppExit>,
) {
    for (interaction, action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match action {
                MenuButtonAction::Play => {
                    next_state.set(AppState::LoadingLevel);
                }
                MenuButtonAction::Quit => {
                    // 2. Revert back to .write() if .send() throws an error
                    exit_writer.write(AppExit::Success);
                }
            }
        }
    }
}

pub fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuUI>>) {
    for entity in &query {
        commands.entity(entity).despawn(); // Changed to despawn_recursive to destroy the children properly
    }
}

// Consolidated Plugin Registration
pub fn main_menu_plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Menu), setup_main_menu)
        .add_systems(Update, main_menu_actions.run_if(in_state(AppState::Menu)))
        .add_systems(OnExit(AppState::Menu), cleanup_menu);
}
