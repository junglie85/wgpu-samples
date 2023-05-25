use std::{borrow::Cow, iter::once, mem::size_of, time::Instant};

use bytemuck::cast_slice;
use bytemuck_derive::{Pod, Zeroable};
use futures::executor::block_on;
use glam::{Mat4, Quat, Vec2, Vec3};
use image::GenericImageView;
use wgpu::{
    AddressMode, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BufferBindingType, BufferDescriptor,
    BufferSize, BufferUsages, Color, CommandEncoderDescriptor, CompareFunction, DepthBiasState,
    DepthStencilState, Device, DeviceDescriptor, Extent3d, Face, Features, FilterMode,
    FragmentState, ImageDataLayout, IndexFormat, Instance, LoadOp, MultisampleState, Operations,
    PipelineLayoutDescriptor, PresentMode, PrimitiveState, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
    RequestAdapterOptions, SamplerBindingType, SamplerDescriptor, ShaderModuleDescriptor,
    ShaderSource, ShaderStages, StencilState, SurfaceConfiguration, Texture, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView,
    TextureViewDescriptor, TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexState, VertexStepMode,
};
use wgpu_samples::assets;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};

#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    position: [f32; 3],
    tex_coord: [f32; 2],
}

impl Vertex {
    fn new(position: Vec3, tex_coord: Vec2) -> Self {
        Self {
            position: position.to_array(),
            tex_coord: tex_coord.to_array(),
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
                    format: VertexFormat::Float32x2,
                    offset: size_of::<[f32; 3]>() as u64,
                    shader_location: 1,
                },
            ],
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Globals {
    view: [f32; 16],
    projection: [f32; 16],
}

impl Globals {
    fn new(view: Mat4, projection: Mat4) -> Self {
        Self {
            view: view.to_cols_array(),
            projection: projection.to_cols_array(),
        }
    }
}

fn main() {
    let mut event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(1280, 720))
        .with_title("Coordinate systems")
        .with_visible(false)
        .build(&event_loop)
        .expect("failed to create a window");

    let physical_size = window.inner_size();

    let instance = Instance::default();
    let surface = unsafe { instance.create_surface(&window) }.expect("failed to create surface");
    let adapter = block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    }))
    .expect("failed to find a suitable adapter");

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchian_format = if swapchain_capabilities
        .formats
        .contains(&TextureFormat::Bgra8Unorm)
    {
        TextureFormat::Bgra8Unorm
    } else if swapchain_capabilities
        .formats
        .contains(&TextureFormat::Rgba8Unorm)
    {
        TextureFormat::Rgba8Unorm
    } else {
        swapchain_capabilities.formats[0]
    };

    let (device, queue) = block_on(adapter.request_device(
        &DeviceDescriptor {
            label: None,
            features: Features::empty(),
            limits: adapter.limits(),
        },
        None,
    ))
    .expect("failed to create a device");

    let shader_src = include_str!("shader.wgsl");
    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(Cow::Borrowed(shader_src)),
    });

    let global_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("bind_group_layout::global"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: BufferSize::new(size_of::<Globals>() as u64),
            },
            count: None,
        }],
    });

    let model_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("bind_group_layout::model"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(size_of::<[f32; 16]>() as u64),
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
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&global_bind_group_layout, &model_bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: &shader,
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
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(swapchian_format.into())],
        }),
        multiview: None,
    });

    let (mut depth_texture, mut depth_texture_view) =
        create_depth_texture(&device, physical_size.width, physical_size.height);

    let sampler = device.create_sampler(&SamplerDescriptor {
        label: None,
        address_mode_u: AddressMode::Repeat,
        address_mode_v: AddressMode::Repeat,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        ..Default::default()
    });

    let mut config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: swapchian_format,
        width: physical_size.width,
        height: physical_size.height,
        present_mode: PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: Vec::new(),
    };

    surface.configure(&device, &config);

    let globals_ubo = device.create_buffer(&BufferDescriptor {
        label: Some("ubo::globals"),
        size: size_of::<Globals>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let globals_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("bind_group::globals"),
        layout: &global_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: globals_ubo.as_entire_binding(),
        }],
    });

    let model_ubo = device.create_buffer(&BufferDescriptor {
        label: Some("ubo::model"),
        size: size_of::<[f32; 16]>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let vertices = [
        // Front
        Vertex::new(Vec3::new(-0.5, 0.5, 0.5), Vec2::new(0.0, 0.0)), // top left
        Vertex::new(Vec3::new(-0.5, -0.5, 0.5), Vec2::new(0.0, 1.0)), // bottom left
        Vertex::new(Vec3::new(0.5, -0.5, 0.5), Vec2::new(1.0, 1.0)), // bottom right
        Vertex::new(Vec3::new(0.5, 0.5, 0.5), Vec2::new(1.0, 0.0)),  // top right
        // Back
        Vertex::new(Vec3::new(0.5, 0.5, -0.5), Vec2::new(0.0, 0.0)), // top left
        Vertex::new(Vec3::new(0.5, -0.5, -0.5), Vec2::new(0.0, 1.0)), // bottom left
        Vertex::new(Vec3::new(-0.5, -0.5, -0.5), Vec2::new(1.0, 1.0)), // bottom right
        Vertex::new(Vec3::new(-0.5, 0.5, -0.5), Vec2::new(1.0, 0.0)), // top right
        // Left
        Vertex::new(Vec3::new(-0.5, 0.5, -0.5), Vec2::new(0.0, 0.0)), // top left
        Vertex::new(Vec3::new(-0.5, -0.5, -0.5), Vec2::new(0.0, 1.0)), // bottom left
        Vertex::new(Vec3::new(-0.5, -0.5, 0.5), Vec2::new(1.0, 1.0)), // bottom right
        Vertex::new(Vec3::new(-0.5, 0.5, 0.5), Vec2::new(1.0, 0.0)),  // top right
        // Right
        Vertex::new(Vec3::new(0.5, 0.5, 0.5), Vec2::new(0.0, 0.0)), // top left
        Vertex::new(Vec3::new(0.5, -0.5, 0.5), Vec2::new(0.0, 1.0)), // bottom left
        Vertex::new(Vec3::new(0.5, -0.5, -0.5), Vec2::new(1.0, 1.0)), // bottom right
        Vertex::new(Vec3::new(0.5, 0.5, -0.5), Vec2::new(1.0, 0.0)), // top right
        // Top
        Vertex::new(Vec3::new(-0.5, 0.5, -0.5), Vec2::new(0.0, 0.0)), // top left
        Vertex::new(Vec3::new(-0.5, 0.5, 0.5), Vec2::new(0.0, 1.0)),  // bottom left
        Vertex::new(Vec3::new(0.5, 0.5, 0.5), Vec2::new(1.0, 1.0)),   // bottom right
        Vertex::new(Vec3::new(0.5, 0.5, -0.5), Vec2::new(1.0, 0.0)),  // top right
        // Bottom
        Vertex::new(Vec3::new(-0.5, -0.5, 0.5), Vec2::new(0.0, 0.0)), // top left
        Vertex::new(Vec3::new(-0.5, -0.5, -0.5), Vec2::new(0.0, 1.0)), // bottom left
        Vertex::new(Vec3::new(0.5, -0.5, -0.5), Vec2::new(1.0, 1.0)), // bottom right
        Vertex::new(Vec3::new(0.5, -0.5, 0.5), Vec2::new(1.0, 0.0)),  // top right
    ];

    let indices = [
        0_u32, 1, 3, 1, 2, 3, // front
        4_u32, 5, 7, 5, 6, 7, // back
        8_u32, 9, 11, 9, 10, 11, // left
        12_u32, 13, 15, 13, 14, 15, // right
        16_u32, 17, 19, 17, 18, 19, // top
        20_u32, 21, 23, 21, 22, 23, // bottom
    ];

    let vbo = device.create_buffer(&BufferDescriptor {
        label: None,
        size: size_of::<Vertex>() as u64 * vertices.len() as u64,
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let ibo = device.create_buffer(&BufferDescriptor {
        label: None,
        size: size_of::<u32>() as u64 * indices.len() as u64,
        usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let container_bytes = assets::load("assets/container.jpg").expect("unable to open file");
    let container_image = image::load_from_memory(&container_bytes).expect("unable to load image");
    let container_image_data = container_image.to_rgba8();
    let container_image_size = container_image.dimensions();

    let container_texture_size = Extent3d {
        width: container_image_size.0,
        height: container_image_size.1,
        depth_or_array_layers: 1,
    };

    let container_texture = device.create_texture(&TextureDescriptor {
        label: Some("texture::container"),
        size: container_texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let container_texture_view = container_texture.create_view(&TextureViewDescriptor::default());

    let face_bytes = assets::load("assets/awesomeface.png").expect("unable to open file");
    let face_image = image::load_from_memory(&face_bytes).expect("unable to load image");
    let face_image_data = face_image.to_rgba8();
    let face_image_size = face_image.dimensions();

    let face_texture_size = Extent3d {
        width: container_image_size.0,
        height: container_image_size.1,
        depth_or_array_layers: 1,
    };

    let face_texture = device.create_texture(&TextureDescriptor {
        label: Some("texture::face"),
        size: face_texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let face_texture_view = face_texture.create_view(&TextureViewDescriptor::default());

    let model_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("bind_group::model"),
        layout: &model_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: model_ubo.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&sampler),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::TextureView(&container_texture_view),
            },
            BindGroupEntry {
                binding: 3,
                resource: BindingResource::TextureView(&face_texture_view),
            },
        ],
    });

    queue.write_buffer(&vbo, 0, cast_slice(&vertices));
    queue.write_buffer(&ibo, 0, cast_slice(&indices));
    queue.write_texture(
        container_texture.as_image_copy(),
        &container_image_data,
        ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * container_image_size.0),
            rows_per_image: Some(container_image_size.1),
        },
        container_texture_size,
    );
    queue.write_texture(
        face_texture.as_image_copy(),
        &face_image_data,
        ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * face_image_size.0),
            rows_per_image: Some(face_image_size.1),
        },
        face_texture_size,
    );

    let view = Mat4::from_translation(Vec3::new(0.0, 0.0, -3.0));
    let projection = Mat4::perspective_rh(45.0_f32.to_radians(), 1280.0 / 720.0, 0.1, 100.0);
    let globals = Globals::new(view, projection);

    window.set_visible(true);
    let mut running = true;
    let start_time = Instant::now();
    while running {
        event_loop.run_return(|event, _, control_flow| {
            control_flow.set_wait();

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => running = false,

                    WindowEvent::Resized(size) => {
                        config.width = size.width;
                        config.height = size.height;
                        surface.configure(&device, &config);

                        (depth_texture, depth_texture_view) =
                            create_depth_texture(&device, config.width, config.height);
                    }

                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        config.width = new_inner_size.width;
                        config.height = new_inner_size.height;
                        surface.configure(&device, &config);

                        (depth_texture, depth_texture_view) =
                            create_depth_texture(&device, config.width, config.height);
                    }

                    _ => (),
                },

                Event::MainEventsCleared => control_flow.set_exit(),

                _ => (),
            }
        });

        let elapsed = Instant::now() - start_time;
        let model = Mat4::from_quat(Quat::from_axis_angle(
            Vec3::new(0.5, 1.0, 0.0).normalize(),
            elapsed.as_secs_f32() * 50.0_f32.to_radians(),
        ));

        queue.write_buffer(&globals_ubo, 0, cast_slice(&[globals]));
        queue.write_buffer(&model_ubo, 0, cast_slice(&model.to_cols_array()));

        let frame = surface
            .get_current_texture()
            .expect("failed to get next swapchain texture");
        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });

        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.2,
                            g: 0.3,
                            b: 0.3,
                            a: 1.0,
                        }),
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

            rpass.set_bind_group(0, &globals_bind_group, &[]);
            rpass.set_pipeline(&render_pipeline);
            rpass.set_bind_group(1, &model_bind_group, &[]);
            rpass.set_vertex_buffer(0, vbo.slice(..));
            rpass.set_index_buffer(ibo.slice(..), IndexFormat::Uint32);
            rpass.draw_indexed(0..indices.len() as u32, 0, 0..1);
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
