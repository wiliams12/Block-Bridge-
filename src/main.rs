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
        .add_plugins((
            levels_plugin,
            game_plugin,
            general_plugin,
            grid_plugin,
            main_menu_plugin,
            popup_plugin,
            ui_overlay_plugin,
        ))
        .run();
}
