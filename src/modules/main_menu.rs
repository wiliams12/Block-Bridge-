use crate::modules::general::*;
use crate::modules::grid::snap_to_grid;
use bevy::prelude::*;

#[derive(Component)]
pub struct MenuUI;

#[derive(Component)]
pub enum MenuButtonAction {
    Play,
    Quit,
}

pub fn main_menu(app: &mut App) {
    app.add_systems(OnEnter(AppState::Menu), setup_main_menu)
        .add_systems(Update, main_menu_actions.run_if(in_state(AppState::Menu)))
        .add_systems(Update, snap_to_grid.run_if(in_state(AppState::InGame)))
        .add_systems(OnExit(AppState::Menu), cleanup_menu);
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

pub fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn main_menu_plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Menu), setup_main_menu);
    app.add_systems(Update, main_menu_actions.run_if(in_state(AppState::Menu)));
    app.add_systems(OnExit(AppState::Menu), cleanup_menu);
}
