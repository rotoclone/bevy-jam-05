//! Handle player input and translate it into movement.
//! Note that the approach used here is simple for demonstration purposes.
//! If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/latest/examples/movement/physics_in_fixed_timestep.rs).

use bevy::prelude::*;

use crate::AppSet;

use super::spawn::{
    level::{Floor, RectCollider},
    player::Player,
};

/// Gravity in pixels/sec^2
const GRAVITY: f32 = 2200.0;

/// Jump velocity in pixels/sec
const JUMP_VELOCITY: f32 = 600.0;

pub(super) fn plugin(app: &mut App) {
    app.observe(do_player_action);

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
    collider_query: Query<(&Transform, &RectCollider), Without<Player>>,
) {
    for (player, mut controller, mut player_transform) in &mut movement_query {
        let velocity = Vec2::new(controller.speed, 0.0);
        player_transform.translation += velocity.extend(0.0) * time.delta_seconds();
        //TODO check for obstacles in front of the player

        // why import a physics library when I can just implement a bad one myself
        for (transform, collider) in &collider_query {
            // first check if the player is above the thing
            let player_left_edge = player_transform.translation.x - (player.collider.x / 2.0);
            let player_right_edge = player_transform.translation.x + (player.collider.x / 2.0);
            let obstacle_left_edge = transform.translation.x - (collider.0.x / 2.0);
            let obstacle_right_edge = transform.translation.x + (collider.0.x / 2.0);

            if player_left_edge > obstacle_right_edge || player_right_edge < obstacle_left_edge {
                // player isn't above the obstacle so it doesn't matter
                continue;
            }

            let bottom_of_player = player_transform.translation.y - (player.collider.y / 2.0);
            let top_of_obstacle = transform.translation.y + (collider.0.y / 2.0);
            let distance_from_floor = bottom_of_player - top_of_obstacle;
            if distance_from_floor > f32::EPSILON || controller.vertical_velocity > f32::EPSILON {
                // player is in the air, or should be in the air
                let proposed_y = player_transform.translation.y
                    + (controller.vertical_velocity * time.delta_seconds());
                let min_y = top_of_obstacle + (player.collider.y / 2.0);
                player_transform.translation.y = proposed_y.max(min_y);
                controller.vertical_velocity -= GRAVITY * time.delta_seconds();
            } else {
                // player is on the obstacle
                controller.vertical_velocity = 0.0;
                controller.jumping = false;
            }
        }
    }
}

fn wrap_within_level(
    mut wrap_query: Query<(&mut Transform, &Player), Without<Floor>>,
    floor_query: Query<(&Transform, &RectCollider), With<Floor>>,
) {
    if let Ok((_, floor_collider)) = floor_query.get_single() {
        for (mut transform, player) in &mut wrap_query {
            let size = Vec2::new(floor_collider.0.x + (player.collider.x * 2.0), 1000.0);
            let half_size = size / 2.0;
            let position = transform.translation.xy();
            let wrapped = (position + half_size).rem_euclid(size) - half_size;
            transform.translation = wrapped.extend(transform.translation.z);
        }
    }
}
