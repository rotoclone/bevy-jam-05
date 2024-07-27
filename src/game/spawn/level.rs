//! Spawn the main level by triggering other observers.

use bevy::prelude::*;

use crate::game::{
    assets::{HandleMap, ImageKey},
    SHOW_COLLIDERS,
};

use super::{player::SpawnPlayer, sequencer::SpawnSequencer};

/// The Y coordinate of the floor
pub const FLOOR_Y: f32 = 100.0;

/// The width of the level, in pixels
pub const LEVEL_WIDTH: f32 = 1280.0;

/// The thickness of the floor, in pixels
pub const FLOOR_HEIGHT: f32 = 2.0;

const IMAGE_SCALE: f32 = 3.0;

const BOX_RAW_IMAGE_SIZE: f32 = 19.0;
const BOX_SIZE: f32 = BOX_RAW_IMAGE_SIZE * IMAGE_SCALE;

const SPIKES_RAW_IMAGE_SIZE: f32 = 19.0;
const SPIKES_IMAGE_SIZE: f32 = SPIKES_RAW_IMAGE_SIZE * IMAGE_SCALE;
const SPIKES_WIDTH: f32 = SPIKES_IMAGE_SIZE;
const SPIKES_HEIGHT: f32 = 6.0 * IMAGE_SCALE;

const TOTAL_LEVELS: u32 = 6;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);
    app.observe(spawn_obstacles);
    app.insert_resource(CurrentLevel(0));
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

#[derive(Event, Debug)]
pub struct SpawnObstacles(pub u32);

#[derive(Resource, Debug)]
pub struct CurrentLevel(pub u32);

#[derive(Component)]
pub struct Obstacle;

#[derive(Component, Clone)]
pub struct RectCollider {
    pub bounds: Vec2,
    pub offset: Vec2,
}

#[derive(Component)]
pub struct Floor;

#[derive(Component)]
pub struct Spikes;

fn spawn_level(
    _trigger: Trigger<SpawnLevel>,
    current_level: Res<CurrentLevel>,
    mut commands: Commands,
) {
    commands.trigger(SpawnPlayer);
    commands.trigger(SpawnSequencer);
    commands.trigger(SpawnObstacles(current_level.0));

    commands.spawn((
        Name::new("Floor"),
        Floor,
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(LEVEL_WIDTH + 500.0, FLOOR_HEIGHT)),
                color: Color::BLACK,
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, FLOOR_Y, 0.0)),
            ..default()
        },
        RectCollider {
            bounds: Vec2::new(LEVEL_WIDTH + 500.0, 2.0),
            offset: Vec2::ZERO,
        },
    ));

    let curtain_width = 5000.0;
    let curtain_height = 5000.0;
    let curtain_center_distance = (curtain_width / 2.0) + (LEVEL_WIDTH / 2.0);
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

fn spawn_obstacles(
    trigger: Trigger<SpawnObstacles>,
    existing_obstacles_query: Query<Entity, With<Obstacle>>,
    image_handles: Res<HandleMap<ImageKey>>,
    mut commands: Commands,
) {
    for existing_obstacle in &existing_obstacles_query {
        commands.entity(existing_obstacle).despawn_recursive();
    }

    match trigger.event().0 % TOTAL_LEVELS {
        0 => spawn_level_0(&image_handles, &mut commands),
        1 => spawn_level_1(&image_handles, &mut commands),
        2 => spawn_level_2(&image_handles, &mut commands),
        3 => spawn_level_3(&image_handles, &mut commands),
        4 => spawn_level_4(&image_handles, &mut commands),
        5 => spawn_level_5(&image_handles, &mut commands),
        _ => unreachable!(),
    }
}

fn spawn_level_0(image_handles: &HandleMap<ImageKey>, commands: &mut Commands) {
    let top_of_floor = FLOOR_Y + (FLOOR_HEIGHT / 2.0);
    spawn_box(
        Vec2::new(0.0, top_of_floor + (BOX_SIZE / 2.0)),
        image_handles,
        commands,
    );
    spawn_floor_spikes(
        Vec2::new(
            (BOX_SIZE / 2.0) + (SPIKES_IMAGE_SIZE / 2.0),
            top_of_floor + (SPIKES_IMAGE_SIZE / 2.0),
        ),
        image_handles,
        commands,
    );
}

fn spawn_level_1(image_handles: &HandleMap<ImageKey>, commands: &mut Commands) {
    let top_of_floor = FLOOR_Y + (FLOOR_HEIGHT / 2.0);
    spawn_floor_spikes(
        Vec2::new(
            (-BOX_SIZE / 2.0) - (SPIKES_IMAGE_SIZE / 2.0),
            top_of_floor + (SPIKES_IMAGE_SIZE / 2.0),
        ),
        image_handles,
        commands,
    );
    spawn_box(
        Vec2::new(0.0, top_of_floor + (BOX_SIZE / 2.0)),
        image_handles,
        commands,
    );
    spawn_box(
        Vec2::new(0.0, top_of_floor + (BOX_SIZE / 2.0)),
        image_handles,
        commands,
    );
    spawn_floor_spikes(
        Vec2::new(0.0, top_of_floor + BOX_SIZE + (SPIKES_IMAGE_SIZE / 2.0)),
        image_handles,
        commands,
    );
}

fn spawn_level_2(image_handles: &HandleMap<ImageKey>, commands: &mut Commands) {
    todo!() //TODO
}

fn spawn_level_3(image_handles: &HandleMap<ImageKey>, commands: &mut Commands) {
    todo!() //TODO
}

fn spawn_level_4(image_handles: &HandleMap<ImageKey>, commands: &mut Commands) {
    todo!() //TODO
}

fn spawn_level_5(image_handles: &HandleMap<ImageKey>, commands: &mut Commands) {
    todo!() //TODO
}

fn spawn_box(position: Vec2, image_handles: &HandleMap<ImageKey>, commands: &mut Commands) {
    let collider = RectCollider {
        bounds: Vec2::new(BOX_SIZE, BOX_SIZE),
        offset: Vec2::ZERO,
    };
    commands
        .spawn((
            Name::new("Box"),
            Obstacle,
            SpriteBundle {
                texture: image_handles.get(ImageKey::Box),
                transform: Transform::from_scale(Vec2::splat(IMAGE_SCALE).extend(1.0))
                    .with_translation(Vec3::new(position.x, position.y, 0.0)),
                ..Default::default()
            },
            collider.clone(),
        ))
        .with_children(|children| {
            if SHOW_COLLIDERS {
                children.spawn((
                    Name::new("Box collider visualization"),
                    SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(collider.bounds / IMAGE_SCALE),
                            color: Color::srgba(0.0, 1.0, 0.0, 0.3),
                            ..default()
                        },
                        transform: Transform::from_translation(
                            (collider.offset / IMAGE_SCALE).extend(1.0),
                        ),
                        ..default()
                    },
                ));
            }
        });
}

fn spawn_floor_spikes(
    position: Vec2,
    image_handles: &HandleMap<ImageKey>,
    commands: &mut Commands,
) {
    let collider = RectCollider {
        bounds: Vec2::new(SPIKES_WIDTH, SPIKES_HEIGHT),
        offset: Vec2::new(0.0, -6.0 * IMAGE_SCALE),
    };
    commands
        .spawn((
            Name::new("Spikes"),
            Obstacle,
            Spikes,
            SpriteBundle {
                texture: image_handles.get(ImageKey::Spikes),
                transform: Transform::from_scale(Vec2::splat(IMAGE_SCALE).extend(1.0))
                    .with_translation(Vec3::new(position.x, position.y, 0.0)),
                ..Default::default()
            },
            collider.clone(),
        ))
        .with_children(|children| {
            if SHOW_COLLIDERS {
                children.spawn((
                    Name::new("Spikes collider visualization"),
                    SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(collider.bounds / IMAGE_SCALE),
                            color: Color::srgba(0.0, 1.0, 0.0, 0.3),
                            ..default()
                        },
                        transform: Transform::from_translation(
                            (collider.offset / IMAGE_SCALE).extend(1.0),
                        ),
                        ..default()
                    },
                ));
            }
        });
}
