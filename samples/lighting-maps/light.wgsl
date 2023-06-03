struct VsIn {
    @location(0) position: vec3<f32>,
};

struct VsOut {
    @builtin(position) position: vec4<f32>,
};

struct Camera {
    projection: mat4x4<f32>,
    view: mat4x4<f32>,
    position: vec3<f32>,
};

struct Model {
    model_matrix: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
};

@group(0)
@binding(0)
var<uniform> camera: Camera;

@group(1)
@binding(0)
var<uniform> model: Model;

@vertex
fn vs_main(in: VsIn) -> VsOut {
    let position = camera.projection * camera.view * model.model_matrix * vec4<f32>(in.position, 1.0);

    return VsOut(
        position,
    );
}

@group(2)
@binding(0)
var<uniform> light_color: vec4<f32>;

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return light_color;
}
