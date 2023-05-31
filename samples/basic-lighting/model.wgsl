struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) frag_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) object_color: vec4<f32>,
};

struct Camera {
    projection: mat4x4<f32>,
    view: mat4x4<f32>,
    position: vec3<f32>,
};

struct Model {
    model_matrix: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
    color: vec4<f32>,
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
    let frag_position = (model.model_matrix * vec4<f32>(in.position, 1.0)).xyz;
    let normal = (model.normal_matrix * vec4<f32>(in.normal, 0.0)).xyz;

    return VsOut(
        position,
        frag_position,
        normal,
        model.color,
    );
}

struct Light {
    color: vec4<f32>,
    position: vec3<f32>,
    _pad1: f32,
}

@group(0)
@binding(1)
var<uniform> light: Light;

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let normal = normalize(in.normal);
    let light_direction = normalize(light.position - in.frag_position);

    let ambient_strength = 0.1;
    let ambient = ambient_strength * light.color;

    let diff = max(dot(in.normal, light_direction), 0.0);
    let diffuse = diff * light.color;

    let specular_strength = 0.5;
    let view_direction = normalize(camera.position - in.frag_position);
    let reflect_direction = reflect(-light_direction, normal);
    let spec = pow(max(dot(view_direction, reflect_direction), 0.0), 32.0);
    let specular = specular_strength * spec * light.color;

    return (ambient + diffuse + specular) * in.object_color;
}
