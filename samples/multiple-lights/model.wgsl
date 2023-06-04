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

struct DirectionalLight {
    direction: vec3<f32>,
    _pad1: f32,
    ambient: vec4<f32>,
    diffuse: vec4<f32>,
    specular: vec4<f32>,
}

struct PointLight {
    position: vec3<f32>,
    constant: f32,
    linear: f32,
    quadratic: f32,
    _pad1: f32,
    _pad2: f32,
    ambient: vec4<f32>,
    diffuse: vec4<f32>,
    specular: vec4<f32>,
}

struct SpotLight {
    position: vec3<f32>,
    _pad1: f32,
    direction: vec3<f32>,
    cut_off: f32,
    outer_cut_off: f32,
    constant: f32,
    linear: f32,
    quadratic: f32,
    ambient: vec4<f32>,
    diffuse: vec4<f32>,
    specular: vec4<f32>,
}

struct Material {
    shininess: f32,
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
}

const POINT_LIGHT_COUNT: i32 = 4;

@group(0)
@binding(1)
var<uniform> directional_light: DirectionalLight;

@group(0)
@binding(2)
var<uniform> point_lights: array<PointLight, POINT_LIGHT_COUNT>;


@group(0)
@binding(3)
var<uniform> spot_light: SpotLight;

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

fn calculate_directional_light(light: DirectionalLight, normal: vec3<f32>, view_direction: vec3<f32>, tex_coords: vec2<f32>) -> vec4<f32> {
    let light_direction = normalize(-light.direction);

    let diff = max(dot(normal, light_direction), 0.0);

    let reflect_direction = reflect(-light_direction, normal);
    let spec = pow(max(dot(view_direction, reflect_direction), 0.0), material.shininess);

    var ambient = light.ambient * textureSample(diffuse_texture, diffuse_sampler, tex_coords);
    var diffuse = light.diffuse * diff * textureSample(diffuse_texture, diffuse_sampler, tex_coords);
    var specular = light.specular * spec * textureSample(specular_texture, specular_sampler, tex_coords);

    return ambient + diffuse + specular;
}

fn calculate_point_light(light: PointLight, normal: vec3<f32>, frag_position: vec3<f32>, view_direction: vec3<f32>, tex_coords: vec2<f32>) -> vec4<f32> {
    let light_direction = normalize(light.position - frag_position);

    let diff = max(dot(normal, light_direction), 0.0);

    let reflect_direction = reflect(-light_direction, normal);
    let spec = pow(max(dot(view_direction, reflect_direction), 0.0), material.shininess);

    let distance = length(light.position - frag_position);
    let attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * distance * distance);

    var ambient = light.ambient * textureSample(diffuse_texture, diffuse_sampler, tex_coords);
    var diffuse = light.diffuse * diff * textureSample(diffuse_texture, diffuse_sampler, tex_coords);
    var specular = light.specular * spec * textureSample(specular_texture, specular_sampler, tex_coords);
    ambient *= attenuation;
    diffuse *= attenuation;
    specular *= attenuation;

    return ambient + diffuse + specular;
}

fn calculate_spot_light(light: SpotLight, normal: vec3<f32>, frag_position: vec3<f32>, view_direction: vec3<f32>, tex_coords: vec2<f32>) -> vec4<f32> {
    let light_direction = normalize(light.position - frag_position);

    let diff = max(dot(normal, light_direction), 0.0);

    let reflect_direction = reflect(-light_direction, normal);
    let spec = pow(max(dot(view_direction, reflect_direction), 0.0), material.shininess);

    let distance = length(light.position - frag_position);
    let attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * distance * distance);

    let theta = dot(light_direction, normalize(-light.direction));
    let epsilon = light.cut_off - light.outer_cut_off;
    let intensity = clamp((theta - light.outer_cut_off) / epsilon, 0.0, 1.0);

    var ambient = light.ambient * textureSample(diffuse_texture, diffuse_sampler, tex_coords);
    var diffuse = light.diffuse * diff * textureSample(diffuse_texture, diffuse_sampler, tex_coords);
    var specular = light.specular * spec * textureSample(specular_texture, specular_sampler, tex_coords);
    ambient *= attenuation * intensity;
    diffuse *= attenuation * intensity;
    specular *= attenuation * intensity;

    return ambient + diffuse + specular;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let normal = normalize(in.normal);
    let view_direction = normalize(camera.position - in.frag_position);

    var result = calculate_directional_light(directional_light, normal, view_direction, in.tex_coords);
    for (var i = 0; i < POINT_LIGHT_COUNT; i++) {
        result += calculate_point_light(point_lights[i], normal, in.frag_position, view_direction, in.tex_coords);
    }
    result += calculate_spot_light(spot_light, normal, in.frag_position, view_direction, in.tex_coords);

    return result;
}
