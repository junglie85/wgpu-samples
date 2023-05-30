use std::{borrow::Cow, iter::once, mem::size_of, time::Instant};

use bytemuck::cast_slice;
use bytemuck_derive::{Pod, Zeroable};
use futures::executor::block_on;
use glam::{Mat4, Quat, Vec3, Vec4};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Backends, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, BufferBindingType, BufferDescriptor, BufferSize, BufferUsages, Color,
    CommandEncoderDescriptor, CompareFunction, DepthBiasState, DepthStencilState, Device,
    DeviceDescriptor, Extent3d, Face, Features, FragmentState, IndexFormat, InstanceDescriptor,
    LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor, PowerPreference, PresentMode,
    PrimitiveState, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor,
    ShaderSource, ShaderStages, StencilState, SurfaceConfiguration, Texture, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};
use wgpu_samples::camera::{Camera, CameraDescriptor, GpuCamera};
use winit::{
    dpi::LogicalSize,
    event::{DeviceEvent, ElementState, Event, MouseScrollDelta, VirtualKeyCode, WindowEvent},
    event_loop::EventLoop,
    platform::run_return::EventLoopExtRunReturn,
    window::{CursorGrabMode, WindowBuilder},
};

const SCREEN_WIDTH: u32 = 1280;
const SCREEN_HEIGHT: u32 = 720;
const TITLE: &str = "Colors";

const VERTICES: [Vertex; 24] = [
    // Front
    Vertex::new(Vec3::new(-0.5, 0.5, 0.5)),  // top left
    Vertex::new(Vec3::new(-0.5, -0.5, 0.5)), // bottom left
    Vertex::new(Vec3::new(0.5, -0.5, 0.5)),  // bottom right
    Vertex::new(Vec3::new(0.5, 0.5, 0.5)),   // top right
    // Back
    Vertex::new(Vec3::new(0.5, 0.5, -0.5)),   // top left
    Vertex::new(Vec3::new(0.5, -0.5, -0.5)),  // bottom left
    Vertex::new(Vec3::new(-0.5, -0.5, -0.5)), // bottom right
    Vertex::new(Vec3::new(-0.5, 0.5, -0.5)),  // top right
    // Left
    Vertex::new(Vec3::new(-0.5, 0.5, -0.5)),  // top left
    Vertex::new(Vec3::new(-0.5, -0.5, -0.5)), // bottom left
    Vertex::new(Vec3::new(-0.5, -0.5, 0.5)),  // bottom right
    Vertex::new(Vec3::new(-0.5, 0.5, 0.5)),   // top right
    // Right
    Vertex::new(Vec3::new(0.5, 0.5, 0.5)),   // top left
    Vertex::new(Vec3::new(0.5, -0.5, 0.5)),  // bottom left
    Vertex::new(Vec3::new(0.5, -0.5, -0.5)), // bottom right
    Vertex::new(Vec3::new(0.5, 0.5, -0.5)),  // top right
    // Top
    Vertex::new(Vec3::new(-0.5, 0.5, -0.5)), // top left
    Vertex::new(Vec3::new(-0.5, 0.5, 0.5)),  // bottom left
    Vertex::new(Vec3::new(0.5, 0.5, 0.5)),   // bottom right
    Vertex::new(Vec3::new(0.5, 0.5, -0.5)),  // top right
    // Bottom
    Vertex::new(Vec3::new(-0.5, -0.5, 0.5)),  // top left
    Vertex::new(Vec3::new(-0.5, -0.5, -0.5)), // bottom left
    Vertex::new(Vec3::new(0.5, -0.5, -0.5)),  // bottom right
    Vertex::new(Vec3::new(0.5, -0.5, 0.5)),   // top right
];

const INDICES: [u32; 36] = [
    0, 1, 3, 1, 2, 3, // front
    4, 5, 7, 5, 6, 7, // back
    8, 9, 11, 9, 10, 11, // left
    12, 13, 15, 13, 14, 15, // right
    16, 17, 19, 17, 18, 19, // top
    20, 21, 23, 21, 22, 23, // bottom
];

#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    position: [f32; 3],
}

impl Vertex {
    const fn new(position: Vec3) -> Self {
        Self {
            position: position.to_array(),
        }
    }

    fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Vertex>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: &[VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            }],
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Model {
    model_matrix: [f32; 16],
    color: [f32; 4],
}

impl Model {
    fn new(model_matrix: Mat4, color: Vec4) -> Self {
        Self {
            model_matrix: model_matrix.to_cols_array(),
            color: color.to_array(),
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
                visibility: ShaderStages::VERTEX,
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
                    min_binding_size: BufferSize::new(size_of::<[f32; 4]>() as u64),
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

    let light_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("pipeline_layout::light"),
        bind_group_layouts: &[&scene_bind_group_layout, &model_bind_group_layout],
        push_constant_ranges: &[],
    });

    let model_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("pipeline_layout::model"),
        bind_group_layouts: &[&scene_bind_group_layout, &model_bind_group_layout],
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
        size: size_of::<[f32; 4]>() as u64,
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

    let light_transform = Mat4::from_scale_rotation_translation(
        Vec3::new(0.2, 0.2, 0.2),
        Quat::IDENTITY,
        Vec3::new(1.2, 1.0, 2.0),
    );
    let light_cube = Model::new(light_transform, Vec4::new(1.0, 1.0, 1.0, 1.0));

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

    let cube_transform = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
    let cube = Model::new(cube_transform, Vec4::new(1.0, 0.5, 0.31, 1.0));

    let cube_ubo = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("vbo::cube"),
        contents: cast_slice(&[cube]),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });

    let cube_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("bind_group::cube"),
        layout: &model_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: cube_ubo.as_entire_binding(),
        }],
    });

    let light_color = Vec4::new(1.0, 1.0, 1.0, 1.0);

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
        queue.write_buffer(&lighting_ubo, 0, cast_slice(&light_color.to_array()));

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
            rpass.set_vertex_buffer(0, light_cube_vbo.slice(..));
            rpass.set_index_buffer(light_cube_ibo.slice(..), IndexFormat::Uint32);
            rpass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);

            rpass.set_pipeline(&model_pipeline);
            rpass.set_bind_group(1, &cube_bind_group, &[]);
            rpass.set_vertex_buffer(0, cube_vbo.slice(..));
            rpass.set_index_buffer(cube_ibo.slice(..), IndexFormat::Uint32);
            rpass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
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
