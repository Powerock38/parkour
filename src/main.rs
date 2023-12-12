use bevy::prelude::*;
use bevy_wasm_window_resize::WindowResizePlugin;

mod game;
mod platforms;
mod player;
mod skybox;
mod theme;

use bevy_rapier3d::plugin::{NoUserData, RapierPhysicsPlugin};
use game::*;
use platforms::*;
use player::*;
use skybox::*;
use theme::*;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Loading,
    Game,
}

fn main() {
    let mut app = App::new();
    let app = app
        .add_plugins((
            DefaultPlugins,
            WindowResizePlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            // bevy_rapier3d::render::RapierDebugRenderPlugin::default(),
            MaterialPlugin::<SkyboxCustomMaterial>::default(),
        ))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.5,
        })
        .add_state::<AppState>()
        .add_systems(Startup, (spawn_player, init_hud))
        .add_systems(Update, (apply_loaded_theme,))
        .add_systems(OnEnter(AppState::Game), (init_game,))
        .add_systems(
            Update,
            (
                player_touch_platform,
                player_hover_platform,
                player_movement,
                camera_rotation,
                update_moving_platforms,
                delete_touched_platforms,
                gltf_compute_colliders,
                reset,
                force_respawn,
                force_theme_change,
            )
                .run_if(in_state(AppState::Game)),
        );

    let change_theme_system = app.world.register_system(change_theme);

    let _ = app.world.run_system(change_theme_system);

    app.world
        .insert_resource(ThemeChangeSystem(change_theme_system));

    let spawn_platform_system = app.world.register_system(spawn_platform);
    let update_hud_system = app.world.register_system(update_hud);

    app.world.insert_resource(Game {
        started: false,
        points: 0,
        update_hud_system,
        spawn_platform_system,
        next_platform_position: Vec3::ZERO,
        direction_bias_horizontal: 0.0,
        direction_bias_vertical: 0.0,
    });

    app.run();
}
