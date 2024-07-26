//! Spawn the player.

use bevy::prelude::*;

use crate::{
    game::{
        animation::PlayerAnimation,
        assets::{HandleMap, ImageKey},
        movement::MovementController,
    },
    screen::Screen,
};

use super::level::{FLOOR_WIDTH, FLOOR_Y};

const PLAYER_SCALE: f32 = 3.0;

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
}

fn spawn_player(
    _trigger: Trigger<SpawnPlayer>,
    mut commands: Commands,
    image_handles: Res<HandleMap<ImageKey>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // A texture atlas is a way to split one image with a grid into multiple sprites.
    // By attaching it to a [`SpriteBundle`] and providing an index, we can specify which section of the image we want to see.
    // We will use this to animate our player character. You can learn more about texture atlases in this example:
    // https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(24), 7, 3, Some(UVec2::splat(0)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_animation = PlayerAnimation::new();
    let player_size = 24.0 * PLAYER_SCALE;

    commands.spawn((
        Name::new("Player"),
        Player {
            collider: Vec2::new(player_size, player_size),
        },
        SpriteBundle {
            texture: image_handles.get(ImageKey::Player),
            transform: Transform::from_scale(Vec2::splat(PLAYER_SCALE).extend(1.0))
                .with_translation(Vec3::new(
                    (-FLOOR_WIDTH / 2.0) + (player_size / 2.0),
                    FLOOR_Y + player_size,
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
    ));
}
