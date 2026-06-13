use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::modules::game::*;
use crate::modules::general::*;

// --- RESOURCES & COMPONENTS ---

#[derive(Component)]
pub struct GameOverScreen;

#[derive(Component)]
pub struct InitialsText;

#[derive(Resource, Default)]
pub struct CurrentInitials(pub String);

#[derive(Serialize, Deserialize, Clone)]
pub struct LeaderboardEntry {
    pub initials: String,
    pub score: u32,
}

const LEADERBOARD_FILE: &str = "leaderboard.json";

// --- HELPERS ---

fn load_leaderboard() -> Vec<LeaderboardEntry> {
    if let Ok(data) = fs::read_to_string(LEADERBOARD_FILE) {
        if let Ok(entries) = serde_json::from_str(&data) {
            return entries;
        }
    }
    Vec::new() // Return empty if file doesn't exist yet
}

fn save_leaderboard(entries: &[LeaderboardEntry]) {
    if let Ok(data) = serde_json::to_string_pretty(entries) {
        let _ = fs::write(LEADERBOARD_FILE, data);
    }
}

// --- SYSTEMS ---

pub fn spawn_end_screen(
    mut commands: Commands,
    score: Res<Score>, // Assuming this is crate::modules::general::Score
) {
    let leaderboard = load_leaderboard();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.08, 0.08, 0.98)),
            ZIndex(100),
            GameOverScreen,
        ))
        .with_children(|parent| {
            // 1. GAME OVER TITLE
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 0.2, 0.2, 1.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // 2. CURRENT SCORE
            parent.spawn((
                Text::new(format!("FINAL SCORE: {}", score.0)),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                },
            ));

            // 3. INITIALS PROMPT
            parent.spawn((
                Text::new("ENTER INITIALS: _ _ _"),
                TextFont {
                    font_size: 50.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 0.0, 1.0)),
                InitialsText,
                Node {
                    margin: UiRect::bottom(Val::Px(60.0)),
                    ..default()
                },
            ));

            // 4. LEADERBOARD HEADER
            parent.spawn((
                Text::new("LEADERBOARD"),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // 5. DYNAMIC LEADERBOARD ENTRIES
            for (i, entry) in leaderboard.iter().take(5).enumerate() {
                parent.spawn((
                    Text::new(format!("{}. {} - {}", i + 1, entry.initials, entry.score)),
                    TextFont {
                        font_size: 25.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
                    Node {
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },
                ));
            }
        });
}

pub fn game_end(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut initials: ResMut<CurrentInitials>,
    mut score: ResMut<Score>,
    mut next_state: ResMut<NextState<AppState>>,
    mut text_query: Query<&mut Text, With<InitialsText>>,
) {
    let mut updated = false;

    // Listen for inputs to build the 3-character initials
    for key in keyboard_input.get_just_pressed() {
        if initials.0.len() < 3 {
            let char_to_add = match key {
                KeyCode::KeyA => "A",
                KeyCode::KeyB => "B",
                KeyCode::KeyC => "C",
                KeyCode::KeyD => "D",
                KeyCode::KeyE => "E",
                KeyCode::KeyF => "F",
                KeyCode::KeyG => "G",
                KeyCode::KeyH => "H",
                KeyCode::KeyI => "I",
                KeyCode::KeyJ => "J",
                KeyCode::KeyK => "K",
                KeyCode::KeyL => "L",
                KeyCode::KeyM => "M",
                KeyCode::KeyN => "N",
                KeyCode::KeyO => "O",
                KeyCode::KeyP => "P",
                KeyCode::KeyQ => "Q",
                KeyCode::KeyR => "R",
                KeyCode::KeyS => "S",
                KeyCode::KeyT => "T",
                KeyCode::KeyU => "U",
                KeyCode::KeyV => "V",
                KeyCode::KeyW => "W",
                KeyCode::KeyX => "X",
                KeyCode::KeyY => "Y",
                KeyCode::KeyZ => "Z",
                _ => "",
            };

            if !char_to_add.is_empty() {
                initials.0.push_str(char_to_add);
                updated = true;
            }
        }

        if *key == KeyCode::Backspace && !initials.0.is_empty() {
            initials.0.pop();
            updated = true;
        }

        // Submit Logic
        if *key == KeyCode::Enter && initials.0.len() == 3 {
            let mut leaderboard = load_leaderboard();

            leaderboard.push(LeaderboardEntry {
                initials: initials.0.clone(),
                score: score.0,
            });

            // Sort highest to lowest
            leaderboard.sort_by(|a, b| b.score.cmp(&a.score));

            // Optional: Keep only top 10 to prevent infinite file size
            leaderboard.truncate(10);

            save_leaderboard(&leaderboard);

            // Reset game data and transition out
            score.0 = 0;
            next_state.set(AppState::Menu);
            return;
        }
    }

    // Visually update the text component with underscores for missing letters
    if updated {
        if let Ok(mut text) = text_query.single_mut() {
            let chars: Vec<char> = initials.0.chars().collect();
            let c1 = chars.get(0).unwrap_or(&'_');
            let c2 = chars.get(1).unwrap_or(&'_');
            let c3 = chars.get(2).unwrap_or(&'_');

            text.0 = format!("ENTER INITIALS: {} {} {}", c1, c2, c3);
        }
    }
}

pub fn despawn_end_screen(
    mut commands: Commands,
    query: Query<Entity, With<GameOverScreen>>,
    mut initials: ResMut<CurrentInitials>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    // Wipe initials from memory so they are fresh for the next run
    initials.0.clear();
}

pub fn game_end_plugin(app: &mut App) {
    app.init_resource::<CurrentInitials>()
        .add_systems(OnEnter(AppState::GameEnd), spawn_end_screen)
        .add_systems(Update, game_end.run_if(in_state(AppState::GameEnd)))
        .add_systems(OnExit(AppState::GameEnd), despawn_end_screen);
}
