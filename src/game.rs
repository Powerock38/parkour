use bevy::{ecs::system::SystemId, gltf::Gltf, prelude::*};

use crate::{
    platforms::Platform,
    player::{Player, SPAWN_POINT},
};

const NB_PLATFORMS_INIT: u32 = 10;

#[derive(Resource)]
pub struct Game {
    pub theme_platforms_handles: Vec<Handle<Gltf>>,
    pub change_theme_system: SystemId,
    pub started: bool,
    pub points: u32,
    pub update_hud_system: SystemId,
    pub spawn_platform_system: SystemId,
    pub next_platform_position: Vec3,
    pub direction_bias_horizontal: f64,
    pub direction_bias_vertical: f64,
}

impl Game {
    pub fn difficulty(&self) -> f32 {
        self.points as f32 / 1000.0
    }
}

pub fn init_game(mut commands: Commands, mut game: ResMut<Game>) {
    game.started = false;
    game.points = 0;
    game.next_platform_position = Vec3::ZERO;
    game.direction_bias_horizontal = 0.0;
    game.direction_bias_vertical = 0.0;

    commands.run_system(game.update_hud_system);
    commands.run_system(game.change_theme_system);

    for _ in 0..NB_PLATFORMS_INIT {
        commands.run_system(game.spawn_platform_system);
    }
}

pub fn init_hud(mut commands: Commands) {
    commands.spawn((
        TextBundle::from_section(
            "Text Example",
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
    let mut text = query.single_mut();
    if game.points == 0 {
        text.sections[0].value = "JUMP TO START".to_string();
    } else {
        text.sections[0].value = format!("Score: {}", game.points);
    }
}

pub fn reset(
    game: ResMut<Game>,
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

        init_game(commands, game);
    }
}

pub fn force_respawn(
    keyboard_input: Res<Input<KeyCode>>,
    mut player: Query<(&mut Player, &mut Transform)>,
) {
    if keyboard_input.just_pressed(KeyCode::R) {
        let (mut player, mut transform) = player.single_mut();
        transform.translation.y = -100.0;
        player.velocity_y = -100.0;
    }
}

pub fn force_theme_change(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    game: Res<Game>,
) {
    if keyboard_input.just_pressed(KeyCode::T) {
        commands.run_system(game.change_theme_system);
    }
}
