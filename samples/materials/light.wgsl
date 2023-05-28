struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct Instance {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
    @location(6) color: vec4<f32>,
}

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

struct Camera {
    projection: mat4x4<f32>,
    view: mat4x4<f32>,
    position: vec3<f32>,
}

@group(0)
@binding(0)
var<uniform> camera: Camera;

@vertex
fn vs_main(in: VsIn, instance: Instance) -> VsOut {
    let model = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    let position = camera.projection * camera.view * model * vec4<f32>(in.position, 1.0);

    return VsOut(
        position,
        instance.color,
    );
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return in.color;
}
