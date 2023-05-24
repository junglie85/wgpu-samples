struct Globals {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
}

struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
};

struct VsOut {
    @location(1) tex_coord: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

@group(0)
@binding(0)
var<uniform> globals: Globals;

@group(1)
@binding(0)
var<uniform> model: mat4x4<f32>;

@vertex
fn vs_main(in: VsIn) -> VsOut {
    let position = globals.projection * globals.view * model * vec4<f32>(in.position, 1.0);
    
    return VsOut (
        in.tex_coord,
        position
    );
}

@group(1)
@binding(1)
var texture_sampler: sampler;

@group(1)
@binding(2)
var texture_1: texture_2d<f32>;

@group(1)
@binding(3)
var texture_2: texture_2d<f32>;

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return mix(
        textureSample(texture_1, texture_sampler, in.tex_coord),
        textureSample(texture_2, texture_sampler, in.tex_coord),
        0.2
    );
}
