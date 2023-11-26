use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::game::Game;

const PLATFORM_SIZE: f32 = 2.0;

const MOVING_PLATFORM_CHANCE: f64 = 0.1;

#[derive(Component)]
pub struct Platform;

#[derive(Component)]
pub struct MovingPlatform {
    pub progress: f32,
    pub going_negative: bool,
    pub z: f32,
}

#[derive(Component)]
pub struct Touched;

#[derive(Component)]
pub struct Hovered;

pub fn spawn_platform(
    mut game: ResMut<Game>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = meshes.add(Mesh::from(shape::Cube {
        size: PLATFORM_SIZE,
    }));

    let mut rng = rand::thread_rng();

    let random_color = Color::rgb(rng.gen(), rng.gen(), rng.gen());

    let mut c = commands.spawn((
        Platform,
        Collider::cuboid(
            PLATFORM_SIZE / 2.0,
            PLATFORM_SIZE / 2.0,
            PLATFORM_SIZE / 2.0,
        ),
        RigidBody::Fixed,
        PbrBundle {
            mesh: cube.clone(),
            material: materials.add(random_color.into()),
            transform: Transform::from_translation(game.next_platform_position),
            ..default()
        },
    ));

    //moving platform
    if rng.gen_bool(MOVING_PLATFORM_CHANCE) {
        c.insert(MovingPlatform {
            progress: rng.gen_range(-1.0..1.0),
            going_negative: rng.gen_bool(0.5),
            z: game.next_platform_position.z,
        });
    }

    let mut platform_spacing = 5.5;

    let y = if rng.gen_bool(0.5) {
        rng.gen_range(-5.0..0.0)
    } else {
        rng.gen_range(0.0..2.0)
    };

    if game.next_platform_position.y > y + 2.0 {
        platform_spacing = rng.gen_range(6.0..10.0)
    }

    game.next_platform_position += Vec3::new(rng.gen_range(4.0..8.0), y, rng.gen_range(-5.0..5.0))
        .normalize()
        * platform_spacing;
}

pub fn update_moving_platforms(
    time: Res<Time>,
    mut platforms_transforms: Query<(&mut Transform, &mut MovingPlatform)>,
) {
    for (mut transform, mut moving_platform) in platforms_transforms.iter_mut() {
        if moving_platform.going_negative {
            moving_platform.progress -= time.delta_seconds();
        } else {
            moving_platform.progress += time.delta_seconds();
        }

        if moving_platform.progress > 1.0 {
            moving_platform.progress = 1.0;
            moving_platform.going_negative = true;
        } else if moving_platform.progress < -1.0 {
            moving_platform.progress = -1.0;
            moving_platform.going_negative = false;
        }

        transform.translation.z = moving_platform.progress * 2.0 + moving_platform.z;
    }
}
