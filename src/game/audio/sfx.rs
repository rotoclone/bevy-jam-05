use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};

use crate::game::assets::{HandleMap, SfxKey};

pub(super) fn plugin(app: &mut App) {
    app.observe(play_sfx);
}

fn play_sfx(
    trigger: Trigger<PlaySfx>,
    mut commands: Commands,
    sfx_handles: Res<HandleMap<SfxKey>>,
) {
    commands.spawn(AudioSourceBundle {
        source: sfx_handles.get(trigger.event().0),
        settings: PlaybackSettings {
            mode: PlaybackMode::Despawn,
            volume: Volume::new(0.5),
            ..default()
        },
    });
}

/// Trigger this event to play a single sound effect.
#[derive(Event)]
pub struct PlaySfx(pub SfxKey);
