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
}

struct VsOut {
    @builtin(position) position: vec4<f32>,
}

@group(0)
@binding(0)
var<uniform> camera: Camera;

@group(1)
@binding(0)
var<uniform> transform: Transform;

@vertex
fn vs_main(in: VsIn) -> VsOut {
    let position = camera.projection * camera.view * transform.model_matrix * vec4<f32>(in.position, 1.0);

    return VsOut(position);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.5, 0.2, 1.0);
}
