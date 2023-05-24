struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tex_coord: vec2<f32>,
};

struct VsOut {
    @location(0) color: vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(in: VsIn) -> VsOut {
    return VsOut (
        in.color,
        in.tex_coord,
        vec4<f32>(in.position, 1.0)
    );
}

@group(0)
@binding(0)
var texture_sampler: sampler;

@group(0)
@binding(1)
var texture_1: texture_2d<f32>;

@group(0)
@binding(2)
var texture_2: texture_2d<f32>;

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return mix(
        textureSample(texture_1, texture_sampler, in.tex_coord),
        textureSample(texture_2, texture_sampler, in.tex_coord),
        0.2
    );
}
