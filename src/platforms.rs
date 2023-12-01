use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::{seq::SliceRandom, Rng};

use crate::game::{Game, SKYBOXES, SKYBOX_CHANGE_CHANCE};

const PLATFORM_SIZE: f32 = 2.0;
const PLATFORM_SPACING_MIN: f32 = 5.0;
const PLATFORM_SPACING_MAX: f32 = 9.0;

const DIRECTION_BIAS_HORIZONTAL_CHANCE: f64 = 0.1;
const DIRECTION_BIAS_VERTICAL_CHANCE: f64 = 0.1;

const VERTICAL_VARIATION_UP: f32 = 3.0;
const VERTICAL_VARIATION_DOWN: f32 = 8.0;
const HORIZONTAL_VARIATION: f32 = 8.0;

const MOVING_PLATFORM_CHANCE_MIN: f64 = 0.1;
const MOVING_PLATFORM_CHANCE_MAX: f64 = 0.5;

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
    let mut rng = rand::thread_rng();

    // Platform mesh
    let size = PLATFORM_SIZE * (1.0 - game.difficulty()) * rng.gen_range(0.8..1.2);
    let mesh = meshes.add(Mesh::from(shape::Cube { size }));

    // Small chance to update current skybox
    if rng.gen_bool(SKYBOX_CHANGE_CHANCE) {
        game.skybox = SKYBOXES.choose(&mut rng).unwrap();
        println!("skybox: {:?}", game.skybox);
        commands.run_system(game.change_skybox_system);
    }

    // Small chance to update direction bias
    if rng.gen_bool(DIRECTION_BIAS_HORIZONTAL_CHANCE) {
        game.direction_bias_horizontal = rng.gen_range(0.0..1.0);
    }

    if rng.gen_bool(DIRECTION_BIAS_VERTICAL_CHANCE) {
        game.direction_bias_vertical = rng.gen_range(0.0..1.0);
    }

    // Position
    let next_platform_z = if rng.gen_bool(game.direction_bias_horizontal) {
        rng.gen_range(-HORIZONTAL_VARIATION..0.0)
    } else {
        rng.gen_range(0.0..HORIZONTAL_VARIATION)
    };

    let next_platform_y = if rng.gen_bool(game.direction_bias_vertical) {
        rng.gen_range(-VERTICAL_VARIATION_DOWN..0.0)
    } else {
        rng.gen_range(0.0..VERTICAL_VARIATION_UP)
    };

    let position = game.next_platform_position;
    let mut next_platform_spacing = rng.gen_range(PLATFORM_SPACING_MIN..PLATFORM_SPACING_MAX);

    // bigger gap if we are going down
    if position.y > next_platform_y + 4.0 {
        next_platform_spacing *= 1.0 + rng.gen_range(0.0..0.2);
    }

    // Set next platform position
    game.next_platform_position +=
        Vec3::new(rng.gen_range(4.0..8.0), next_platform_y, next_platform_z).normalize()
            * next_platform_spacing;

    // Spawn platform
    let mut c = commands.spawn((
        PbrBundle {
            mesh,
            material: materials.add(Color::rgb(rng.gen(), rng.gen(), rng.gen()).into()),
            transform: Transform::from_translation(position)
                .looking_at(game.next_platform_position, Vec3::Y),
            ..default()
        },
        Platform,
        Collider::cuboid(size / 2.0, size / 2.0, size / 2.0),
        RigidBody::Fixed,
    ));

    // Chance to be a moving platform
    let moving_platform_chance =
        (game.difficulty() as f64 + MOVING_PLATFORM_CHANCE_MIN).min(MOVING_PLATFORM_CHANCE_MAX);

    if rng.gen_bool(moving_platform_chance) {
        c.insert(MovingPlatform {
            progress: rng.gen_range(-1.0..1.0),
            going_negative: rng.gen_bool(0.5),
            z: position.z,
        });
    }
}

pub fn update_moving_platforms(
    time: Res<Time>,
    game: Res<Game>,
    mut platforms_transforms: Query<(&mut Transform, &mut MovingPlatform)>,
) {
    let speed = (1.0 + game.difficulty() * 2.0).min(5.0);

    for (mut transform, mut moving_platform) in platforms_transforms.iter_mut() {
        if moving_platform.going_negative {
            moving_platform.progress -= speed * time.delta_seconds();
        } else {
            moving_platform.progress += speed * time.delta_seconds();
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
