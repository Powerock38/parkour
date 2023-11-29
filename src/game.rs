use bevy::{ecs::system::SystemId, prelude::*};

const NB_PLATFORMS_INIT: u32 = 10;

#[derive(Resource)]
pub struct Game {
    pub started: bool,
    pub points: u32,
    pub generate_platform: SystemId,
    pub next_platform_position: Vec3,
}

pub fn init_game(mut commands: Commands, mut game: ResMut<Game>) {
    game.started = false;

    for _ in 0..NB_PLATFORMS_INIT {
        commands.run_system(game.generate_platform);
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
    text.sections[0].value = format!("Score: {}", game.points);
}
