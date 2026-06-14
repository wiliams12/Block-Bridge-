use bevy::audio::Volume;
use bevy::prelude::*;

pub fn init_soundtrack(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioPlayer::new(asset_server.load("audio/soundtrack.ogg")), // Use your file name
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.3)),
    ));
}

pub fn audio_plugin(app: &mut App) {
    app.add_systems(Startup, init_soundtrack);
}
