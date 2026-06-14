use crate::modules::game::*;
use bevy::prelude::*;

use crate::modules::general::*;

#[derive(Component)]
pub struct NextBlockPreview;

#[derive(Component)]
pub struct NextBlocksPanel;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct LevelCompleteOverlay;

pub fn update_next_blocks_ui(
    mut commands: Commands,
    next_blocks: Res<NextBlocks>,
    asset_server: Res<AssetServer>,
    panel_query: Query<Entity, With<NextBlocksPanel>>,
    preview_query: Query<Entity, With<NextBlockPreview>>,
) {
    if !next_blocks.is_changed() {
        return;
    }

    // 1. Delete all visual blocks from the previous frame
    for entity in &preview_query {
        commands.entity(entity).despawn();
    }

    let Ok(panel_entity) = panel_query.single() else {
        return;
    };

    let block_size = 30.0;

    // 2. Build the new shapes based on the queue
    for (i, shape) in next_blocks.0.iter().enumerate() {
        let offsets = shape.get_base_offsets();

        // Spawn an invisible 150x150 container for this specific shape
        let shape_container = commands
            .spawn((
                Node {
                    width: Val::Px(150.0),
                    height: Val::Px(150.0),
                    position_type: PositionType::Absolute,
                    top: Val::Px(180.0 + (i as f32 * 170.0)), // Space them vertically
                    // Center inside the 350px panel: (350 - 150) / 2 = 100
                    left: Val::Px(100.0),
                    ..default()
                },
                NextBlockPreview, // Tag the container so it can be deleted later
            ))
            .id();

        // Attach the container to the main dark panel
        commands.entity(panel_entity).add_child(shape_container);

        // Spawn the individual textured blocks inside the container
        for (x, y) in offsets {
            let tile = commands
                .spawn((
                    Node {
                        width: Val::Px(block_size),
                        height: Val::Px(block_size),
                        position_type: PositionType::Absolute,
                        // Center the blocks inside the 150x150 container mathematically
                        bottom: Val::Px((y + 1) as f32 * block_size),
                        left: Val::Px((x + 2) as f32 * block_size),
                        ..default()
                    },
                    // Use your exact game texture!
                    ImageNode::new(asset_server.load("textures/player_block0.png")),
                ))
                .id();

            // Attach the tile to the shape container
            commands.entity(shape_container).add_child(tile);
        }
    }
}

pub fn update_score_ui(score: Res<Score>, mut query: Query<&mut Text, With<ScoreText>>) {
    // Performance optimization: Only reconstruct the string if the score changed
    if !score.is_changed() {
        return;
    }

    if let Ok(mut text) = query.single_mut() {
        // Assuming your Score resource looks like: pub struct Score(pub u32);
        text.0 = format!("SCORE: {}", score.0);
    }
}

pub fn spawn_level_complete_overlay(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center, // Centers children horizontally
                align_items: AlignItems::Center,         // Centers children vertically
                ..default()
            },
            // 0.85 alpha makes it 85% black, 15% transparent
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
            ZIndex(100), // Forces this node to the very front of the screen
            LevelCompleteOverlay,
        ))
        .with_child((
            Text::new("LEVEL CLEARED"),
            TextFont {
                font_size: 80.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
}

pub fn despawn_level_complete_overlay(
    mut commands: Commands,
    query: Query<Entity, With<LevelCompleteOverlay>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn ui_overlay_plugin(app: &mut App) {
    app.add_systems(
        OnEnter(AppState::LevelComplete),
        spawn_level_complete_overlay,
    );
    app.add_systems(
        OnExit(AppState::LevelComplete),
        despawn_level_complete_overlay,
    );
    app.add_systems(
        Update,
        (update_next_blocks_ui, update_score_ui).run_if(in_state(AppState::InGame)),
    );
}
