use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    platforms::Platform,
    player::{Player, SPAWN_POINT},
    ChangeThemeRandom, PlatformGeneration, SpawnPlatform,
};

const NB_PLATFORMS_INIT: u32 = 10;

#[derive(Resource, Default)]
pub struct Game {
    pub started: bool,
    pub points: u32,
}

impl Game {
    pub fn difficulty(&self) -> f32 {
        self.points as f32 / 1000.0
    }
}

pub fn init_game(mut commands: Commands) {
    commands.insert_resource(Game::default());
    commands.insert_resource(PlatformGeneration::default());

    for _ in 0..NB_PLATFORMS_INIT {
        commands.trigger(SpawnPlatform);
    }
}

pub fn init_hud(mut commands: Commands) {
    commands.trigger(ChangeThemeRandom);

    commands.spawn((
        Label,
        Text::new("Loading..."),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        Node {
            margin: UiRect::all(Val::Px(5.)),
            ..default()
        },
    ));
}

pub fn update_hud(game: Res<Game>, mut query: Query<&mut Text, With<Label>>) {
    if game.is_added() || game.is_changed() {
        let mut text = query.single_mut();
        if game.started {
            text.0 = format!("Score: {}", game.points);
        } else {
            text.0 = "JUMP TO START".to_string();
        }
    }
}

pub fn reset(
    mut commands: Commands,
    q_player: Single<(&mut Transform, &mut LinearVelocity), With<Player>>,
    q_camera: Single<&mut Transform, (With<Camera3d>, Without<Player>)>,
    platforms: Query<Entity, With<Platform>>,
) {
    let (mut transform, mut velocity) = q_player.into_inner();

    if velocity.y < -20.0 {
        for entity in &platforms {
            commands.entity(entity).despawn_recursive();
        }

        let mut camera_transform = q_camera.into_inner();
        camera_transform.look_at(Vec3::X, Vec3::Y);

        transform.translation = SPAWN_POINT;
        *velocity = LinearVelocity::ZERO;

        init_game(commands);
    }
}
