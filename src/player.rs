use avian3d::prelude::*;
use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};
use std::f32::consts::PI;

use crate::{
    game::Game,
    platforms::{Hovered, Platform, Touched, TOUCHED_PLATFORM_TTL},
    skybox::{generate_skybox_mesh, SkyboxCustom},
    PlatformGeneration, SpawnPlatform,
};

pub const SPAWN_POINT: Vec3 = Vec3::new(-5.0, 5.0, 0.0);

const GRAVITY: f32 = -9.81;

const COYOTE_TIME: f32 = 0.2;

const JUMP: f32 = 5.5;
const JUMP_BOOST_DURATION: f32 = 0.6;
const JUMP_BOOST_MIN_TIME: f32 = 0.15;
const JUMP_BOOST_SPEED: f32 = 10.0;

const SPEED: f32 = 5.0;
const DIRECTION_LERP: f32 = 0.9;

const CAMERA_ROTATION_SPEED: f32 = 2.0;

#[derive(Component)]
pub struct Player {
    coyote_time: Timer,
    jump_pressed: bool,
    jump_boost_duration: Timer,
    last_direction_2d: Vec2,
}

pub fn spawn_player(mut commands: Commands, mut mesh_assets: ResMut<Assets<Mesh>>) {
    commands
        .spawn((
            Player {
                coyote_time: Timer::from_seconds(COYOTE_TIME, TimerMode::Once),
                jump_pressed: false,
                jump_boost_duration: Timer::from_seconds(
                    JUMP_BOOST_DURATION + JUMP_BOOST_MIN_TIME,
                    TimerMode::Once,
                ),
                last_direction_2d: Vec2::ZERO,
            },
            RayCaster::new(Vec3::ZERO, -Dir3::Y).with_max_hits(1),
            RigidBody::Kinematic,
            Collider::sphere(1.0),
            Transform::from_translation(SPAWN_POINT),
            Visibility::default(),
        ))
        .with_children(|c| {
            c.spawn((
                Camera3d::default(),
                Projection::Perspective(PerspectiveProjection {
                    fov: PI / 2.0,
                    ..default()
                }),
                Transform::from_translation(Vec3::Y).looking_at(Vec3::X, Vec3::Y),
            ));

            c.spawn((
                SkyboxCustom,
                Mesh3d(mesh_assets.add(generate_skybox_mesh())),
                NotShadowCaster,
                NotShadowReceiver,
            ));
        });
}

pub fn player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut game: ResMut<Game>,
    platform_gen: Res<PlatformGeneration>,
    q_player: Single<(&Transform, &mut LinearVelocity, &RayHits, &mut Player)>,
    q_children: Query<&Children>,
    q_platforms_untouched: Query<(Entity, &Transform), (With<Platform>, Without<Touched>)>,
    q_platforms_touched: Query<Entity, (With<Platform>, With<Touched>)>,
) {
    let (player_transform, mut velocity, ray_hits, mut player) = q_player.into_inner();

    let jump_just_pressed = keyboard.just_pressed(KeyCode::Space)
        || mouse.just_pressed(MouseButton::Left)
        || touches.iter_just_pressed().any(|touch| touch.id() == 0);

    let jump_just_released = keyboard.just_released(KeyCode::Space)
        || mouse.just_released(MouseButton::Left)
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
    let mut velocity_y = velocity.y;

    let is_grounded = ray_hits
        .iter()
        .filter(|hit| hit.time_of_impact < 1.0)
        .any(|hit| {
            q_platforms_touched
                .iter()
                .chain(q_platforms_untouched.iter().map(|p| p.0))
                .any(|p| p == hit.entity || q_children.iter_descendants(p).any(|c| c == hit.entity))
        });

    if is_grounded {
        player.coyote_time.reset();
    }

    let is_grounded_coyote = is_grounded || !player.coyote_time.tick(time.delta()).finished();

    if is_grounded_coyote {
        velocity_y = 0.0;

        if player.jump_pressed {
            velocity_y += JUMP;
            player.jump_boost_duration.reset();
        }
    } else {
        velocity_y += GRAVITY * time.delta_secs();
    }

    player.jump_boost_duration.tick(time.delta());

    if player.jump_pressed
        && !player.jump_boost_duration.finished()
        && player.jump_boost_duration.elapsed_secs() > JUMP_BOOST_MIN_TIME
    {
        velocity_y += JUMP_BOOST_SPEED * time.delta_secs();
    }

    // MOVEMENT: move towards the closest platform without Touched
    let mut next_untouched = None;
    let mut min_distance = f32::MAX;

    for (_, platform_transform) in &q_platforms_untouched {
        let distance = platform_transform
            .translation
            .distance(player_transform.translation);

        if distance < min_distance {
            next_untouched = Some(platform_transform.translation);
            min_distance = distance;
        }
    }

    let next_untouched_position = next_untouched.unwrap_or(platform_gen.next_platform_position);

    // get the 2d direction towards the platform, normalized
    let direction_2d = (next_untouched_position.xz() - player_transform.translation.xz())
        .normalize()
        .lerp(player.last_direction_2d, DIRECTION_LERP); // smooth the direction change

    player.last_direction_2d = direction_2d;

    let forward_speed = SPEED * (game.difficulty() + 1.0);
    let movement_2d = direction_2d * forward_speed;

    velocity.0 = Vec3::new(movement_2d.x, velocity_y, movement_2d.y);
}

pub fn camera_rotation(
    time: Res<Time>,
    platform_gen: Res<PlatformGeneration>,
    mut camera: Query<(&mut Transform, &GlobalTransform), With<Camera3d>>,
    platforms_unhovered: Query<&Transform, (With<Platform>, Without<Hovered>, Without<Camera3d>)>,
) {
    let (mut camera_transform, camera_global_transform) = camera.single_mut();

    // CAMERA: look at the closest platform without Hovered
    let mut next_unhovered = None;
    let mut min_distance = f32::MAX;

    for platform_transform in &platforms_unhovered {
        let distance = platform_transform
            .translation
            .distance(camera_global_transform.translation());

        if distance < min_distance {
            next_unhovered = Some(platform_transform.translation);
            min_distance = distance;
        }
    }

    let next_unhovered_position = next_unhovered.unwrap_or(platform_gen.next_platform_position);

    // Rotate the camera towards the platform
    let direction = next_unhovered_position - camera_global_transform.translation() + Vec3::Y * 2.0;
    // code from Transform::look_at
    let back = -direction.try_normalize().unwrap_or(Vec3::NEG_Z);
    let right = Vec3::Y.cross(back).try_normalize().unwrap_or(Vec3::Z);
    let up = back.cross(right);
    let rotation = Quat::from_mat3(&Mat3::from_cols(right, up, back));

    // frame-independent lerp
    camera_transform.rotation = camera_transform
        .rotation
        .lerp(rotation, CAMERA_ROTATION_SPEED * time.delta_secs());
}

pub fn player_touch_platform(
    mut commands: Commands,
    mut game: ResMut<Game>,
    collisions: Res<Collisions>,
    player: Single<Entity, With<Player>>,
    q_platforms_untouched: Query<Entity, (With<Platform>, Without<Touched>)>,
    q_children: Query<&Children>,
) {
    let player_entity = *player;

    if let Some(entity) = q_platforms_untouched.iter().find(|platform| {
        collisions.contains(player_entity, *platform)
            || q_children
                .iter_descendants(*platform)
                .any(|c| collisions.contains(c, player_entity))
    }) {
        commands.entity(entity).insert(Touched(Timer::from_seconds(
            TOUCHED_PLATFORM_TTL,
            TimerMode::Once,
        )));
        game.points += 1;
        commands.trigger(SpawnPlatform);
    }
}

pub fn player_hover_platform(
    mut commands: Commands,
    q_player: Single<&RayHits, With<Player>>,
    q_platforms_unhovered: Query<Entity, (With<Platform>, Without<Hovered>)>,
    q_children: Query<&Children>,
) {
    let ray_hits = q_player.into_inner();

    if let Some(platform) = q_platforms_unhovered.iter().find(|platform| {
        ray_hits.iter().any(|hit| {
            *platform == hit.entity
                || q_children
                    .iter_descendants(*platform)
                    .any(|c| c == hit.entity)
        })
    }) {
        commands.entity(platform).try_insert(Hovered);
    }
}

pub fn force_respawn(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q_player: Single<(&mut Transform, &mut LinearVelocity), With<Player>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        let (mut transform, mut velocity) = q_player.into_inner();
        transform.translation.y = -100.0;
        velocity.y = -100.0;
    }
}
