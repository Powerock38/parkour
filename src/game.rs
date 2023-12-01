use bevy::{core_pipeline::Skybox, ecs::system::SystemId, prelude::*};

const NB_PLATFORMS_INIT: u32 = 10;

// https://jaxry.github.io/panorama-to-cubemap/
// https://www.imgonline.com.ua/eng/cut-photo-into-pieces.php
// https://sora.ws/gltf/
// toktx --cubemap --t2 sky.ktx2 right.jpg left.jpg top.jpg bottom.jpg front.jpg back.jpg
// toktx --cubemap --t2 sky.ktx2 px.png nx.png py.png ny.png pz.png nz.png
pub const SKYBOXES: &[&str] = &[
    "green_explosion.ktx2",
    "nebula_dark.ktx2",
    "nebula.ktx2",
    "orange_sky.ktx2",
    "blue_sky.ktx2",
];

pub const SKYBOX_CHANGE_CHANCE: f64 = 0.01;

#[derive(Resource)]
pub struct Game {
    pub skybox: &'static str,
    pub change_skybox_system: SystemId,
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

    for _ in 0..NB_PLATFORMS_INIT {
        commands.run_system(game.spawn_platform_system);
    }

    commands.run_system(game.update_hud_system);
    commands.run_system(game.change_skybox_system);
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
    text.sections[0].value = format!("Score: {}", game.points);
}

pub fn change_skybox(
    game: Res<Game>,
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    entity: Query<Entity, With<Camera3d>>,
) {
    let camera = entity.single();
    commands
        .entity(camera)
        .insert(Skybox(assets_server.load(game.skybox)));
}
