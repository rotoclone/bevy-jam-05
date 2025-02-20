//! Spawn the main level by triggering other observers.

use bevy::prelude::*;

use crate::{
    game::{
        assets::{FontKey, HandleMap, ImageKey},
        movement::TotalDistance,
        SHOW_COLLIDERS,
    },
    ui::palette::LABEL_TEXT,
    AppSet,
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

const TOP_OF_FLOOR: f32 = FLOOR_Y + (FLOOR_HEIGHT / 2.0);

pub const TOTAL_LEVELS: u32 = 4;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);
    app.observe(spawn_distance_display);
    app.observe(spawn_obstacles);
    app.insert_resource(CurrentLevel(0));

    app.add_systems(Update, update_distance_display.in_set(AppSet::Update));
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

#[derive(Event, Debug)]
pub struct SpawnDistanceDisplay;

#[derive(Event, Debug)]
pub struct SpawnObstacles(pub u32);

#[derive(Resource, Debug)]
pub struct CurrentLevel(pub u32);

#[derive(Component)]
pub struct DistanceDisplayText;

#[derive(Component)]
pub struct Obstacle;

#[derive(Component)]
pub struct Background;

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
    commands.trigger(SpawnDistanceDisplay);
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

fn spawn_distance_display(
    _trigger: Trigger<SpawnDistanceDisplay>,
    font_handles: Res<HandleMap<FontKey>>,
    mut commands: Commands,
) {
    let mut entity = commands.spawn((
        Name::new("Distance display"),
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(35.0),
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        },
    ));
    entity.with_children(|children| {
        children.spawn((
            Name::new("Distance display text"),
            DistanceDisplayText,
            TextBundle::from_section(
                "Distance: 0",
                TextStyle {
                    font: font_handles.get(FontKey::General),
                    font_size: 30.0,
                    color: LABEL_TEXT,
                },
            ),
        ));
    });
}

fn update_distance_display(
    mut distance_display_text_query: Query<&mut Text, With<DistanceDisplayText>>,
    total_distance: Res<TotalDistance>,
) {
    for mut text in &mut distance_display_text_query {
        text.sections[0].value = format!("Distance: {}", *total_distance);
    }
}

fn spawn_obstacles(
    trigger: Trigger<SpawnObstacles>,
    existing_obstacles_query: Query<Entity, With<Obstacle>>,
    background_query: Query<Entity, With<Background>>,
    image_handles: Res<HandleMap<ImageKey>>,
    mut commands: Commands,
) {
    for existing_obstacle in &existing_obstacles_query {
        commands.entity(existing_obstacle).despawn_recursive();
    }

    for background in &background_query {
        commands.entity(background).despawn_recursive();
    }

    match trigger.event().0 % TOTAL_LEVELS {
        0 => spawn_level_0(&image_handles, &mut commands),
        1 => spawn_level_1(&image_handles, &mut commands),
        2 => spawn_level_2(&image_handles, &mut commands),
        3 => spawn_level_3(&image_handles, &mut commands),
        _ => unreachable!(),
    }
}

fn spawn_level_0(image_handles: &HandleMap<ImageKey>, commands: &mut Commands) {
    spawn_background(Color::srgb(0.6, 0.4, 0.4), commands);

    spawn_box(
        Vec2::new(0.0, TOP_OF_FLOOR + (BOX_SIZE / 2.0)),
        image_handles,
        commands,
    );
    spawn_floor_spikes(
        Vec2::new(
            (BOX_SIZE / 2.0) + (SPIKES_IMAGE_SIZE / 2.0),
            TOP_OF_FLOOR + (SPIKES_IMAGE_SIZE / 2.0),
        ),
        image_handles,
        commands,
    );
}

fn spawn_level_1(image_handles: &HandleMap<ImageKey>, commands: &mut Commands) {
    spawn_background(Color::srgb(0.4, 0.6, 0.4), commands);

    spawn_box_with_spikes_on_side(
        Vec2::new(-BOX_SIZE, TOP_OF_FLOOR + (BOX_SIZE / 2.0)),
        image_handles,
        commands,
    );
    spawn_floor_spikes(
        Vec2::new(
            -BOX_SIZE,
            TOP_OF_FLOOR + BOX_SIZE + (SPIKES_IMAGE_SIZE / 2.0),
        ),
        image_handles,
        commands,
    );
}

fn spawn_level_2(image_handles: &HandleMap<ImageKey>, commands: &mut Commands) {
    spawn_background(Color::srgb(0.4, 0.4, 0.6), commands);

    spawn_box(
        Vec2::new(BOX_SIZE * -3.0, TOP_OF_FLOOR + (BOX_SIZE / 2.0)),
        image_handles,
        commands,
    );

    spawn_box_with_spikes_on_side(
        Vec2::new(0.0, TOP_OF_FLOOR + (BOX_SIZE * 3.0)),
        image_handles,
        commands,
    );

    spawn_box(
        Vec2::new(BOX_SIZE * 3.0, TOP_OF_FLOOR + (BOX_SIZE / 2.0)),
        image_handles,
        commands,
    );
}

fn spawn_level_3(image_handles: &HandleMap<ImageKey>, commands: &mut Commands) {
    spawn_background(Color::srgb(0.6, 0.6, 0.4), commands);

    spawn_box(
        Vec2::new(BOX_SIZE * -4.0, TOP_OF_FLOOR + (BOX_SIZE / 2.0)),
        image_handles,
        commands,
    );
    spawn_box(
        Vec2::new(BOX_SIZE * -3.0, TOP_OF_FLOOR + BOX_SIZE + (BOX_SIZE / 2.0)),
        image_handles,
        commands,
    );
    spawn_box(
        Vec2::new(
            BOX_SIZE * -2.0,
            TOP_OF_FLOOR + (BOX_SIZE * 2.0) + (BOX_SIZE / 2.0),
        ),
        image_handles,
        commands,
    );

    spawn_box_with_spikes_on_side(
        Vec2::new(
            BOX_SIZE * 2.0,
            TOP_OF_FLOOR + (BOX_SIZE * 5.0) + (BOX_SIZE / 2.0),
        ),
        image_handles,
        commands,
    );

    spawn_box_with_spikes_on_side(
        Vec2::new(
            BOX_SIZE * 2.0,
            TOP_OF_FLOOR + (BOX_SIZE * 4.0) + (BOX_SIZE / 2.0),
        ),
        image_handles,
        commands,
    );
    spawn_box_with_spikes_on_side(
        Vec2::new(
            BOX_SIZE * 2.0,
            TOP_OF_FLOOR + (BOX_SIZE * 3.0) + (BOX_SIZE / 2.0),
        ),
        image_handles,
        commands,
    );
    spawn_box_with_spikes_on_side(
        Vec2::new(
            BOX_SIZE * 2.0,
            TOP_OF_FLOOR + (BOX_SIZE * 2.0) + (BOX_SIZE / 2.0),
        ),
        image_handles,
        commands,
    );
}

fn spawn_background(color: Color, commands: &mut Commands) {
    commands.spawn((
        Name::new("Background"),
        Background,
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(LEVEL_WIDTH, LEVEL_WIDTH)),
                ..default()
            },
            ..default()
        },
    ));
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
        bounds: Vec2::new(
            SPIKES_WIDTH - (4.0 * IMAGE_SCALE),
            SPIKES_HEIGHT - IMAGE_SCALE,
        ),
        offset: Vec2::new(0.0, -7.0 * IMAGE_SCALE),
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

fn spawn_wall_spikes(position: Vec2, image_handles: &HandleMap<ImageKey>, commands: &mut Commands) {
    let collider = RectCollider {
        bounds: Vec2::new(
            SPIKES_HEIGHT - IMAGE_SCALE,
            SPIKES_WIDTH - (4.0 * IMAGE_SCALE),
        ),
        offset: Vec2::new(7.0 * IMAGE_SCALE, 0.0),
    };
    commands
        .spawn((
            Name::new("Spikes"),
            Obstacle,
            Spikes,
            SpriteBundle {
                texture: image_handles.get(ImageKey::Spikes),
                transform: Transform::from_scale(Vec2::splat(IMAGE_SCALE).extend(1.0))
                    .with_translation(Vec3::new(position.x, position.y, 0.0))
                    .with_rotation(Quat::from_rotation_z(90.0_f32.to_radians())),
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
                            (Vec2::new(collider.offset.y, -collider.offset.x) / IMAGE_SCALE)
                                .extend(1.0),
                        )
                        .with_rotation(Quat::from_rotation_z(90.0_f32.to_radians())),
                        ..default()
                    },
                ));
            }
        });
}

fn spawn_box_with_spikes_on_side(
    position: Vec2,
    image_handles: &HandleMap<ImageKey>,
    commands: &mut Commands,
) {
    spawn_box(position, image_handles, commands);
    spawn_wall_spikes(
        Vec2::new(
            position.x - (BOX_SIZE / 2.0) - (SPIKES_IMAGE_SIZE / 2.0),
            position.y,
        ),
        image_handles,
        commands,
    );
}
