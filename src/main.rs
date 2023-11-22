use bevy::prelude::*;

mod player;
mod terrain;

use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use player::*;
use terrain::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // RapierDebugRenderPlugin::default(),
            RapierPhysicsPlugin::<NoUserData>::default(),
        ))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.5,
        })
        .add_systems(Startup, (spawn_terrain, spawn_player))
        .add_systems(Update, (player_movement, player_touch_platform, respawn))
        .run();
}
