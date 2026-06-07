use bevy::prelude::*;
use bevy::window::WindowMode;

use blockbridge::modules::*;

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
        // Main Menu (spawn UI + button handlers)
        .add_systems(OnEnter(AppState::Menu), setup_main_menu)
        .add_systems(Update, main_menu_actions.run_if(in_state(AppState::Menu)))
        .add_systems(OnExit(AppState::Menu), cleanup_menu)
        // Universal UI Hover (Runs when either menu is active)
        .add_systems(
            Update,
            generic_button_hover.run_if(in_state(AppState::Menu).or(in_state(AppState::PopUpMenu))),
        )
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
