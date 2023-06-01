struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) frag_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
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
    let frag_position = (model.model_matrix * vec4<f32>(in.position, 1.0)).xyz;
    let normal = (model.normal_matrix * vec4<f32>(in.normal, 0.0)).xyz;

    return VsOut(
        position,
        frag_position,
        normal,
    );
}

struct Light {
    ambient: vec4<f32>,
    diffuse: vec4<f32>,
    specular: vec4<f32>,
    position: vec3<f32>,
    _pad1: f32,
}

struct Material {
    ambient: vec4<f32>,
    diffuse: vec4<f32>,
    specular: vec4<f32>,
    shininess: f32,
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
}

@group(0)
@binding(1)
var<uniform> light: Light;

@group(2)
@binding(0)
var<uniform> material: Material;

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let normal = normalize(in.normal);
    let light_direction = normalize(light.position - in.frag_position);

    let ambient = light.ambient * material.ambient;

    let diff = max(dot(in.normal, light_direction), 0.0);
    let diffuse = light.diffuse * diff * material.diffuse;

    let view_direction = normalize(camera.position - in.frag_position);
    let reflect_direction = reflect(-light_direction, normal);
    let spec = pow(max(dot(view_direction, reflect_direction), 0.0), material.shininess);
    let specular = light.specular * spec * material.specular;

    return ambient + diffuse + specular;
}
