use bevy::{asset::LoadState, prelude::*};
use rand::seq::SliceRandom;

use crate::{
    skybox::{SkyboxCustom, SkyboxCustomMaterial},
    AppState,
};

// https://jaxry.github.io/panorama-to-cubemap/
// https://www.imgonline.com.ua/eng/cut-photo-into-pieces.php
// https://sora.ws/gltf/
// https://matheowis.github.io/HDRI-to-CubeMap/
// toktx --cubemap --t2 sky.ktx2 right.jpg left.jpg top.jpg bottom.jpg front.jpg back.jpg
// toktx --cubemap --t2 sky.ktx2 px.png nx.png py.png ny.png pz.png nz.png
pub const THEME_CHANGE_CHANCE: f64 = 0.01;

pub struct Theme {
    pub id: &'static str,
    pub skybox: &'static str,
    pub platforms: &'static [&'static str],
}

pub const THEMES: &[Theme] = &[
    Theme {
        id: "heaven",
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
        id: "space",
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

#[derive(Clone)]
pub struct ThemeLoad {
    pub id: &'static str,
    pub skybox: Handle<Image>,
    pub platforms: Vec<Handle<Scene>>,
}

#[derive(Event)]
pub struct ChangeThemeRandom;

#[derive(Resource)]
pub struct ThemeCurrent {
    pub theme: ThemeLoad,
}

#[derive(Resource)]
pub struct ThemeChange {
    pub theme: ThemeLoad,
}

pub fn change_theme(
    _trigger: Trigger<ChangeThemeRandom>,
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    theme_current: Option<Res<ThemeCurrent>>,
) {
    let mut rng = rand::thread_rng();

    let themes = THEMES
        .iter()
        .filter(|theme| {
            theme_current
                .as_ref()
                .map_or(true, |theme_current| theme_current.theme.id != theme.id)
        })
        .collect::<Vec<_>>();

    let theme = themes.choose(&mut rng).unwrap();
    println!("loading theme with skybox: {:?}", theme.skybox);

    commands.insert_resource(ThemeChange {
        theme: ThemeLoad {
            id: theme.id,
            skybox: assets_server.load(format!("skyboxes/{}", theme.skybox)),
            platforms: theme
                .platforms
                .iter()
                .map(|path| {
                    assets_server
                        .load(GltfAssetLabel::Scene(0).from_asset(format!("platforms/{path}")))
                })
                .collect(),
        },
    });
}

pub fn apply_loaded_theme(
    mut commands: Commands,
    time: Res<Time>,
    assets_server: Res<AssetServer>,
    current_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    theme_current: Option<Res<ThemeCurrent>>,
    theme_change: Option<Res<ThemeChange>>,
    skybox_entity: Query<Entity, With<SkyboxCustom>>,
    mut skybox_materials: ResMut<Assets<SkyboxCustomMaterial>>,
) {
    if let Some(theme_change) = theme_change {
        let fully_loaded = theme_change
            .theme
            .platforms
            .iter()
            .all(|handle| matches!(assets_server.load_state(handle), LoadState::Loaded))
            && matches!(
                assets_server.load_state(&theme_change.theme.skybox),
                LoadState::Loaded
            );

        if fully_loaded {
            let mut time_t0 = time.elapsed_secs_wrapped();
            let sky_texture1 = theme_change.theme.skybox.clone();
            let sky_texture2 = theme_current.map_or_else(
                || {
                    time_t0 += 30.0; // prevent shader from running for nothing when tex1 == tex2
                    sky_texture1.clone()
                },
                |theme_current| theme_current.theme.skybox.clone(),
            );

            commands
                .entity(skybox_entity.single())
                .insert(MeshMaterial3d(skybox_materials.add(
                    SkyboxCustomMaterial::new(time_t0, sky_texture1, sky_texture2),
                )));

            commands.insert_resource(ThemeCurrent {
                theme: theme_change.theme.clone(),
            });

            commands.remove_resource::<ThemeChange>();

            if let AppState::Loading = current_state.get() {
                next_state.set(AppState::Game);
            }
        }
    }
}

pub fn force_theme_change(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        commands.trigger(ChangeThemeRandom);
    }
}
