use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;

const PLATFORM_SIZE: f32 = 2.0;

const LARGE_PLATFORM_SIZE: f32 = 10.0;

#[derive(Component)]
pub struct Platform;

#[derive(Component)]
pub struct Touched;

pub fn spawn_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Collider::cuboid(LARGE_PLATFORM_SIZE / 2.0, 1.0, LARGE_PLATFORM_SIZE / 2.0),
        RigidBody::Fixed,
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: LARGE_PLATFORM_SIZE,
                ..default()
            })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        },
    ));

    let cube = meshes.add(Mesh::from(shape::Cube {
        size: PLATFORM_SIZE,
    }));

    let mut rng = rand::thread_rng();

    let mut position = Vec3::X;

    for index in 0..100 {
        let i = index as f32;

        commands.spawn((
            Platform,
            Collider::cuboid(PLATFORM_SIZE / 2.0, PLATFORM_SIZE, PLATFORM_SIZE / 2.0),
            RigidBody::Fixed,
            PbrBundle {
                mesh: cube.clone(),
                material: materials.add(Color::rgb(i.sin() / 2.0 + 1.0, 0.5, 0.5).into()),
                transform: Transform::from_translation(position),
                ..default()
            },
        ));

        position += Vec3::new(
            rng.gen_range(5.0..10.0),
            rng.gen_range(-5.0..2.0),
            rng.gen_range(-5.0..5.0),
        )
        .normalize()
            * 5.0;
    }
}
