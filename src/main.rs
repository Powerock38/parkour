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
        .add_systems(Startup, (init_hud, init_game.after(init_hud), spawn_player))
        .add_systems(
            Update,
            (
                player_movement,
                camera_rotation,
                respawn,
                force_respawn,
                update_moving_platforms,
            ),
        )
        .add_systems(PostUpdate, (player_touch_platform, player_hover_platform));

    let spawn_platform_system = app.world.register_system(spawn_platform);
    let update_hud_system = app.world.register_system(update_hud);

    app.world.insert_resource(Game {
        started: false,
        points: 0,
        update_hud_system,
        spawn_platform_system,
        next_platform_position: Vec3::ZERO,
    });

    app.run();
}
