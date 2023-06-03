use std::{borrow::Cow, iter::once, mem::size_of, time::Instant};

use bytemuck::cast_slice;
use bytemuck_derive::{Pod, Zeroable};
use futures::executor::block_on;
use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use image::GenericImageView;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Backends, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingResource, BindingType, BufferBindingType, BufferDescriptor, BufferSize, BufferUsages,
    Color, CommandEncoderDescriptor, CompareFunction, DepthBiasState, DepthStencilState, Device,
    DeviceDescriptor, Extent3d, Face, Features, FilterMode, FragmentState, ImageDataLayout,
    IndexFormat, InstanceDescriptor, LoadOp, MultisampleState, Operations,
    PipelineLayoutDescriptor, PowerPreference, PresentMode, PrimitiveState,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    RenderPipelineDescriptor, RequestAdapterOptions, Sampler, SamplerBindingType,
    SamplerDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, StencilState,
    SurfaceConfiguration, Texture, TextureDescriptor, TextureDimension, TextureFormat,
    TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};
use wgpu_samples::{
    assets,
    camera::{Camera, CameraDescriptor, GpuCamera},
};
use winit::{
    dpi::LogicalSize,
    event::{DeviceEvent, ElementState, Event, MouseScrollDelta, VirtualKeyCode, WindowEvent},
    event_loop::EventLoop,
    platform::run_return::EventLoopExtRunReturn,
    window::{CursorGrabMode, WindowBuilder},
};

const SCREEN_WIDTH: u32 = 1280;
const SCREEN_HEIGHT: u32 = 720;
const TITLE: &str = "Light casters (point)";

const VERTICES: [Vertex; 24] = [
    // Front
    Vertex::new(
        Vec3::new(-0.5, 0.5, 0.5),
        Vec3::new(0.0, 0.0, 1.0),
        Vec2::new(0.0, 0.0),
    ), // top left
    Vertex::new(
        Vec3::new(-0.5, -0.5, 0.5),
        Vec3::new(0.0, 0.0, 1.0),
        Vec2::new(0.0, 1.0),
    ), // bottom left
    Vertex::new(
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(0.0, 0.0, 1.0),
        Vec2::new(1.0, 1.0),
    ), // bottom right
    Vertex::new(
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(0.0, 0.0, 1.0),
        Vec2::new(1.0, 0.0),
    ), // top right
    // Back
    Vertex::new(
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(0.0, 0.0, -1.0),
        Vec2::new(0.0, 0.0),
    ), // top left
    Vertex::new(
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(0.0, 0.0, -1.0),
        Vec2::new(0.0, 1.0),
    ), // bottom left
    Vertex::new(
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(0.0, 0.0, -1.0),
        Vec2::new(1.0, 1.0),
    ), // bottom right
    Vertex::new(
        Vec3::new(-0.5, 0.5, -0.5),
        Vec3::new(0.0, 0.0, -1.0),
        Vec2::new(1.0, 0.0),
    ), // top right
    // Left
    Vertex::new(
        Vec3::new(-0.5, 0.5, -0.5),
        Vec3::new(-1.0, 0.0, 0.0),
        Vec2::new(0.0, 0.0),
    ), // top left
    Vertex::new(
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(-1.0, 0.0, 0.0),
        Vec2::new(0.0, 1.0),
    ), // bottom left
    Vertex::new(
        Vec3::new(-0.5, -0.5, 0.5),
        Vec3::new(-1.0, 0.0, 0.0),
        Vec2::new(1.0, 1.0),
    ), // bottom right
    Vertex::new(
        Vec3::new(-0.5, 0.5, 0.5),
        Vec3::new(-1.0, 0.0, 0.0),
        Vec2::new(1.0, 0.0),
    ), // top right
    // Right
    Vertex::new(
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(1.0, 0.0, 0.0),
        Vec2::new(0.0, 0.0),
    ), // top left
    Vertex::new(
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(1.0, 0.0, 0.0),
        Vec2::new(0.0, 1.0),
    ), // bottom left
    Vertex::new(
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(1.0, 0.0, 0.0),
        Vec2::new(1.0, 1.0),
    ), // bottom right
    Vertex::new(
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(1.0, 0.0, 0.0),
        Vec2::new(1.0, 0.0),
    ), // top right
    // Top
    Vertex::new(
        Vec3::new(-0.5, 0.5, -0.5),
        Vec3::new(0.0, 1.0, 0.0),
        Vec2::new(0.0, 0.0),
    ), // top left
    Vertex::new(
        Vec3::new(-0.5, 0.5, 0.5),
        Vec3::new(0.0, 1.0, 0.0),
        Vec2::new(0.0, 1.0),
    ), // bottom left
    Vertex::new(
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(0.0, 1.0, 0.0),
        Vec2::new(1.0, 1.0),
    ), // bottom right
    Vertex::new(
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(0.0, 1.0, 0.0),
        Vec2::new(1.0, 0.0),
    ), // top right
    // Bottom
    Vertex::new(
        Vec3::new(-0.5, -0.5, 0.5),
        Vec3::new(0.0, -1.0, 0.0),
        Vec2::new(0.0, 0.0),
    ), // top left
    Vertex::new(
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(0.0, -1.0, 0.0),
        Vec2::new(0.0, 1.0),
    ), // bottom left
    Vertex::new(
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(0.0, -1.0, 0.0),
        Vec2::new(1.0, 1.0),
    ), // bottom right
    Vertex::new(
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(0.0, -1.0, 0.0),
        Vec2::new(1.0, 0.0),
    ), // top right
];

const INDICES: [u32; 36] = [
    0, 1, 3, 1, 2, 3, // front
    4, 5, 7, 5, 6, 7, // back
    8, 9, 11, 9, 10, 11, // left
    12, 13, 15, 13, 14, 15, // right
    16, 17, 19, 17, 18, 19, // top
    20, 21, 23, 21, 22, 23, // bottom
];

const POSITIONS: [Vec3; 10] = [
    Vec3::new(0.0, 0.0, 0.0),
    Vec3::new(2.0, 5.0, -15.0),
    Vec3::new(-1.5, -2.2, -2.5),
    Vec3::new(-3.8, -2.0, -12.3),
    Vec3::new(2.4, -0.4, -3.5),
    Vec3::new(-1.7, 3.0, -7.5),
    Vec3::new(1.3, -2.0, -2.5),
    Vec3::new(1.5, 2.0, -2.5),
    Vec3::new(1.5, 0.2, -1.5),
    Vec3::new(-1.3, 1.0, -1.5),
];

#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    const fn new(position: Vec3, normal: Vec3, tex_coords: Vec2) -> Self {
        Self {
            position: position.to_array(),
            normal: normal.to_array(),
            tex_coords: tex_coords.to_array(),
        }
    }

    fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Vertex>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: size_of::<[f32; 3]>() as u64,
                    shader_location: 1,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: size_of::<[f32; 6]>() as u64,
                    shader_location: 2,
                },
            ],
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Model {
    model_matrix: [f32; 16],
    normal_matrix: [f32; 16],
}

impl Model {
    fn new(model_matrix: Mat4) -> Self {
        let normal_matrix = model_matrix.inverse().transpose();

        Self {
            model_matrix: model_matrix.to_cols_array(),
            normal_matrix: normal_matrix.to_cols_array(),
        }
    }
}

#[derive(Debug)]
struct Material {
    diffuse_texture: Texture,
    diffuse_sampler: Sampler,
    specular_texture: Texture,
    specular_sampler: Sampler,
    shininess: f32,
}

impl Material {
    fn new(
        diffuse_texture: Texture,
        diffuse_sampler: Sampler,
        specular_texture: Texture,
        specular_sampler: Sampler,
        shininess: f32,
    ) -> Self {
        Self {
            diffuse_texture,
            diffuse_sampler,
            specular_texture,
            specular_sampler,
            shininess,
        }
    }

    fn get_gpu_material(&self) -> GpuMaterial {
        GpuMaterial {
            shininess: self.shininess,
            _pad1: 0.0,
            _pad2: 0.0,
            _pad3: 0.0,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct GpuMaterial {
    shininess: f32,
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
}

#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Light {
    ambient: [f32; 4],
    diffuse: [f32; 4],
    specular: [f32; 4],
    position: [f32; 3],
    constant: f32,
    linear: f32,
    quadratic: f32,
    _pad1: f32,
    _pad2: f32,
}

impl Light {
    fn new(
        position: Vec3,
        ambient: Vec4,
        diffuse: Vec4,
        specular: Vec4,
        constant: f32,
        linear: f32,
        quadratic: f32,
    ) -> Self {
        Self {
            position: position.to_array(),
            ambient: ambient.to_array(),
            diffuse: diffuse.to_array(),
            specular: specular.to_array(),
            constant,
            linear,
            quadratic,
            _pad1: 0.0,
            _pad2: 0.0,
        }
    }
}

fn main() {
    let mut event_loop = EventLoop::new();

    let logical_size = LogicalSize::new(SCREEN_WIDTH, SCREEN_HEIGHT);
    let window = WindowBuilder::new()
        .with_inner_size(logical_size)
        .with_title(TITLE)
        .with_visible(false)
        .build(&event_loop)
        .expect("failed to create window");
    let physical_size = window.inner_size();

    let instance = wgpu::Instance::new(InstanceDescriptor {
        backends: Backends::PRIMARY,
        ..Default::default()
    });

    let surface = unsafe {
        instance
            .create_surface(&window)
            .expect("failed to create surface")
    };

    let adapter = block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::LowPower,
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    }))
    .expect("failed to get a suitable adapter");

    let (device, queue) = block_on(adapter.request_device(
        &DeviceDescriptor {
            label: Some("device"),
            features: Features::empty(),
            limits: adapter.limits(),
        },
        None,
    ))
    .expect("failed to get a device");

    let surface_capabilities = surface.get_capabilities(&adapter);
    let surface_format = if surface_capabilities
        .formats
        .contains(&TextureFormat::Rgba8Unorm)
    {
        TextureFormat::Rgba8Unorm
    } else if surface_capabilities
        .formats
        .contains(&TextureFormat::Bgra8Unorm)
    {
        TextureFormat::Bgra8Unorm
    } else {
        surface_capabilities.formats[0]
    };

    let mut surface_config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: physical_size.width,
        height: physical_size.height,
        present_mode: PresentMode::Fifo,
        alpha_mode: surface_capabilities.alpha_modes[0],
        view_formats: Vec::new(),
    };

    surface.configure(&device, &surface_config);

    let (mut depth_texture, mut depth_texture_view) =
        create_depth_texture(&device, physical_size.width, physical_size.height);

    let scene_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("bind_group_layout::scene"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(size_of::<GpuCamera>() as u64),
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(size_of::<Light>() as u64),
                },
                count: None,
            },
        ],
    });

    let model_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("bind_group_layout::model"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: BufferSize::new(size_of::<Model>() as u64),
            },
            count: None,
        }],
    });

    let material_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("bind_group_layout::material"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(size_of::<GpuMaterial>() as u64),
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 4,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
        ],
    });

    let light_cube_color_bind_group_layout =
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("bind_group_layout::light_cube_color"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(size_of::<[f32; 4]>() as u64),
                },
                count: None,
            }],
        });

    let light_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("pipeline_layout::light"),
        bind_group_layouts: &[
            &scene_bind_group_layout,
            &model_bind_group_layout,
            &light_cube_color_bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

    let model_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("pipeline_layout::model"),
        bind_group_layouts: &[
            &scene_bind_group_layout,
            &model_bind_group_layout,
            &material_bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

    let camera_ubo = device.create_buffer(&BufferDescriptor {
        label: Some("ubo::camera"),
        size: size_of::<GpuCamera>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let lighting_ubo = device.create_buffer(&BufferDescriptor {
        label: Some("ubo::lighting"),
        size: size_of::<Light>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let scene_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("bind_group::scene"),
        layout: &scene_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: camera_ubo.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: lighting_ubo.as_entire_binding(),
            },
        ],
    });

    let light_shader_src = include_str!("light.wgsl");
    let light_shader_module = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("shader_module::light"),
        source: ShaderSource::Wgsl(Cow::Borrowed(light_shader_src)),
    });

    let light_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("render_pipeline::light"),
        layout: Some(&light_pipeline_layout),
        vertex: VertexState {
            module: &light_shader_module,
            entry_point: "vs_main",
            buffers: &[Vertex::layout()],
        },
        primitive: PrimitiveState {
            cull_mode: Some(Face::Back),
            ..Default::default()
        },
        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Less,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        }),
        multisample: MultisampleState::default(),
        fragment: Some(FragmentState {
            module: &light_shader_module,
            entry_point: "fs_main",
            targets: &[Some(surface_format.into())],
        }),
        multiview: None,
    });

    let model_shader_src = include_str!("model.wgsl");
    let model_shader_module = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("shader_module::model"),
        source: ShaderSource::Wgsl(Cow::Borrowed(model_shader_src)),
    });

    let model_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("render_pipeline::model"),
        layout: Some(&model_pipeline_layout),
        vertex: VertexState {
            module: &model_shader_module,
            entry_point: "vs_main",
            buffers: &[Vertex::layout()],
        },
        primitive: PrimitiveState {
            cull_mode: Some(Face::Back),
            ..Default::default()
        },
        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Less,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        }),
        multisample: MultisampleState::default(),
        fragment: Some(FragmentState {
            module: &model_shader_module,
            entry_point: "fs_main",
            targets: &[Some(surface_format.into())],
        }),
        multiview: None,
    });

    let light_cube_vbo = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("vbo::light_cube"),
        contents: cast_slice(&VERTICES),
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
    });

    let light_cube_ibo = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("ibo::light_cube"),
        contents: cast_slice(&INDICES),
        usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
    });

    let cube_vbo = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("vbo::cube"),
        contents: cast_slice(&VERTICES),
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
    });

    let cube_ibo = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("ibo::cube"),
        contents: cast_slice(&INDICES),
        usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
    });

    let light_position = Vec3::new(1.2, 1.0, 2.0);
    let light_transform = Mat4::from_scale_rotation_translation(
        Vec3::new(0.2, 0.2, 0.2),
        Quat::IDENTITY,
        light_position,
    );
    let light_cube = Model::new(light_transform);

    let light_cube_ubo = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("ubo::light_cube"),
        contents: cast_slice(&[light_cube]),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });

    let light_cube_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("bind_group::light_cube"),
        layout: &model_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: light_cube_ubo.as_entire_binding(),
        }],
    });

    let light_cube_color = Vec4::new(1.0, 1.0, 1.0, 1.0);
    let light_cube_color_ubo = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("ubo::light_color"),
        contents: cast_slice(&light_cube_color.to_array()),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });

    let light_cube_color_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("bind_group::light_cube_color"),
        layout: &light_cube_color_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: light_cube_color_ubo.as_entire_binding(),
        }],
    });

    let models = POSITIONS
        .iter()
        .enumerate()
        .map(|(i, position)| {
            let angle = 20.0_f32 * i as f32;
            let rotation =
                Quat::from_axis_angle(Vec3::new(1.0, 0.3, 0.5).normalize(), angle.to_radians());
            let cube_transform = Mat4::from_rotation_translation(rotation, *position);
            Model::new(cube_transform)
        })
        .collect::<Vec<_>>();

    let cube_ubos = models
        .iter()
        .enumerate()
        .map(|(i, cube)| {
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some(&format!("vbo::cube_{}", i)),
                contents: cast_slice(&[*cube]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        })
        .collect::<Vec<_>>();

    let cube_bind_groups = cube_ubos
        .iter()
        .enumerate()
        .map(|(i, ubo)| {
            device.create_bind_group(&BindGroupDescriptor {
                label: Some(&format!("bind_group::cube_{}", i)),
                layout: &model_bind_group_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: ubo.as_entire_binding(),
                }],
            })
        })
        .collect::<Vec<_>>();

    let diffuse_bytes = assets::load("assets/container2.png").expect("unable to open file");
    let diffuse_image = image::load_from_memory(&diffuse_bytes).expect("unable to load image");
    let diffuse_image_data = diffuse_image.to_rgba8();
    let diffuse_image_size = diffuse_image.dimensions();

    let diffuse_texture_size = Extent3d {
        width: diffuse_image_size.0,
        height: diffuse_image_size.1,
        depth_or_array_layers: 1,
    };

    let diffuse_texture = device.create_texture(&TextureDescriptor {
        label: Some("texture::diffuse::cube"),
        size: diffuse_texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        diffuse_texture.as_image_copy(),
        &diffuse_image_data,
        ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * diffuse_image_size.0),
            rows_per_image: Some(diffuse_image_size.1),
        },
        diffuse_texture_size,
    );

    let diffuse_sampler = device.create_sampler(&SamplerDescriptor {
        label: Some("sampler::diffuse::cube"),
        mag_filter: FilterMode::Nearest,
        min_filter: FilterMode::Linear,
        ..Default::default()
    });

    let specular_bytes =
        assets::load("assets/container2_specular.png").expect("unable to open file");
    let specular_image = image::load_from_memory(&specular_bytes).expect("unable to load image");
    let specular_image_data = specular_image.to_rgba8();
    let specular_image_size = specular_image.dimensions();

    let specular_texture_size = Extent3d {
        width: specular_image_size.0,
        height: specular_image_size.1,
        depth_or_array_layers: 1,
    };

    let specular_texture = device.create_texture(&TextureDescriptor {
        label: Some("texture::specular::cube"),
        size: specular_texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        specular_texture.as_image_copy(),
        &specular_image_data,
        ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * specular_image_size.0),
            rows_per_image: Some(specular_image_size.1),
        },
        specular_texture_size,
    );

    let specular_sampler = device.create_sampler(&SamplerDescriptor {
        label: Some("sampler::specular::cube"),
        mag_filter: FilterMode::Nearest,
        min_filter: FilterMode::Linear,
        ..Default::default()
    });

    let cube_material = Material::new(
        diffuse_texture,
        diffuse_sampler,
        specular_texture,
        specular_sampler,
        32.0,
    );

    let cube_material_ubo = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("ubo::cube_material"),
        contents: cast_slice(&[cube_material.get_gpu_material()]),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });

    let cube_material_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("bind_group::cube_material"),
        layout: &material_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: cube_material_ubo.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&cube_material.diffuse_sampler),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::TextureView(
                    &cube_material
                        .diffuse_texture
                        .create_view(&TextureViewDescriptor::default()),
                ),
            },
            BindGroupEntry {
                binding: 3,
                resource: BindingResource::Sampler(&cube_material.specular_sampler),
            },
            BindGroupEntry {
                binding: 4,
                resource: BindingResource::TextureView(
                    &cube_material
                        .specular_texture
                        .create_view(&TextureViewDescriptor::default()),
                ),
            },
        ],
    });

    let light = Light::new(
        light_position,
        Vec4::new(0.2, 0.2, 0.2, 1.0),
        Vec4::new(0.5, 0.5, 0.5, 1.0),
        Vec4::new(1.0, 1.0, 1.0, 1.0),
        1.0,
        0.09,
        0.032,
    );

    let mut camera = Camera::new(&CameraDescriptor {
        aspect_ratio: SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32,
        ..Default::default()
    });

    window.set_cursor_visible(false);
    window
        .set_cursor_grab(CursorGrabMode::Confined)
        .expect("failed to grab cursor");
    window.set_visible(true);

    let mut last_time = Instant::now();

    let mut mouse_in_window = false;
    let mut running = true;
    while running {
        let current_time = Instant::now();
        let dt = (current_time - last_time).as_secs_f32();
        last_time = current_time;

        event_loop.run_return(|event, _, control_flow| {
            control_flow.set_wait();

            match event {
                Event::WindowEvent { window_id, event } if window.id() == window_id => {
                    match event {
                        WindowEvent::CloseRequested => running = false,

                        WindowEvent::Resized(size) => {
                            surface_config.width = size.width;
                            surface_config.height = size.height;
                            surface.configure(&device, &surface_config);

                            (depth_texture, depth_texture_view) = create_depth_texture(
                                &device,
                                surface_config.width,
                                surface_config.height,
                            );
                        }

                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            surface_config.width = new_inner_size.width;
                            surface_config.height = new_inner_size.height;
                            surface.configure(&device, &surface_config);

                            (depth_texture, depth_texture_view) = create_depth_texture(
                                &device,
                                surface_config.width,
                                surface_config.height,
                            );
                        }

                        WindowEvent::CursorEntered { .. } => {
                            mouse_in_window = true;
                        }

                        WindowEvent::MouseWheel { delta, .. } => {
                            if let MouseScrollDelta::LineDelta(_, y) = delta {
                                camera.zoom(y);
                            }
                        }

                        WindowEvent::KeyboardInput { input, .. } => {
                            if let Some(key) = input.virtual_keycode {
                                match key {
                                    VirtualKeyCode::Escape
                                        if input.state == ElementState::Pressed =>
                                    {
                                        running = false;
                                    }
                                    VirtualKeyCode::W if input.state == ElementState::Pressed => {
                                        camera.move_forward(dt);
                                    }
                                    VirtualKeyCode::S if input.state == ElementState::Pressed => {
                                        camera.move_backward(dt);
                                    }
                                    VirtualKeyCode::A if input.state == ElementState::Pressed => {
                                        camera.skew_left(dt);
                                    }
                                    VirtualKeyCode::D if input.state == ElementState::Pressed => {
                                        camera.skew_right(dt);
                                    }

                                    _ => (),
                                }
                            }
                        }

                        _ => (),
                    }
                }

                Event::DeviceEvent { event, .. } if mouse_in_window => match event {
                    DeviceEvent::MouseMotion { delta } => {
                        let (x, y) = delta;

                        camera.yaw_pitch(x as f32, -y as f32);
                    }

                    _ => (),
                },

                Event::MainEventsCleared => control_flow.set_exit(),

                _ => (),
            }
        });

        queue.write_buffer(&camera_ubo, 0, cast_slice(&[camera.get_gpu_camera()]));
        queue.write_buffer(&lighting_ubo, 0, cast_slice(&[light]));

        let frame = surface
            .get_current_texture()
            .expect("failed to get current swapchain texture");
        let output_texture_view = frame.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("command_encoder"),
        });

        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &output_texture_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &depth_texture_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: Some(Operations {
                        load: LoadOp::Clear(0),
                        store: true,
                    }),
                }),
            });

            rpass.set_bind_group(0, &scene_bind_group, &[]);

            rpass.set_pipeline(&light_pipeline);
            rpass.set_bind_group(1, &light_cube_bind_group, &[]);
            rpass.set_bind_group(2, &light_cube_color_bind_group, &[]);
            rpass.set_vertex_buffer(0, light_cube_vbo.slice(..));
            rpass.set_index_buffer(light_cube_ibo.slice(..), IndexFormat::Uint32);
            rpass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);

            rpass.set_pipeline(&model_pipeline);
            for bind_group in &cube_bind_groups {
                rpass.set_bind_group(1, &bind_group, &[]);
                rpass.set_bind_group(2, &cube_material_bind_group, &[]);
                rpass.set_vertex_buffer(0, cube_vbo.slice(..));
                rpass.set_index_buffer(cube_ibo.slice(..), IndexFormat::Uint32);
                rpass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
            }
        }

        queue.submit(once(encoder.finish()));
        frame.present();
    }
}

fn create_depth_texture(device: &Device, width: u32, height: u32) -> (Texture, TextureView) {
    let texture = device.create_texture(&TextureDescriptor {
        label: Some("texture::depth"),
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Depth32Float,
        usage: TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    let texture_view = texture.create_view(&TextureViewDescriptor::default());

    (texture, texture_view)
}
