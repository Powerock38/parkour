use bevy::prelude::*;
use bevy_wasm_window_resize::WindowResizePlugin;

mod game;
mod platforms;
mod player;

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
        .add_systems(
            Startup,
            (
                spawn_player,
                init_hud,
                init_game.after(init_hud).after(spawn_player),
            ),
        )
        .add_systems(
            Update,
            (
                player_touch_platform,
                player_hover_platform,
                player_movement,
                camera_rotation,
                respawn,
                force_respawn,
                update_moving_platforms,
            ),
        );

    let spawn_platform_system = app.world.register_system(spawn_platform);
    let update_hud_system = app.world.register_system(update_hud);
    let change_skybox_system = app.world.register_system(change_skybox);

    app.world.insert_resource(Game {
        started: false,
        points: 0,
        update_hud_system,
        spawn_platform_system,
        next_platform_position: Vec3::ZERO,
        direction_bias_horizontal: 0.0,
        direction_bias_vertical: 0.0,
        skybox: SKYBOXES[0],
        change_skybox_system,
    });

    app.run();
}
