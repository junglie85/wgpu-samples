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
    @location(7) normal_matrix_0: vec3<f32>,
    @location(8) normal_matrix_1: vec3<f32>,
    @location(9) normal_matrix_2: vec3<f32>,
}

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) object_color: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) frag_position: vec3<f32>,
}

struct Camera {
    projection: mat4x4<f32>,
    view: mat4x4<f32>,
    position: vec3<f32>,
}

struct Light {
    color: vec4<f32>,
    position: vec3<f32>,
    _pad: f32,
}

@group(0)
@binding(0)
var<uniform> camera: Camera;

@group(0)
@binding(1)
var<uniform> light: Light;

@vertex
fn vs_main(in: VsIn, instance: Instance) -> VsOut {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
    );

    let position = camera.projection * camera.view * model_matrix * vec4<f32>(in.position, 1.0);
    let frag_position = (model_matrix * vec4<f32>(in.position, 1.0)).xyz;

    let normal = normal_matrix * in.normal;

    return VsOut(
        position,
        instance.color,
        normal,
        frag_position,
    );
}

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

    let result = (ambient + diffuse + specular) * in.object_color;

    return result;
}
