use bevy::prelude::*;
use bevy_wasm_window_resize::WindowResizePlugin;

mod game;
mod platforms;
mod player;
mod utils;

use bevy_rapier3d::plugin::{NoUserData, RapierPhysicsPlugin};
use game::*;
use platforms::*;
use player::*;

fn main() {
    let mut app = App::new();
    let app = app
        .add_plugins((
            DefaultPlugins,
            WindowResizePlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            // bevy_rapier3d::render::RapierDebugRenderPlugin::default(),
        ))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.5,
        })
        .add_systems(Startup, (init_hud, init_game, spawn_player))
        .add_systems(
            Update,
            (
                player_movement,
                camera_rotation,
                player_touch_platform,
                player_hover_platform,
                respawn,
                force_respawn,
                update_moving_platforms,
            ),
        )
        .add_systems(PostUpdate, (update_hud,));

    let generate_platform = app.world.register_system(spawn_platform);

    app.world.insert_resource(Game {
        points: 0,
        generate_platform,
        next_platform_position: Vec3::ZERO,
    });

    app.run();
}
