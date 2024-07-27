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
const GRAVITY: f32 = 2300.0;

/// Jump velocity in pixels/sec
const JUMP_VELOCITY: f32 = 800.0;

/// Velocity added on float in pixels/sec
const FLOAT_VELOCITY: f32 = 600.0;

/// The maximum final velocity after a float in pixels/sec
const FLOAT_LIMIT: f32 = -10.0;

/// The velocity added on dive in pixels/sec
const DIVE_VELOCITY: f32 = -600.0;

/// The minimum final velocity after a dive in pixels/sec
const DIVE_LIMIT: f32 = -600.0;

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
    Float,
    Dive,
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
            PlayerAction::Float => {
                if controller.jumping && controller.vertical_velocity < FLOAT_LIMIT {
                    controller.vertical_velocity =
                        (controller.vertical_velocity + FLOAT_VELOCITY).min(FLOAT_LIMIT);
                }
            }
            PlayerAction::Dive => {
                if controller.jumping && controller.vertical_velocity > DIVE_LIMIT {
                    controller.vertical_velocity =
                        (controller.vertical_velocity + DIVE_VELOCITY).max(DIVE_LIMIT);
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
        // why import a physics library when I can just implement a bad one myself
        let player_left_edge = player_transform.translation.x - (player.collider.x / 2.0);
        let player_right_edge = player_transform.translation.x + (player.collider.x / 2.0);
        let player_top = player_transform.translation.y + (player.collider.y / 2.0);
        let player_bottom = player_transform.translation.y - (player.collider.y / 2.0);

        // find closest thing to run into when moving to the right
        let mut left_of_closest_wall = None;
        for (transform, collider) in &collider_query {
            let obstacle_left_edge = transform.translation.x - (collider.0.x / 2.0);
            let obstacle_top = transform.translation.y + (collider.0.y / 2.0);
            let obstacle_bottom = transform.translation.y - (collider.0.y / 2.0);

            if !(player_bottom > obstacle_top || player_top < obstacle_bottom)
                && player_right_edge <= obstacle_left_edge
            {
                // player is to the left of obstacle and at the same height
                let distance_from_left_side_of_obstacle = obstacle_left_edge - player_right_edge;
                if let Some(other_left) = left_of_closest_wall {
                    let other_distance_from_left = other_left - player_right_edge;
                    if distance_from_left_side_of_obstacle < other_distance_from_left {
                        left_of_closest_wall = Some(obstacle_left_edge);
                    }
                } else {
                    left_of_closest_wall = Some(obstacle_left_edge);
                }
            }
        }

        // move rightwards
        if let Some(left_of_obstacle) = left_of_closest_wall {
            let distance_from_left_of_obstacle = left_of_obstacle - player_right_edge;
            if distance_from_left_of_obstacle > f32::EPSILON {
                // player can move
                let proposed_x =
                    player_transform.translation.x + (controller.speed * time.delta_seconds());
                let max_x = left_of_obstacle - (player.collider.x / 2.0);
                player_transform.translation.x = proposed_x.min(max_x);
            }
        } else {
            // no walls to worry about running into
            player_transform.translation.x += controller.speed * time.delta_seconds();
        }

        // find closest thing to run into when falling
        let mut top_of_closest_floor = None;
        for (transform, collider) in &collider_query {
            let obstacle_left_edge = transform.translation.x - (collider.0.x / 2.0);
            let obstacle_right_edge = transform.translation.x + (collider.0.x / 2.0);
            let obstacle_top = transform.translation.y + (collider.0.y / 2.0);

            if !(player_left_edge > obstacle_right_edge || player_right_edge < obstacle_left_edge) {
                // player is above obstacle
                let distance_from_top_of_obstacle = player_bottom - obstacle_top;
                if let Some(other_top) = top_of_closest_floor {
                    let other_distance_from_top = player_bottom - other_top;
                    if distance_from_top_of_obstacle < other_distance_from_top {
                        top_of_closest_floor = Some(obstacle_top);
                    }
                } else {
                    top_of_closest_floor = Some(obstacle_top);
                }
            }
        }

        // move downwards
        if let Some(top_of_obstacle) = top_of_closest_floor {
            let distance_from_top_of_obstacle = player_bottom - top_of_obstacle;
            if distance_from_top_of_obstacle > f32::EPSILON
                || controller.vertical_velocity > f32::EPSILON
            {
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
        } else {
            // freefall
            player_transform.translation.y += controller.vertical_velocity * time.delta_seconds();
            controller.vertical_velocity -= GRAVITY * time.delta_seconds();
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
