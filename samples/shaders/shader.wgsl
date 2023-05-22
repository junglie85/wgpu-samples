struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>
};

struct VsOut {
    @location(0) color: vec4<f32>,
    @builtin(position) position: vec4<f32>
};

@vertex
fn vs_main(in: VsIn) -> VsOut {
    return VsOut(
        in.color,
        vec4<f32>(in.position, 1.0)
    );
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return in.color;
}
