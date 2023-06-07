struct Camera {
    projection: mat4x4<f32>,
    view: mat4x4<f32>,
    position: vec3<f32>,
}

struct Transform {
    model_matrix: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
};

struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@group(0)
@binding(0)
var<uniform> camera: Camera;

@group(1)
@binding(0)
var<uniform> transform: Transform;

@group(2)
@binding(0)
var s_diffuse: sampler;

@group(2)
@binding(1)
var t_diffuse: texture_2d<f32>;

@vertex
fn vs_main(in: VsIn) -> VsOut {
    let position = camera.projection * camera.view * transform.model_matrix * vec4<f32>(in.position, 1.0);

    return VsOut(position, in.tex_coords);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
