use bevy::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Menu,
    LoadingLevel,
    InGame,
    PopUpMenu,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileType {
    Dirt,
    Rock,
    Bricks,
    Concrete,
    PlayerBlock,
}

#[derive(Component)]
pub struct MainCamera;

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
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
