//! Handle player input and translate it into movement.
//! Note that the approach used here is simple for demonstration purposes.
//! If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/latest/examples/movement/physics_in_fixed_timestep.rs).

use bevy::prelude::*;

use crate::AppSet;

use super::spawn::{
    level::{CurrentLevel, RectCollider, SpawnObstacles, Spikes, LEVEL_WIDTH},
    player::{Player, PLAYER_IMAGE_SIZE},
    sequencer::{Dead, DeathEvent, PauseSequence, PlaySequence},
};

/// Gravity in pixels/sec^2
const GRAVITY: f32 = 2300.0;

/// Jump velocity in pixels/sec
const JUMP_VELOCITY: f32 = 800.0;

/// Velocity added on float in pixels/sec
const FLOAT_VELOCITY: f32 = 1000.0;

/// The maximum final velocity after a float in pixels/sec
const FLOAT_LIMIT: f32 = -10.0;

/// The velocity added on dive in pixels/sec
const DIVE_VELOCITY: f32 = -800.0;

/// The minimum final velocity after a dive in pixels/sec
const DIVE_LIMIT: f32 = -800.0;

pub(super) fn plugin(app: &mut App) {
    app.observe(do_player_action);
    app.observe(pause);
    app.observe(resume);

    app.insert_resource(TotalDistance(0.0));
    app.insert_resource(Paused(true));

    app.add_systems(
        Update,
        (apply_movement, check_spike_collisions, wrap_within_level)
            .chain()
            .in_set(AppSet::Update),
    );
}

#[derive(Resource, Debug)]
pub struct TotalDistance(pub f32);

impl std::fmt::Display for TotalDistance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (((self.0 / LEVEL_WIDTH) * 50.0).round() as u32).fmt(f)
    }
}

#[derive(Resource, Debug)]
pub struct Paused(pub bool);

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

fn pause(_trigger: Trigger<PauseSequence>, mut paused: ResMut<Paused>) {
    paused.0 = true;
}

fn resume(_trigger: Trigger<PlaySequence>, mut paused: ResMut<Paused>, dead: Res<Dead>) {
    if dead.0 {
        return;
    }

    paused.0 = false;
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
    paused: Res<Paused>,
    mut total_distance: ResMut<TotalDistance>,
) {
    if paused.0 {
        return;
    }

    for (player, mut controller, mut player_transform) in &mut movement_query {
        // why import a physics library when I can just implement a bad one myself
        let player_left_edge =
            player_transform.translation.x + player.collider_offset.x - (player.collider.x / 2.0);
        let player_right_edge =
            player_transform.translation.x + player.collider_offset.x + (player.collider.x / 2.0);
        let player_top =
            player_transform.translation.y + player.collider_offset.y + (player.collider.y / 2.0);
        let player_bottom =
            player_transform.translation.y + player.collider_offset.y - (player.collider.y / 2.0);

        // find closest thing to run into when moving to the right
        let mut left_of_closest_wall = None;
        for (transform, collider) in &collider_query {
            let obstacle_left_edge =
                transform.translation.x + collider.offset.x - (collider.bounds.x / 2.0);
            let obstacle_top =
                transform.translation.y + collider.offset.y + (collider.bounds.y / 2.0);
            let obstacle_bottom =
                transform.translation.y + collider.offset.y - (collider.bounds.y / 2.0);

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
        let original_x = player_transform.translation.x;
        if let Some(left_of_obstacle) = left_of_closest_wall {
            let distance_from_left_of_obstacle = left_of_obstacle - player_right_edge;
            if distance_from_left_of_obstacle > f32::EPSILON {
                // player can move
                let proposed_x =
                    player_transform.translation.x + (controller.speed * time.delta_seconds());
                let max_x = left_of_obstacle - player.collider_offset.x - (player.collider.x / 2.0);
                player_transform.translation.x = proposed_x.min(max_x);
            }
        } else {
            // no walls to worry about running into
            player_transform.translation.x += controller.speed * time.delta_seconds();
        }

        total_distance.0 += player_transform.translation.x - original_x;

        // find closest thing to run into when falling or jumping
        let mut closest_floor_or_ceiling = None;
        for (transform, collider) in &collider_query {
            let obstacle_left_edge =
                transform.translation.x + collider.offset.x - (collider.bounds.x / 2.0);
            let obstacle_right_edge =
                transform.translation.x + collider.offset.x + (collider.bounds.x / 2.0);
            let obstacle_top =
                transform.translation.y + collider.offset.y + (collider.bounds.y / 2.0);
            let obstacle_bottom =
                transform.translation.y + collider.offset.y - (collider.bounds.y / 2.0);

            if controller.vertical_velocity <= 0.0 {
                // falling
                if !(player_left_edge > obstacle_right_edge
                    || player_right_edge < obstacle_left_edge)
                    && obstacle_top <= player_bottom
                {
                    // player is above obstacle
                    let distance_from_top_of_obstacle = player_bottom - obstacle_top;
                    if let Some(other_top) = closest_floor_or_ceiling {
                        let other_distance_from_top = player_bottom - other_top;
                        if distance_from_top_of_obstacle < other_distance_from_top {
                            closest_floor_or_ceiling = Some(obstacle_top);
                        }
                    } else {
                        closest_floor_or_ceiling = Some(obstacle_top);
                    }
                }
            } else {
                // jumping
                if !(player_left_edge > obstacle_right_edge
                    || player_right_edge < obstacle_left_edge)
                    && obstacle_bottom >= player_top
                {
                    // player is below obstacle
                    let distance_from_bottom_of_obstacle = obstacle_bottom - player_top;
                    if let Some(other_bottom) = closest_floor_or_ceiling {
                        let other_distance_from_bottom = other_bottom - player_top;
                        if distance_from_bottom_of_obstacle < other_distance_from_bottom {
                            closest_floor_or_ceiling = Some(obstacle_bottom);
                        }
                    } else {
                        closest_floor_or_ceiling = Some(obstacle_bottom);
                    }
                }
            }
        }

        // move downwards or upwards
        if let Some(closest_floor_or_ceiling) = closest_floor_or_ceiling {
            if controller.vertical_velocity <= 0.0 {
                // falling
                let distance_from_top_of_obstacle = player_bottom - closest_floor_or_ceiling;
                if distance_from_top_of_obstacle > f32::EPSILON {
                    // player is in the air
                    let proposed_y = player_transform.translation.y
                        + (controller.vertical_velocity * time.delta_seconds());
                    let min_y = closest_floor_or_ceiling - player.collider_offset.y
                        + (player.collider.y / 2.0);
                    player_transform.translation.y = proposed_y.max(min_y);
                    if (player_transform.translation.y - min_y).abs() > f32::EPSILON {
                        // player did not hit the obstacle
                        controller.vertical_velocity -= GRAVITY * time.delta_seconds();
                        controller.jumping = true;
                    } else {
                        // player hit the obstacle
                        controller.vertical_velocity = 0.0;
                        controller.jumping = false;
                    }
                }
            } else {
                // jumping
                let distance_from_bottom_of_obstacle = closest_floor_or_ceiling - player_top;
                if distance_from_bottom_of_obstacle > f32::EPSILON {
                    // player has headroom
                    let proposed_y = player_transform.translation.y
                        + (controller.vertical_velocity * time.delta_seconds());
                    let max_y = closest_floor_or_ceiling
                        - player.collider_offset.y
                        - (player.collider.y / 2.0);
                    player_transform.translation.y = proposed_y.min(max_y);
                    if (max_y - player_transform.translation.y).abs() > f32::EPSILON {
                        // player did not hit the obstacle
                        controller.vertical_velocity -= GRAVITY * time.delta_seconds();
                    } else {
                        // player hit the obstacle
                        controller.vertical_velocity = 0.0;
                    }
                } else {
                    // player is smackin their head on the obstacle
                    controller.vertical_velocity -= GRAVITY * time.delta_seconds();
                }
                controller.jumping = true;
            }
        } else {
            // nothing to run into
            player_transform.translation.y += controller.vertical_velocity * time.delta_seconds();
            controller.vertical_velocity -= GRAVITY * time.delta_seconds();
        }
    }
}

fn check_spike_collisions(
    player_query: Query<(&Transform, &Player), Without<Spikes>>,
    spikes_query: Query<(&Transform, &RectCollider), With<Spikes>>,
    paused: Res<Paused>,
    dead: Res<Dead>,
    mut commands: Commands,
) {
    if paused.0 || dead.0 {
        return;
    }

    for (player_transform, player) in &player_query {
        let player_left_edge =
            player_transform.translation.x + player.collider_offset.x - (player.collider.x / 2.0);
        let player_right_edge =
            player_transform.translation.x + player.collider_offset.x + (player.collider.x / 2.0);
        let player_top =
            player_transform.translation.y + player.collider_offset.y + (player.collider.y / 2.0);
        let player_bottom =
            player_transform.translation.y + player.collider_offset.y - (player.collider.y / 2.0);

        for (spikes_transform, spikes_collider) in &spikes_query {
            let spikes_left_edge = spikes_transform.translation.x + spikes_collider.offset.x
                - (spikes_collider.bounds.x / 2.0);
            let spikes_right_edge = spikes_transform.translation.x
                + spikes_collider.offset.x
                + (spikes_collider.bounds.x / 2.0);
            let spikes_top = spikes_transform.translation.y
                + spikes_collider.offset.y
                + (spikes_collider.bounds.y / 2.0);
            let spikes_bottom = spikes_transform.translation.y + spikes_collider.offset.y
                - (spikes_collider.bounds.y / 2.0);

            if ((spikes_left_edge - player_right_edge).abs() <= f32::EPSILON)
                && !(player_bottom > spikes_top || player_top < spikes_bottom)
            {
                // player is touching left side of spikes
                commands.trigger(DeathEvent);
            }

            if (((player_bottom - spikes_top).abs() <= f32::EPSILON)
                || (spikes_bottom - player_top).abs() <= f32::EPSILON)
                && !(player_left_edge > spikes_right_edge || player_right_edge < spikes_left_edge)
            {
                // player is touching top or bottom of spikes
                commands.trigger(DeathEvent);
            }
        }
    }
}

fn wrap_within_level(
    mut wrap_query: Query<&mut Transform, With<Player>>,
    mut current_level: ResMut<CurrentLevel>,
    mut commands: Commands,
) {
    for mut transform in &mut wrap_query {
        let player_left_edge = transform.translation.x - (PLAYER_IMAGE_SIZE / 2.0);
        let level_right_edge = LEVEL_WIDTH / 2.0;
        if player_left_edge > level_right_edge {
            // player has fully left the level, move them back to the left side
            let level_left_edge = -LEVEL_WIDTH / 2.0;
            transform.translation.x = level_left_edge - (PLAYER_IMAGE_SIZE / 2.0);
            // clear the current level and load the next one
            current_level.0 += 1;
            commands.trigger(SpawnObstacles(current_level.0));
        }
    }
}
