use bevy::{ecs::system::SystemId, prelude::*};

const NB_PLATFORMS_INIT: u32 = 10;

#[derive(Resource)]
pub struct Game {
    pub started: bool,
    pub points: u32,
    pub update_hud_system: SystemId,
    pub spawn_platform_system: SystemId,
    pub next_platform_position: Vec3,
}

impl Game {
    pub fn difficulty(&self) -> f32 {
        self.points as f32 / 1000.0
    }
}

pub fn init_game(mut commands: Commands, mut game: ResMut<Game>) {
    game.started = false;

    for _ in 0..NB_PLATFORMS_INIT {
        commands.run_system(game.spawn_platform_system);
    }

    commands.run_system(game.update_hud_system);
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
