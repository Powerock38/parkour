#import bevy_pbr::{mesh_view_bindings::globals, forward_io::VertexOutput}

const DURATION: f32 = 20.0;

struct SkyboxCustomMaterial {
    time_t0: f32,
    // WebGL2 structs must be 16 byte aligned.
    _padding_0: f32,
    _padding_1: f32,
    _padding_2: f32,
}

@group(2) @binding(0)
var<uniform> material: SkyboxCustomMaterial;

@group(2) @binding(1)
var skybox_texture1: texture_cube<f32>;
@group(2) @binding(2)
var skybox_texture1_sampler: sampler;

@group(2) @binding(3)
var skybox_texture2: texture_cube<f32>;
@group(2) @binding(4)
var skybox_texture2_sampler: sampler;

// Noise functions from https://gist.github.com/munrocket/236ed5ba7e409b8bdf1ff6eca5dcdc39

fn rand22(n: vec2f) -> f32 {
    return fract(sin(dot(n, vec2f(12.9898, 4.1414))) * 43758.5453);
}

fn noise2(n: vec2f) -> f32 {
    let d = vec2f(0., 1.);
    let b = floor(n);
    let f = smoothstep(vec2f(0.), vec2f(1.), fract(n));
    return mix(mix(rand22(b), rand22(b + d.yx), f.x), mix(rand22(b + d.xy), rand22(b + d.yy), f.x), f.y);
}

const m2: mat2x2f = mat2x2f(vec2f(0.8, 0.6), vec2f(-0.6, 0.8));
fn fbm(p: vec2f) -> f32 {
    var f: f32 = 0.;
    var p_ = p;
    f = f + 0.5000 * noise2(p_); p_ = m2 * p_ * 2.02;
    f = f + 0.2500 * noise2(p_); p_ = m2 * p_ * 2.03;
    f = f + 0.1250 * noise2(p_); p_ = m2 * p_ * 2.01;
    f = f + 0.0625 * noise2(p_);
    return f / 0.9375;
}

@fragment
fn fragment(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    let texture1: vec4<f32> = textureSample(skybox_texture1, skybox_texture1_sampler, in.world_normal);
    let texture2: vec4<f32> = textureSample(skybox_texture2, skybox_texture2_sampler, in.world_normal);

    let progress = min((globals.time - material.time_t0) / DURATION, 1.0);

    if progress >= 0.8 {
        return texture1;
    }

    let p = 5.0 * (1.0 - progress);
    let noise_xy = fbm(in.world_normal.xy * p);
    let noise_yz = fbm(in.world_normal.yz * p);
    let noise_xz = fbm(in.world_normal.xz * p);
    let noise = (noise_xy + noise_yz + noise_xz) / 3.0;

    let t = 0.8 - progress * 0.8;
    if noise > t {
        return texture1;
    } else {
        return texture2;
    }
}