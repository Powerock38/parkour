#import bevy_pbr::{
    mesh_view_bindings::globals,
    forward_io::VertexOutput,
}

const DURATION: f32 = 10.0;

@group(1) @binding(0)
var<uniform> time_t0: f32;

@group(1) @binding(1)
var skybox_texture1: texture_cube<f32>;
@group(1) @binding(2)
var skybox_texture1_sampler: sampler;

@group(1) @binding(3)
var skybox_texture2: texture_cube<f32>;
@group(1) @binding(4)
var skybox_texture2_sampler: sampler;

@fragment
fn fragment(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    let texture1: vec4<f32> = textureSample(skybox_texture1, skybox_texture1_sampler, in.world_normal);
    let texture2: vec4<f32> = textureSample(skybox_texture2, skybox_texture2_sampler, in.world_normal);

    let progress = min((globals.time - time_t0) / DURATION, 1.0);

    return mix(texture2, texture1, progress);
}