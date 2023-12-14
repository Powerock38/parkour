use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::Indices,
        render_resource::{AsBindGroup, PrimitiveTopology, ShaderRef},
    },
};

#[derive(Component)]
pub struct SkyboxCustom;

#[derive(AsBindGroup, TypeUuid, TypePath, Debug, Clone, Asset)]
#[uuid = "3e5bbf04-bf02-4b74-97f5-ea9c94ca3235"]
pub struct SkyboxCustomMaterial {
    #[uniform(0)]
    pub time_t0: f32,

    #[texture(1, dimension = "cube")]
    #[sampler(2)]
    pub sky_texture1: Handle<Image>,

    #[texture(3, dimension = "cube")]
    #[sampler(4)]
    pub sky_texture2: Handle<Image>,
}

impl Material for SkyboxCustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/skybox.wgsl".into()
    }
}

// from https://github.com/JonahPlusPlus/bevy_atmosphere/blob/master/src/skybox.rs
pub fn generate_skybox_mesh() -> Mesh {
    let far = 1000.0;
    let size = (far * 0.5_f32.sqrt()) - 1.0;
    // sqrt(0.5) is the ratio between squares separated by a circle
    // where one lies on the outside of the circle (edges) and the other lies on the inside of the circle (corners)
    // this is necessary since while the faces of the skybox may be seen, the corners and edges probably won't, since they don't lie on the radius of the far plane
    let norm = f32::sqrt(1. / 3.); // component of normalized (1, 1, 1)
    let (vertices, indices) = (
        &[
            ([size, size, size], [norm, norm, norm]),       // 0(+, +, +)
            ([-size, size, size], [-norm, norm, norm]),     // 1(-, +, +)
            ([size, -size, size], [norm, -norm, norm]),     // 2(+, -, +)
            ([size, size, -size], [norm, norm, -norm]),     // 3(+, +, -)
            ([-size, -size, size], [-norm, -norm, norm]),   // 4(-, -, +)
            ([size, -size, -size], [norm, -norm, -norm]),   // 5(+, -, -)
            ([-size, size, -size], [-norm, norm, -norm]),   // 6(-, +, -)
            ([-size, -size, -size], [-norm, -norm, -norm]), // 7(-, -, -)
        ],
        &[
            0, 5, 2, 5, 0, 3, // +X
            6, 4, 7, 4, 6, 1, // -X
            0, 6, 3, 6, 0, 1, // +Y
            2, 7, 4, 7, 2, 5, // -Y
            1, 2, 4, 2, 1, 0, // +Z
            3, 7, 5, 7, 3, 6, // -Z
        ],
    );

    let positions: Vec<_> = vertices.iter().map(|(p, _)| *p).collect();
    let normals: Vec<_> = vertices.iter().map(|(_, n)| *n).collect();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_indices(Some(Indices::U16(indices.to_vec())));
    mesh
}
