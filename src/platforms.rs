use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::game::Game;

const PLATFORM_SIZE: f32 = 3.0;
const PLATFORM_SPACING: f32 = 5.5;

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

    let size = PLATFORM_SIZE * (1.0 - game.difficulty()) * rng.gen_range(0.8..1.2);

    let mesh = meshes.add(Mesh::from(shape::Cube { size }));

    let random_color = Color::rgb(rng.gen(), rng.gen(), rng.gen());

    let position = game.next_platform_position;

    let mut next_platform_spacing = PLATFORM_SPACING * (game.difficulty() + 1.0);

    let next_platform_y = if rng.gen_bool(0.5) {
        rng.gen_range(-5.0..0.0)
    } else {
        rng.gen_range(0.0..2.0)
    };

    // bigger gap if we are going down
    if position.y > next_platform_y + 4.0 {
        next_platform_spacing = rng.gen_range(PLATFORM_SPACING..PLATFORM_SPACING * 2.0)
    }

    game.next_platform_position += Vec3::new(
        rng.gen_range(4.0..8.0),
        next_platform_y,
        rng.gen_range(-5.0..5.0),
    )
    .normalize()
        * next_platform_spacing;

    let mut c = commands.spawn((
        PbrBundle {
            mesh,
            material: materials.add(random_color.into()),
            transform: Transform::from_translation(position)
                .looking_at(game.next_platform_position, Vec3::Y),
            ..default()
        },
        Platform,
        Collider::cuboid(size / 2.0, size / 2.0, size / 2.0),
        RigidBody::Fixed,
    ));

    //moving platform
    let moving_platform_chance = game.difficulty().min(0.5);
    if rng.gen_bool(moving_platform_chance as f64) {
        c.insert(MovingPlatform {
            progress: rng.gen_range(-1.0..1.0),
            going_negative: rng.gen_bool(0.5),
            z: position.z,
        });
    }
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
