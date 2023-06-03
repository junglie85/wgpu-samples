struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
};

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) frag_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
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
        in.tex_coords,
    );
}

struct Light {
    ambient: vec4<f32>,
    diffuse: vec4<f32>,
    specular: vec4<f32>,
    position: vec3<f32>,
    constant: f32,
    linear: f32,
    quadratic: f32,
    _pad1: f32,
    _pad2: f32,
}

struct Material {
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

@group(2)
@binding(1)
var diffuse_sampler: sampler;

@group(2)
@binding(2)
var diffuse_texture: texture_2d<f32>;

@group(2)
@binding(3)
var specular_sampler: sampler;

@group(2)
@binding(4)
var specular_texture: texture_2d<f32>;

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let normal = normalize(in.normal);
    let light_direction = normalize(light.position - in.frag_position);

    var ambient = light.ambient * textureSample(diffuse_texture, diffuse_sampler, in.tex_coords);

    let diff = max(dot(in.normal, light_direction), 0.0);
    var diffuse = light.diffuse * diff * textureSample(diffuse_texture, diffuse_sampler, in.tex_coords);

    let view_direction = normalize(camera.position - in.frag_position);
    let reflect_direction = reflect(-light_direction, normal);
    let spec = pow(max(dot(view_direction, reflect_direction), 0.0), material.shininess);
    var specular = light.specular * spec * textureSample(specular_texture, specular_sampler, in.tex_coords);

    let distance = length(light.position - in.frag_position);
    let attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * distance * distance);

    ambient *= attenuation;
    diffuse *= attenuation;
    specular *= attenuation;

    return ambient + diffuse + specular;
}
