//! Spawn the main level by triggering other observers.

use bevy::prelude::*;

use super::{player::SpawnPlayer, sequencer::SpawnSequencer};

/// The Y coordinate of the floor
pub const FLOOR_Y: f32 = 100.0;

/// The width of the level, in pixels
pub const FLOOR_WIDTH: f32 = 1280.0;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

#[derive(Component)]
pub struct RectCollider(pub Vec2);

#[derive(Component)]
pub struct Floor;

fn spawn_level(_trigger: Trigger<SpawnLevel>, mut commands: Commands) {
    commands.trigger(SpawnPlayer);
    commands.trigger(SpawnSequencer);

    commands.spawn((
        Name::new("Floor"),
        Floor,
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(FLOOR_WIDTH, 2.0)),
                color: Color::BLACK,
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, FLOOR_Y, 0.0)),
            ..default()
        },
        RectCollider(Vec2::new(FLOOR_WIDTH, 2.0)),
    ));

    let curtain_width = 5000.0;
    let curtain_height = 5000.0;
    let curtain_center_distance = (curtain_width / 2.0) + (FLOOR_WIDTH / 2.0);
    commands.spawn((
        Name::new("Left curtain"),
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(curtain_width, curtain_height)),
                color: Color::BLACK,
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(-curtain_center_distance, 0.0, 1.0)),
            ..default()
        },
    ));
    commands.spawn((
        Name::new("Right curtain"),
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(curtain_width, curtain_height)),
                color: Color::BLACK,
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(curtain_center_distance, 0.0, 1.0)),
            ..default()
        },
    ));

    commands.insert_resource(ClearColor(Color::srgb(0.35, 0.35, 0.35)));
}
