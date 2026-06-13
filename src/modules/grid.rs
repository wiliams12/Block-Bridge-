use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::modules::general::*;

#[derive(Resource, Default)]
pub struct OccupiedGrid(pub HashSet<GridPosition>);

#[derive(Resource, Default)]
pub struct GridConfig {
    pub columns: i32,
    pub rows: i32,
    pub tile_size: f32,
    pub bottom_left: Vec2, // the offset from the center to the bottom left
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

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

pub fn grid_plugin(app: &mut App) {
    app.init_resource::<GridConfig>();
    app.init_resource::<OccupiedGrid>();
    app.add_systems(Update, (snap_to_grid,).run_if(in_state(AppState::InGame)));
}
