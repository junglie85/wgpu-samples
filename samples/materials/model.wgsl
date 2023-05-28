struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct Instance {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
    @location(6) normal_matrix_0: vec3<f32>,
    @location(7) normal_matrix_1: vec3<f32>,
    @location(8) normal_matrix_2: vec3<f32>,
    @location(9) material_ambient: vec4<f32>,
    @location(10) material_diffuse: vec4<f32>,
    @location(11) material_specular: vec4<f32>,
    @location(12) material_shininess: f32,
}

struct Material {
    ambient: vec4<f32>,
    diffuse: vec4<f32>,
    specular: vec4<f32>,
    shininess: f32,
}

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) frag_position: vec3<f32>,
    @location(2) material_ambient: vec4<f32>,
    @location(3) material_diffuse: vec4<f32>,
    @location(4) material_specular: vec4<f32>,
    @location(5) material_shininess: f32,
}

struct Camera {
    projection: mat4x4<f32>,
    view: mat4x4<f32>,
    position: vec3<f32>,
}

struct Light {
    ambient: vec4<f32>,
    diffuse: vec4<f32>,
    specular: vec4<f32>,
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
        normal,
        frag_position,
        instance.material_ambient,
        instance.material_specular,
        instance.material_diffuse,
        instance.material_shininess,
    );
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let material = Material(
        in.material_ambient,
        in.material_specular,
        in.material_diffuse,
        in.material_shininess,
    );

    let normal = normalize(in.normal);
    let light_direction = normalize(light.position - in.frag_position);

    let ambient = light.ambient * material.ambient;

    let diff = max(dot(in.normal, light_direction), 0.0);
    let diffuse = light.diffuse * diff * material.diffuse;

    let view_direction = normalize(camera.position - in.frag_position);
    let reflect_direction = reflect(-light_direction, normal);
    let spec = pow(max(dot(view_direction, reflect_direction), 0.0), material.shininess);
    let specular = light.specular * spec * material.specular;

    let result = ambient + diffuse + specular;

    return result;
}
