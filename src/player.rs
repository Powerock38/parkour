use bevy::{
    core_pipeline::Skybox,
    input::{keyboard::KeyboardInput, touch::TouchPhase, ButtonState},
    prelude::*,
};
use bevy_rapier3d::prelude::*;
use std::f32::consts::PI;

use crate::{
    game::{init_game, Game},
    platforms::{Hovered, Platform, Touched},
    utils::lerp,
};

const SPAWN_POINT: Vec3 = Vec3::new(-3.0, 5.0, 0.0);

const GRAVITY: f32 = -6.0;
const JUMP: f32 = 10.0;
const JUMP_MIN: f32 = 0.1;
const JUMP_TIME: f32 = 0.4;
const SPEED: f32 = 3.0;
const SPEED_JUMPING: f32 = 8.0;
const SPEED_MIN: f32 = 0.2;

const CAMERA_ROTATION_LERP: f32 = 0.97;
const DIRECTION_LERP: f32 = 0.9;

#[derive(Component)]
pub struct Player {
    jump_timer: Timer,
    last_direction_2d: Vec2,
}

pub fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Player {
                jump_timer: Timer::from_seconds(JUMP_TIME, TimerMode::Once),
                last_direction_2d: Vec2::ZERO,
            },
            RigidBody::KinematicPositionBased,
            Collider::ball(1.0),
            KinematicCharacterController::default(),
            TransformBundle::from_transform(Transform::from_translation(SPAWN_POINT)),
            Ccd { enabled: true },
        ))
        .with_children(|c| {
            c.spawn((
                Camera3dBundle {
                    projection: Projection::Perspective(PerspectiveProjection {
                        fov: PI / 2.0,
                        ..default()
                    }),
                    transform: Transform::from_translation(Vec3::Y).looking_at(Vec3::X, Vec3::Y),
                    ..default()
                },
                // https://jaxry.github.io/panorama-to-cubemap/
                // https://www.imgonline.com.ua/eng/cut-photo-into-pieces.php
                // toktx --cubemap --t2 sky.ktx2 right.jpg left.jpg top.jpg bottom.jpg front.jpg back.jpg
                // toktx --cubemap --t2 sky.ktx2 px.png nx.png py.png ny.png pz.png nz.png
                Skybox(asset_server.load("cloud1.ktx2")),
            ));
        });
}

pub fn player_movement(
    time: Res<Time>,
    mut key_event: EventReader<KeyboardInput>,
    mut touch_event: EventReader<TouchInput>,
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

    // JUMP
    let input_jump = key_event
        .read()
        .any(|e| e.state == ButtonState::Pressed && e.key_code == Some(KeyCode::Space))
        || touch_event.read().any(|e| e.phase == TouchPhase::Started);

    if !game.started {
        if input_jump {
            game.started = true;
        } else {
            return;
        }
    }

    if input_jump && controller_output.map(|o| o.grounded).unwrap_or(false) {
        player.jump_timer.reset();
    }

    let mut forward_speed = SPEED * (game.difficulty() + 1.0);
    let mut vertical_speed = GRAVITY;

    if !player.jump_timer.tick(time.delta()).finished() {
        vertical_speed = lerp(JUMP, JUMP_MIN, player.jump_timer.percent());
        forward_speed = SPEED_JUMPING;
    }

    // MOVEMENT: move towards the closest platform without Touched
    let mut closest_untouched_platform = None;
    let mut closest_distance = f32::MAX;

    for platform_transform in platforms_untouched.iter() {
        let distance = platform_transform
            .translation
            .distance(player_transform.translation);

        if distance < closest_distance {
            closest_untouched_platform = Some(platform_transform);
            closest_distance = distance;
        }
    }

    let closest_untouched_platform_transform =
        if let Some(closest_untouched_platform) = closest_untouched_platform {
            closest_untouched_platform
        } else {
            return;
        };

    // get the 2d direction towards the platform, normalized
    let mut direction_2d = (closest_untouched_platform_transform.translation.xz()
        - player_transform.translation.xz())
    .normalize();
    direction_2d.x = direction_2d.x.max(SPEED_MIN); // cap the minimum speed
    direction_2d = direction_2d.lerp(player.last_direction_2d, DIRECTION_LERP); // smooth the direction change

    player.last_direction_2d = direction_2d;

    let movement_2d = direction_2d * forward_speed;
    let movement = Vec3::new(movement_2d.x, vertical_speed, movement_2d.y);

    controller.translation = Some(movement * time.delta_seconds());
}

pub fn camera_rotation(
    mut camera: Query<(&mut Transform, &GlobalTransform), With<Camera3d>>,
    platforms_unhovered: Query<&Transform, (With<Platform>, Without<Hovered>, Without<Camera3d>)>,
) {
    let (mut camera_transform, camera_global_transform) = camera.single_mut();

    // CAMERA: look at the closest platform without Hovered
    let mut closest_unhovered_platform = None;
    let mut closest_distance = f32::MAX;

    for platform_transform in platforms_unhovered.iter() {
        let distance = platform_transform
            .translation
            .distance(camera_global_transform.translation());

        if distance < closest_distance {
            closest_unhovered_platform = Some(platform_transform);
            closest_distance = distance;
        }
    }

    let closest_unhovered_platform_transform =
        if let Some(closest_unhovered_platform) = closest_unhovered_platform {
            closest_unhovered_platform
        } else {
            return;
        };

    // Rotate the camera towards the platform
    let direction = closest_unhovered_platform_transform.translation
        - camera_global_transform.translation()
        + Vec3::Y * 2.0;
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
            commands.run_system(game.generate_platform);
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
    mut game: ResMut<Game>,
    mut commands: Commands,
    mut player: Query<(&mut Player, &mut Transform), With<Player>>,
    mut camera: Query<&mut Transform, (With<Camera3d>, Without<Player>)>,
    platforms: Query<Entity, With<Platform>>,
) {
    let (mut player, mut transform) = player.single_mut();
    if transform.translation.y < game.next_platform_position.y - 20.0 {
        player.jump_timer.reset();
        transform.translation = SPAWN_POINT;

        for entity in platforms.iter() {
            commands.entity(entity).despawn_recursive();
        }

        game.points = 0;
        game.next_platform_position = Vec3::ZERO;

        let mut camera_transform = camera.single_mut();
        camera_transform.look_at(Vec3::X, Vec3::Y);

        init_game(commands, game);
    }
}

pub fn force_respawn(
    keyboard_input: Res<Input<KeyCode>>,
    mut player: Query<&mut Transform, With<Player>>,
) {
    if keyboard_input.just_pressed(KeyCode::R) {
        let mut transform = player.single_mut();
        transform.translation.y -= 1000.0;
    }
}
