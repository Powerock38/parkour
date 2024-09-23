use bevy::prelude::*;

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
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                }),
            RapierPhysicsPlugin::<NoUserData>::default(),
            // bevy_rapier3d::render::RapierDebugRenderPlugin::default(),
            MaterialPlugin::<SkyboxCustomMaterial>::default(),
        ))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1000.0,
        })
        .init_state::<AppState>()
        .init_resource::<Game>()
        .init_resource::<PlatformGeneration>()
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
                update_hud,
            )
                .run_if(in_state(AppState::Game)),
        )
        .observe(change_theme)
        .observe(spawn_platform)
        .run();
}
