//! Spawn the player.

use bevy::prelude::*;

use crate::{
    game::{
        animation::PlayerAnimation,
        assets::{HandleMap, ImageKey},
        movement::MovementController,
        SHOW_COLLIDERS,
    },
    screen::Screen,
};

use super::level::{FLOOR_Y, LEVEL_WIDTH};

const PLAYER_SCALE: f32 = 3.0;
const PLAYER_RAW_IMAGE_SIZE: f32 = 24.0;
pub const PLAYER_IMAGE_SIZE: f32 = PLAYER_RAW_IMAGE_SIZE * PLAYER_SCALE;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_player);
    app.register_type::<Player>();
}

#[derive(Event, Debug)]
pub struct SpawnPlayer;

#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Reflect)]
#[reflect(Component)]
pub struct Player {
    pub collider: Vec2,
    pub collider_offset: Vec2,
}

fn spawn_player(
    _trigger: Trigger<SpawnPlayer>,
    mut commands: Commands,
    image_handles: Res<HandleMap<ImageKey>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    existing_player_query: Query<Entity, With<Player>>,
) {
    // despawn any existing player(s)
    for existing_player in &existing_player_query {
        commands.entity(existing_player).despawn_recursive();
    }

    // A texture atlas is a way to split one image with a grid into multiple sprites.
    // By attaching it to a [`SpriteBundle`] and providing an index, we can specify which section of the image we want to see.
    // We will use this to animate our player character. You can learn more about texture atlases in this example:
    // https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(
        UVec2::splat(PLAYER_RAW_IMAGE_SIZE as u32),
        7,
        3,
        Some(UVec2::splat(0)),
        None,
    );
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_animation = PlayerAnimation::new();
    let collider_size = Vec2::new(7.5 * PLAYER_SCALE, 22.0 * PLAYER_SCALE);
    let collider_offset = Vec2::new(5.5 * PLAYER_SCALE, -1.0 * PLAYER_SCALE);

    commands
        .spawn((
            Name::new("Player"),
            Player {
                collider: collider_size,
                collider_offset,
            },
            SpriteBundle {
                texture: image_handles.get(ImageKey::Player),
                transform: Transform::from_scale(Vec2::splat(PLAYER_SCALE).extend(1.0))
                    .with_translation(Vec3::new(
                        (-LEVEL_WIDTH / 2.0) + (PLAYER_IMAGE_SIZE / 2.0),
                        FLOOR_Y - collider_offset.y + (collider_size.y / 2.0) + 1.0,
                        0.0,
                    )),
                ..Default::default()
            },
            TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: player_animation.get_atlas_index(),
            },
            MovementController::new(),
            player_animation,
            StateScoped(Screen::Playing),
        ))
        .with_children(|children| {
            if SHOW_COLLIDERS {
                children.spawn((
                    Name::new("Player collider visualization"),
                    SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(collider_size / PLAYER_SCALE),
                            color: Color::srgba(0.0, 1.0, 0.0, 0.3),
                            ..default()
                        },
                        transform: Transform::from_translation(
                            (collider_offset / PLAYER_SCALE).extend(1.0),
                        ),
                        ..default()
                    },
                ));
            }
        });
}
