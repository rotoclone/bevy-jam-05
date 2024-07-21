//! Spawn the main level by triggering other observers.

use bevy::prelude::*;

use super::{player::SpawnPlayer, sequencer::SpawnSequencer};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

fn spawn_level(_trigger: Trigger<SpawnLevel>, mut commands: Commands) {
    commands.trigger(SpawnPlayer);
    commands.trigger(SpawnSequencer);

    commands.insert_resource(ClearColor(Color::srgb(0.35, 0.35, 0.35)));
}
