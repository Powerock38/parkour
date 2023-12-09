use bevy::{
    gltf::{Gltf, GltfMesh, GltfNode},
    prelude::*,
};
use bevy_rapier3d::prelude::*;
use rand::{seq::SliceRandom, Rng};
use std::f32::consts::PI;

use crate::{game::Game, theme::THEME_CHANGE_CHANCE};

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

pub const TOUCHED_PLATFORM_TTL: f32 = 10.0;

#[derive(Component)]
pub struct Touched(pub Timer);

#[derive(Component)]
pub struct Hovered;

#[derive(Component)]
pub struct GltfLoader {
    handle: Handle<Gltf>,
    transform: Transform,
}

pub fn spawn_platform(mut game: ResMut<Game>, mut commands: Commands) {
    let mut rng = rand::thread_rng();

    // Small chance to change current theme
    if rng.gen_bool(THEME_CHANGE_CHANCE) {
        commands.run_system(game.change_theme_system);
    }

    // Platform mesh
    let size = (1.0 - game.difficulty()) * rng.gen_range(0.8..1.2);

    let handle = game
        .theme_platforms_handles
        .choose(&mut rng)
        .unwrap()
        .clone();

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

    let mut transform = Transform::from_translation(position)
        .with_scale(Vec3::splat(size))
        .looking_at(game.next_platform_position, Vec3::Y);

    transform.rotate_y(rng.gen_range(0.0..PI * 2.0));

    // Spawn platform
    let mut c = commands.spawn((GltfLoader { handle, transform }, Platform));

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

pub fn delete_touched_platforms(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Touched)>,
) {
    for (entity, mut touched) in query.iter_mut() {
        if touched.0.tick(time.delta()).finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn gltf_compute_colliders(
    mut commands: Commands,
    mut query_gltf_loader: Query<(&GltfLoader, Entity)>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_mesh_assets: Res<Assets<GltfMesh>>,
    gltf_node_assets: Res<Assets<GltfNode>>,
    mesh_assets: Res<Assets<Mesh>>,
) {
    for (gltf_loader, gltf_loader_entity) in query_gltf_loader.iter_mut() {
        let gltf = gltf_assets.get(&gltf_loader.handle);

        if let Some(gltf) = gltf {
            if gltf.nodes.len() != 1 {
                warn!(
                    "Gltf file must have exactly one node: {} has {} nodes",
                    gltf_loader.handle.path().unwrap(),
                    gltf.nodes.len()
                );
            }

            let node = gltf_node_assets.get(&gltf.nodes[0]).unwrap();
            let gltf_mesh = gltf_mesh_assets.get(&node.mesh.clone().unwrap()).unwrap();
            let mesh_primitive = &gltf_mesh.primitives[0];
            let mesh = mesh_assets.get(&mesh_primitive.mesh).unwrap();

            commands
                .entity(gltf_loader_entity)
                .insert((
                    SceneBundle {
                        scene: gltf.scenes.first().unwrap().clone(),
                        transform: gltf_loader.transform,
                        ..default()
                    },
                    Collider::from_bevy_mesh(mesh, &ComputedColliderShape::default()).unwrap(),
                    RigidBody::Fixed,
                ))
                .remove::<GltfLoader>();
        }
    }
}
