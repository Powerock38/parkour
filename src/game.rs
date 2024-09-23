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
        TextBundle::from_section(
            "Loading...",
            TextStyle {
                font_size: 30.0,
                ..default()
            },
        )
        .with_style(Style {
            margin: UiRect::all(Val::Px(5.)),
            ..default()
        }),
        Label,
    ));
}

pub fn update_hud(game: Res<Game>, mut query: Query<&mut Text, With<Label>>) {
    if game.is_added() || game.is_changed() {
        let mut text = query.single_mut();
        if game.started {
            text.sections[0].value = format!("Score: {}", game.points);
        } else {
            text.sections[0].value = "JUMP TO START".to_string();
        }
    }
}

pub fn reset(
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

        init_game(commands);
    }
}
