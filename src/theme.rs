use bevy::{asset::LoadState, core_pipeline::Skybox, gltf::Gltf, prelude::*};
use rand::seq::SliceRandom;

use crate::game::Game;

// https://jaxry.github.io/panorama-to-cubemap/
// https://www.imgonline.com.ua/eng/cut-photo-into-pieces.php
// https://sora.ws/gltf/
// https://matheowis.github.io/HDRI-to-CubeMap/
// toktx --cubemap --t2 sky.ktx2 right.jpg left.jpg top.jpg bottom.jpg front.jpg back.jpg
// toktx --cubemap --t2 sky.ktx2 px.png nx.png py.png ny.png pz.png nz.png
pub const THEME_CHANGE_CHANCE: f64 = 0.01;

pub struct Theme {
    pub skybox: &'static str,
    pub platforms: &'static [&'static str],
}

pub const THEMES: &[Theme] = &[
    Theme {
        skybox: "green_explosion.ktx2",
        platforms: &[
            "ground1.glb",
            "ground2.glb",
            "ground1_top1.glb",
            "ground2_top2.glb",
            "sand.glb",
            "sand_top2.glb",
            "rock.glb",
            "rock_top1.glb",
            "rock_top2.glb",
        ],
    },
    Theme {
        skybox: "nebula_dark.ktx2",
        platforms: &[
            "asteroid1.glb",
            "asteroid2.glb",
            "asteroid3.glb",
            "asteroid4.glb",
            "asteroid5.glb",
        ],
    },
    // Theme {
    //     skybox: "nebula_blue.ktx2",
    //     platforms: &[
    //         "space_rock1.glb",
    //     ],
    // },
    // Theme {
    //     skybox: "nebula.ktx2",
    //     platforms: &[
    //         "rock1.glb",
    //         "rock2.glb",
    //         "rock3.glb",
    //         "rock4.glb",
    //         "rock5.glb",
    //     ],
    // },
];

#[derive(Resource)]
pub struct ThemeChange {
    pub skybox: Handle<Image>,
    pub platforms: Vec<Handle<Gltf>>,
}

pub fn change_theme(
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    mut game: ResMut<Game>,
) {
    let mut rng = rand::thread_rng();

    let theme = THEMES.choose(&mut rng).unwrap();
    println!("new theme: skybox: {:?}", theme.skybox);

    let theme_change = ThemeChange {
        skybox: assets_server.load(format!("skyboxes/{}", theme.skybox)),
        platforms: theme
            .platforms
            .iter()
            .map(|path| assets_server.load(format!("platforms/{}", path)))
            .collect(),
    };

    if game.theme_platforms_handles.is_empty() {
        game.theme_platforms_handles = theme_change.platforms.clone();
    }

    commands.insert_resource(theme_change);
}

pub fn apply_loaded_theme(
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    camera: Query<Entity, With<Camera3d>>,
    theme_change: Option<Res<ThemeChange>>,
    mut game: ResMut<Game>,
) {
    if let Some(theme_change) = theme_change {
        let fully_loaded = theme_change
            .platforms
            .iter()
            .all(|handle| assets_server.load_state(handle) == LoadState::Loaded)
            && assets_server.load_state(&theme_change.skybox) == LoadState::Loaded;

        if fully_loaded {
            commands
                .entity(camera.single())
                .insert(Skybox(theme_change.skybox.clone()));
            game.theme_platforms_handles = theme_change.platforms.clone();

            commands.remove_resource::<ThemeChange>();
        }
    }
}
