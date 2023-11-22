use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use bevy_rapier3d::prelude::*;
use std::f32::consts::TAU;

use crate::terrain::{Platform, Touched};

const GRAVITY: f32 = -4.0;
const JUMP: f32 = 10.0;
const JUMP_TIME: f32 = 0.4;
const SPEED: f32 = 3.0;
const SPEED_JUMPING: f32 = 6.0;

#[derive(Component)]
pub struct Player {
    jump_timer: Timer,
}

const SPAWN_POINT: Vec3 = Vec3::new(0.0, 3.0, 0.0);

pub fn spawn_player(mut commands: Commands) {
    commands
        .spawn((
            Player {
                jump_timer: Timer::from_seconds(JUMP_TIME, TimerMode::Once),
            },
            RigidBody::KinematicPositionBased,
            Collider::ball(0.5),
            KinematicCharacterController::default(),
            TransformBundle::from_transform(Transform::from_translation(SPAWN_POINT)),
            Ccd { enabled: true },
        ))
        .with_children(|c| {
            c.spawn(Camera3dBundle {
                projection: Projection::Perspective(PerspectiveProjection {
                    fov: TAU / 5.0,
                    ..default()
                }),
                //transform: Transform::default().looking_at(Vec3::new(1.0, -0.3, 0.0), Vec3::Y),
                ..default()
            });
        });
}

pub fn player_movement(
    time: Res<Time>,
    mut key_event: EventReader<KeyboardInput>,
    mut player_query: Query<(
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
        &mut Player,
    )>,
    mut camera: Query<(&mut Transform, &GlobalTransform), With<Camera3d>>,
    platforms: Query<&Transform, (With<Platform>, Without<Touched>, Without<Camera3d>)>,
) {
    let (mut controller, controller_output, mut player) = player_query.single_mut();
    let (mut camera_transform, camera_global_transform) = camera.single_mut();

    let mut movement = Vec3::new(SPEED, GRAVITY, 0.0);

    // CAMERA: look at the closest platform (without Touched)
    let mut closest_platform = None;
    let mut closest_distance = f32::MAX;

    for platform_transform in platforms.iter() {
        let distance = platform_transform
            .translation
            .distance(camera_global_transform.translation());

        if distance < closest_distance {
            closest_platform = Some(platform_transform);
            closest_distance = distance;
        }
    }

    if closest_platform.is_none() {
        return;
    }
    let closest_platform_transform = closest_platform.unwrap();

    // Rotate the camera towards the platform
    let direction = closest_platform_transform.translation - camera_global_transform.translation()
        + Vec3::Y * 2.0;
    let up = Vec3::Y;

    // code from Transform::look_at
    let back = -direction.try_normalize().unwrap_or(Vec3::NEG_Z);
    let up = up.try_normalize().unwrap_or(Vec3::Y);
    let right = up
        .cross(back)
        .try_normalize()
        .unwrap_or_else(|| up.any_orthonormal_vector());
    let up = back.cross(right);
    let rotation = Quat::from_mat3(&Mat3::from_cols(right, up, back));

    // Spherical interpolation
    camera_transform.rotation = camera_transform.rotation.slerp(rotation, 0.05);

    if key_event
        .read()
        .any(|e| e.state == ButtonState::Pressed && e.key_code == Some(KeyCode::Space))
        && controller_output.map(|o| o.grounded).unwrap_or(false)
    {
        player.jump_timer.reset();
    }

    if !player.jump_timer.tick(time.delta()).finished() {
        movement.y = JUMP * (1.0 - player.jump_timer.percent());
        movement.x = SPEED_JUMPING;
    }

    let rotation_y = -rotation.to_euler(EulerRot::XZY).0;
    movement = Quat::from_rotation_y(rotation_y) * movement;

    println!("movement: {:?}", movement);

    controller.translation = Some(movement * time.delta_seconds());
}

pub fn player_touch_platform(
    mut commands: Commands,
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
        }
    }
}

pub fn respawn(
    mut commands: Commands,
    mut player: Query<(&mut Player, &mut Transform), With<Player>>,
    platforms_touched: Query<Entity, With<Touched>>,
) {
    let (mut player, mut transform) = player.single_mut();
    if transform.translation.y < -10.0 {
        player.jump_timer.reset();
        transform.translation = SPAWN_POINT;

        for entity in platforms_touched.iter() {
            commands.entity(entity).remove::<Touched>();
        }
    }
}
