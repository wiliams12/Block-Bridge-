use bevy::prelude::*;

use crate::modules::general::*;

#[derive(Component)]
pub struct PopUpMenuUI;

#[derive(Component)]
pub enum PopUpAction {
    Resume,
    Restart,
    QuitToMenu,
    QuitToDesktop,
}

pub fn toggle_popup_menu(
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

pub fn cleanup_popup(mut commands: Commands, query: Query<Entity, With<PopUpMenuUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn popup_plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::PopUpMenu), setup_popup_menu);
    app.add_systems(Update, popup_actions.run_if(in_state(AppState::PopUpMenu)));
    app.add_systems(OnExit(AppState::PopUpMenu), cleanup_popup);
    app.add_systems(
        Update,
        toggle_popup_menu.run_if(in_state(AppState::InGame).or(in_state(AppState::PopUpMenu))),
    );
}
