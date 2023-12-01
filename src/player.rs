use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::f32::consts::PI;

use crate::{
    game::{init_game, Game},
    platforms::{Hovered, Platform, Touched},
};

const SPAWN_POINT: Vec3 = Vec3::new(-3.0, 5.0, 0.0);

const GRAVITY: f32 = -9.81;

const JUMP: f32 = 5.5;
const JUMP_BOOST_DURATION: f32 = 0.6;
const JUMP_BOOST_MIN_TIME: f32 = 0.15;
const JUMP_BOOST_SPEED: f32 = 10.0;

const SPEED: f32 = 5.0;
const SPEED_MIN: f32 = 0.2;

const CAMERA_ROTATION_LERP: f32 = 0.97;
const DIRECTION_LERP: f32 = 0.9;

#[derive(Component)]
pub struct Player {
    jump_pressed: bool,
    jump_boost_duration: Timer,
    velocity_y: f32,
    last_direction_2d: Vec2,
}

pub fn spawn_player(mut commands: Commands) {
    commands
        .spawn((
            Player {
                jump_pressed: false,
                jump_boost_duration: Timer::from_seconds(
                    JUMP_BOOST_DURATION + JUMP_BOOST_MIN_TIME,
                    TimerMode::Once,
                ),
                velocity_y: 0.0,
                last_direction_2d: Vec2::ZERO,
            },
            RigidBody::KinematicPositionBased,
            Collider::ball(1.0),
            KinematicCharacterController {
                max_slope_climb_angle: PI, // climb anything
                ..default()
            },
            TransformBundle::from_transform(Transform::from_translation(SPAWN_POINT)),
            Ccd { enabled: true },
        ))
        .with_children(|c| {
            c.spawn((Camera3dBundle {
                projection: Projection::Perspective(PerspectiveProjection {
                    fov: PI / 2.0,
                    ..default()
                }),
                transform: Transform::from_translation(Vec3::Y).looking_at(Vec3::X, Vec3::Y),
                ..default()
            },));
        });
}

pub fn player_movement(
    time: Res<Time>,
    keyboard: Res<Input<KeyCode>>,
    touches: Res<Touches>,
    mut game: ResMut<Game>,
    mut player_query: Query<(
        &Transform,
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
        &mut Player,
    )>,
    platforms_untouched: Query<&Transform, (With<Platform>, Without<Touched>)>,
) {
    let (player_transform, mut controller, controller_output, mut player) =
        player_query.single_mut();

    let jump_just_pressed = keyboard.just_pressed(KeyCode::Space)
        || touches.iter_just_pressed().any(|touch| touch.id() == 0);

    let jump_just_released = keyboard.just_released(KeyCode::Space)
        || touches.iter_just_released().any(|touch| touch.id() == 0);

    if !player.jump_pressed && jump_just_pressed {
        player.jump_pressed = true;
    } else if player.jump_pressed && jump_just_released {
        player.jump_pressed = false;
    }

    // Start on first jump
    if !game.started {
        if player.jump_pressed {
            game.started = true;
        } else {
            return;
        }
    }

    // Jump & Gravity
    // FIXME: not grounded on steep descending slopes
    let is_grounded = controller_output.map(|o| o.grounded).unwrap_or(false);

    if is_grounded {
        player.velocity_y = 0.0;

        if player.jump_pressed {
            player.velocity_y += JUMP;
            player.jump_boost_duration.reset();
        }
    } else {
        player.velocity_y += GRAVITY * time.delta_seconds();
    }

    player.jump_boost_duration.tick(time.delta());

    if player.jump_pressed
        && !player.jump_boost_duration.finished()
        && player.jump_boost_duration.elapsed_secs() > JUMP_BOOST_MIN_TIME
    {
        player.velocity_y += JUMP_BOOST_SPEED * time.delta_seconds();
    }

    // MOVEMENT: move towards the closest platform without Touched
    let mut next_untouched = None;
    let mut min_distance = f32::MAX;

    for platform_transform in platforms_untouched.iter() {
        let distance = platform_transform
            .translation
            .distance(player_transform.translation);

        if distance < min_distance {
            next_untouched = Some(platform_transform.translation);
            min_distance = distance;
        }
    }

    let next_untouched_position = next_untouched.unwrap_or(game.next_platform_position);

    // get the 2d direction towards the platform, normalized
    let mut direction_2d =
        (next_untouched_position.xz() - player_transform.translation.xz()).normalize();
    direction_2d.x = direction_2d.x.max(SPEED_MIN); // cap the minimum speed
    direction_2d = direction_2d.lerp(player.last_direction_2d, DIRECTION_LERP); // smooth the direction change

    player.last_direction_2d = direction_2d;

    let forward_speed = SPEED * (game.difficulty() + 1.0);
    let movement_2d = direction_2d * forward_speed;

    let movement = Vec3::new(movement_2d.x, player.velocity_y, movement_2d.y);

    controller.translation = Some(movement * time.delta_seconds());
}

pub fn camera_rotation(
    game: Res<Game>,
    mut camera: Query<(&mut Transform, &GlobalTransform), With<Camera3d>>,
    platforms_unhovered: Query<&Transform, (With<Platform>, Without<Hovered>, Without<Camera3d>)>,
) {
    let (mut camera_transform, camera_global_transform) = camera.single_mut();

    // CAMERA: look at the closest platform without Hovered
    let mut next_unhovered = None;
    let mut min_distance = f32::MAX;

    for platform_transform in platforms_unhovered.iter() {
        let distance = platform_transform
            .translation
            .distance(camera_global_transform.translation());

        if distance < min_distance {
            next_unhovered = Some(platform_transform.translation);
            min_distance = distance;
        }
    }

    let next_unhovered_position = next_unhovered.unwrap_or(game.next_platform_position);

    // Rotate the camera towards the platform
    let direction = next_unhovered_position - camera_global_transform.translation() + Vec3::Y * 2.0;
    // code from Transform::look_at
    let back = -direction.try_normalize().unwrap_or(Vec3::NEG_Z);
    let right = Vec3::Y
        .cross(back)
        .try_normalize()
        .unwrap_or_else(|| Vec3::Z);
    let up = back.cross(right);
    let rotation = Quat::from_mat3(&Mat3::from_cols(right, up, back));

    // Spherical interpolation
    camera_transform.rotation = rotation.slerp(camera_transform.rotation, CAMERA_ROTATION_LERP);
}

pub fn player_touch_platform(
    mut commands: Commands,
    mut game: ResMut<Game>,
    rapier_context: Res<RapierContext>,
    player: Query<(Entity, &Transform), With<Player>>,
    platforms: Query<Entity, (With<Platform>, Without<Touched>)>,
) {
    let (player_entity, player_transform) = player.single();

    let filter = QueryFilter::default().exclude_rigid_body(player_entity);
    let pos = player_transform.translation;

    if let Some((entity, _)) = rapier_context.cast_ray(pos, -Vec3::Y, 2.0, true, filter) {
        if platforms.contains(entity) {
            commands.entity(entity).insert(Touched);
            game.points += 1;
            commands.run_system(game.update_hud_system);
            commands.run_system(game.spawn_platform_system);
        }
    }
}

pub fn player_hover_platform(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    player: Query<(Entity, &Transform), With<Player>>,
    platforms: Query<Entity, (With<Platform>, Without<Hovered>)>,
) {
    let (player_entity, player_transform) = player.single();

    let filter = QueryFilter::default().exclude_rigid_body(player_entity);
    let pos = player_transform.translation;

    if let Some((entity, _)) = rapier_context.cast_ray(pos, -Vec3::Y, 100.0, true, filter) {
        if platforms.contains(entity) {
            commands.entity(entity).insert(Hovered);
        }
    }
}

pub fn respawn(
    game: ResMut<Game>,
    mut commands: Commands,
    mut player: Query<(&mut Player, &mut Transform), With<Player>>,
    mut camera: Query<&mut Transform, (With<Camera3d>, Without<Player>)>,
    platforms: Query<Entity, With<Platform>>,
) {
    let (mut player, mut transform) = player.single_mut();
    if player.velocity_y < -20.0 {
        for entity in platforms.iter() {
            commands.entity(entity).despawn_recursive();
        }

        let mut camera_transform = camera.single_mut();
        camera_transform.look_at(Vec3::X, Vec3::Y);

        player.velocity_y = 0.0;
        transform.translation = SPAWN_POINT;

        init_game(commands, game);
    }
}

pub fn force_respawn(
    keyboard_input: Res<Input<KeyCode>>,
    mut player: Query<(&mut Player, &mut Transform)>,
) {
    if keyboard_input.just_pressed(KeyCode::R) {
        let (mut player, mut transform) = player.single_mut();
        transform.translation.y = -100.0;
        player.velocity_y = -100.0;
    }
}
