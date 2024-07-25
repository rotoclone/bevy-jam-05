//! Handle player input and translate it into movement.
//! Note that the approach used here is simple for demonstration purposes.
//! If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/latest/examples/movement/physics_in_fixed_timestep.rs).

use bevy::prelude::*;

use crate::AppSet;

use super::spawn::{
    level::{FLOOR_Y, LEVEL_WIDTH},
    player::Player,
};

/// Gravity in pixels/sec^2
const GRAVITY: f32 = 2200.0;

/// Jump velocity in pixels/sec
const JUMP_VELOCITY: f32 = 600.0;

pub(super) fn plugin(app: &mut App) {
    app.observe(do_player_action);

    app.register_type::<WrapWithinLevel>();
    app.add_systems(
        Update,
        (apply_movement, wrap_within_level)
            .chain()
            .in_set(AppSet::Update),
    );
}

/// Event that makes the player do something
#[derive(Event)]
pub enum PlayerAction {
    SetSpeed(f32),
    Jump,
}

fn do_player_action(
    trigger: Trigger<PlayerAction>,
    mut movement_query: Query<&mut MovementController>,
) {
    for mut controller in &mut movement_query {
        match trigger.event() {
            PlayerAction::SetSpeed(x) => controller.speed = *x,
            PlayerAction::Jump => {
                if !controller.jumping {
                    controller.jumping = true;
                    controller.vertical_velocity = JUMP_VELOCITY;
                }
            }
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MovementController {
    pub speed: f32,
    pub jumping: bool,
    pub vertical_velocity: f32,
}

impl MovementController {
    pub fn new() -> MovementController {
        MovementController {
            speed: 0.0,
            jumping: false,
            vertical_velocity: 0.0,
        }
    }
}

fn apply_movement(
    time: Res<Time>,
    mut movement_query: Query<(&Player, &mut MovementController, &mut Transform)>,
) {
    for (player, mut controller, mut transform) in &mut movement_query {
        let velocity = Vec2::new(controller.speed, 0.0);
        transform.translation += velocity.extend(0.0) * time.delta_seconds();

        // why import a physics library when I can just implement a bad one myself
        let bottom_of_player = transform.translation.y - player.collider_radius;
        let distance_from_floor = bottom_of_player - FLOOR_Y;
        if distance_from_floor > f32::EPSILON || controller.vertical_velocity > f32::EPSILON {
            // player is in the air, or should be in the air
            let proposed_y =
                transform.translation.y + (controller.vertical_velocity * time.delta_seconds());
            let min_y = FLOOR_Y + player.collider_radius;
            transform.translation.y = proposed_y.max(min_y);
            controller.vertical_velocity -= GRAVITY * time.delta_seconds();
        } else {
            // player is on the ground
            controller.vertical_velocity = 0.0;
            controller.jumping = false;
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct WrapWithinLevel;

fn wrap_within_level(mut wrap_query: Query<&mut Transform, With<WrapWithinLevel>>) {
    let size = Vec2::new(LEVEL_WIDTH + (24.0 * 3.0), LEVEL_WIDTH);
    let half_size = size / 2.0;
    for mut transform in &mut wrap_query {
        let position = transform.translation.xy();
        let wrapped = (position + half_size).rem_euclid(size) - half_size;
        transform.translation = wrapped.extend(transform.translation.z);
    }
}
