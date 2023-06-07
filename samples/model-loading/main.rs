// todo: assimp stuff.

use std::{borrow::Cow, iter::once, mem::size_of, path::Path, time::Instant};

use bytemuck::cast_slice;
use bytemuck_derive::{Pod, Zeroable};
use futures::executor::block_on;
use glam::{Mat4, Quat, Vec2, Vec3};
use image::GenericImageView;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Adapter, Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
    BufferBindingType, BufferDescriptor, BufferSize, BufferUsages, Color, CommandEncoderDescriptor,
    CompareFunction, DepthBiasState, DepthStencilState, Device, DeviceDescriptor, Extent3d, Face,
    FilterMode, FragmentState, FrontFace, ImageDataLayout, IndexFormat, Instance,
    InstanceDescriptor, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor,
    PowerPreference, PresentMode, PrimitiveState, Queue, RenderPass, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
    RequestAdapterOptions, Sampler, SamplerBindingType, SamplerDescriptor, ShaderModuleDescriptor,
    ShaderSource, ShaderStages, StencilState, Surface, SurfaceConfiguration, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView,
    TextureViewDescriptor, TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexState, VertexStepMode,
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
    window::{CursorGrabMode, Window, WindowBuilder},
};

const SCREEN_WIDTH: u32 = 1280;
const SCREEN_HEIGHT: u32 = 720;
const TITLE: &'static str = "Model loading";

#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    position: Vec3,
    tex_coords: Vec2,
}

impl Vertex {
    fn new(position: Vec3, tex_coords: Vec2) -> Self {
        Self {
            position,
            tex_coords,
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
struct Transform {
    model_matrix: Mat4,
    normal_matrix: Mat4,
}

impl Transform {
    fn new(model_matrix: Mat4) -> Self {
        let normal_matrix = model_matrix.inverse().transpose();

        Self {
            model_matrix,
            normal_matrix,
        }
    }
}

struct Texture {
    _extent: Extent3d,
    _texture: wgpu::Texture,
    texture_view: TextureView,
    sampler: Sampler,
}

impl Texture {
    fn from_bytes(bytes: &[u8], device: &Device, queue: &Queue, label: Option<&str>) -> Self {
        let image = image::load_from_memory(&bytes).expect("unable to load image from bytes");
        let image_data = image.to_rgba8();
        let image_size = image.dimensions();

        let extent = Extent3d {
            width: image_size.0,
            height: image_size.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            label,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&TextureViewDescriptor::default());

        let sampler = device.create_sampler(&SamplerDescriptor {
            label,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Linear,
            ..Default::default()
        });

        queue.write_texture(
            texture.as_image_copy(),
            &image_data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * image_size.0),
                rows_per_image: Some(image_size.1),
            },
            extent,
        );

        Self {
            _extent: extent,
            _texture: texture,
            texture_view,
            sampler,
        }
    }
}

struct Material {
    _diffuse_texture: Texture,
    bind_group: BindGroup,
}

impl Material {
    fn new(
        device: &Device,
        layout: &BindGroupLayout,
        diffuse_texture: Texture,
        label: Option<&str>,
    ) -> Self {
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label,
            layout: &layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Sampler(&diffuse_texture.sampler),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&diffuse_texture.texture_view),
                },
            ],
        });

        Self {
            _diffuse_texture: diffuse_texture,
            bind_group,
        }
    }
}

struct Mesh {
    _vertices: Vec<Vertex>,
    indices: Vec<u32>,
    material_id: usize,
    vbo: Buffer,
    ibo: Buffer,
}

impl Mesh {
    fn new(
        device: &Device,
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        material_id: usize,
        label: Option<&str>,
    ) -> Self {
        let vbo = device.create_buffer_init(&BufferInitDescriptor {
            label,
            contents: cast_slice(&vertices),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let ibo = device.create_buffer_init(&BufferInitDescriptor {
            label,
            contents: cast_slice(&indices),
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
        });

        Self {
            _vertices: vertices,
            indices,
            material_id,
            vbo,
            ibo,
        }
    }
}

struct Model {
    materials: Vec<Material>,
    meshes: Vec<Mesh>,
}

impl Model {
    fn load<P>(path: P, device: &Device, queue: &Queue, layout: &BindGroupLayout) -> Self
    where
        P: AsRef<Path>,
    {
        let mut materials = Vec::new();
        let mut meshes = Vec::new();

        let vertices = vec![
            Vertex::new(Vec3::new(-0.5, 0.5, 0.0), Vec2::new(0.0, 0.0)),
            Vertex::new(Vec3::new(-0.5, -0.5, 0.0), Vec2::new(0.0, 1.0)),
            Vertex::new(Vec3::new(0.5, -0.5, 0.0), Vec2::new(1.0, 1.0)),
            Vertex::new(Vec3::new(0.5, 0.5, 0.0), Vec2::new(1.0, 0.0)),
        ];

        let indices = vec![0, 1, 2, 0, 2, 3];

        let label = path.as_ref().to_str();
        let texture_bytes = assets::load(path.as_ref()).expect("failed to load asset");
        let texture = Texture::from_bytes(&texture_bytes, &device, &queue, label);
        let material = Material::new(&device, &layout, texture, label);

        let material_id = materials.len();
        materials.push(material);

        let mesh = Mesh::new(&device, vertices, indices, material_id, label);
        meshes.push(mesh);

        Self { materials, meshes }
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
        .expect("failed to create a window");
    let physical_size = window.inner_size();

    let (_instance, adapter, device, queue, surface) = setup_gpu(&window);
    let mut surface_config = setup_surface(
        &surface,
        physical_size.width,
        physical_size.height,
        &adapter,
        &device,
    );

    let global_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("bind_group_layout::global"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: BufferSize::new(size_of::<GpuCamera>() as u64),
            },
            count: None,
        }],
    });

    let transform_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("bind_group_layout::transform"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: BufferSize::new(size_of::<Transform>() as u64),
            },
            count: None,
        }],
    });

    let texture_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("bind_group_layout::texture"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
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
        label: Some("pipeline_layout"),
        bind_group_layouts: &[
            &global_bind_group_layout,
            &transform_bind_group_layout,
            &texture_bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

    // Define pipelines.

    let (mut depth_texture, mut depth_texture_view) =
        create_depth_texture(&device, physical_size.width, physical_size.height);

    let shader_src = include_str!("shader.wgsl");
    let shader_module = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("shader_module"),
        source: ShaderSource::Wgsl(Cow::Borrowed(shader_src)),
    });

    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("render_pipeline"),
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: &shader_module,
            entry_point: "vs_main",
            buffers: &[Vertex::layout()],
        },
        primitive: PrimitiveState {
            front_face: FrontFace::Ccw,
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
            module: &shader_module,
            entry_point: "fs_main",
            targets: &[Some(surface_config.format.into())],
        }),
        multiview: None,
    });

    // Game objects.
    let mut camera = Camera::new(&CameraDescriptor {
        aspect_ratio: SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32,
        ..Default::default()
    });

    let camera_ubo = device.create_buffer(&BufferDescriptor {
        label: Some("ubo::camera"),
        size: size_of::<GpuCamera>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let global_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("bind_group::global"),
        layout: &global_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: camera_ubo.as_entire_binding(),
        }],
    });

    let model = Model::load(
        "assets/awesomeface.png",
        &device,
        &queue,
        &texture_bind_group_layout,
    );

    let transform = Transform::new(Mat4::from_scale_rotation_translation(
        Vec3::ONE,
        Quat::IDENTITY,
        Vec3::ZERO,
    ));

    let transform_ubo = device.create_buffer(&BufferDescriptor {
        label: Some("ubo::transform"),
        size: size_of::<Transform>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let transform_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("bind_group::transform"),
        layout: &transform_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: transform_ubo.as_entire_binding(),
        }],
    });

    queue.write_buffer(&transform_ubo, 0, cast_slice(&[transform]));

    window.set_cursor_visible(false);
    window
        .set_cursor_grab(CursorGrabMode::Confined)
        .expect("failed to grab cursor");
    window.set_visible(true);

    let mut last_time = Instant::now();
    let mut running = true;
    while running {
        let current_time = Instant::now();
        let dt = (current_time - last_time).as_secs_f32();
        last_time = current_time;

        running = process_events(
            &mut event_loop,
            &window,
            &device,
            &surface,
            &mut surface_config,
            &mut depth_texture,
            &mut depth_texture_view,
            &mut camera,
            dt,
        );

        queue.write_buffer(&camera_ubo, 0, cast_slice(&[camera.get_gpu_camera()]));

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

            rpass.set_bind_group(0, &global_bind_group, &[]);
            rpass.set_bind_group(1, &transform_bind_group, &[]);
            rpass.set_pipeline(&pipeline);

            for mesh in &model.meshes {
                let material = &model.materials[mesh.material_id];

                rpass.set_vertex_buffer(0, mesh.vbo.slice(..));
                rpass.set_index_buffer(mesh.ibo.slice(..), IndexFormat::Uint32);
                rpass.set_bind_group(2, &material.bind_group, &[]);
                rpass.draw_indexed(0..mesh.indices.len() as u32, 0, 0..1);
            }
        }

        queue.submit(once(encoder.finish()));
        frame.present();
    }
}

fn setup_gpu(window: &Window) -> (Instance, Adapter, Device, Queue, Surface) {
    let instance = Instance::new(InstanceDescriptor {
        backends: Backends::PRIMARY,
        ..Default::default()
    });

    let surface = unsafe {
        instance
            .create_surface(window)
            .expect("failed to create a surface")
    };

    let adapter = block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    }))
    .expect("failed to get a compatible adapter");

    let (device, queue) = block_on(adapter.request_device(
        &DeviceDescriptor {
            label: Some("device"),
            features: adapter.features(),
            limits: adapter.limits(),
        },
        None,
    ))
    .expect("failed to get a compatible device");

    (instance, adapter, device, queue, surface)
}

fn setup_surface(
    surface: &Surface,
    width: u32,
    height: u32,
    adapter: &Adapter,
    device: &Device,
) -> SurfaceConfiguration {
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

    let config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width,
        height,
        present_mode: PresentMode::Fifo,
        alpha_mode: surface_capabilities.alpha_modes[0],
        view_formats: Vec::new(),
    };

    surface.configure(&device, &config);

    config
}

fn create_depth_texture(device: &Device, width: u32, height: u32) -> (wgpu::Texture, TextureView) {
    let texture = device.create_texture(&TextureDescriptor {
        label: Some("depth_texture"),
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

fn process_events(
    event_loop: &mut EventLoop<()>,
    window: &Window,
    device: &Device,
    surface: &Surface,
    surface_config: &mut SurfaceConfiguration,
    depth_texture: &mut wgpu::Texture,
    depth_texture_view: &mut TextureView,
    camera: &mut Camera,
    dt: f32,
) -> bool {
    let mut quit = false;

    event_loop.run_return(|event, _, control_flow| {
        control_flow.set_wait();

        match event {
            Event::WindowEvent { window_id, event } if window.id() == window_id => match event {
                WindowEvent::CloseRequested => quit = true,

                WindowEvent::Resized(size) => {
                    surface_config.width = size.width;
                    surface_config.height = size.height;
                    surface.configure(&device, &surface_config);

                    (*depth_texture, *depth_texture_view) =
                        create_depth_texture(&device, surface_config.width, surface_config.height);
                }

                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    surface_config.width = new_inner_size.width;
                    surface_config.height = new_inner_size.height;
                    surface.configure(&device, &surface_config);

                    (*depth_texture, *depth_texture_view) =
                        create_depth_texture(&device, surface_config.width, surface_config.height);
                }

                WindowEvent::CursorEntered { .. } => {
                    camera.set_has_mouse(true);
                }

                WindowEvent::MouseWheel { delta, .. } => {
                    if let MouseScrollDelta::LineDelta(_, y) = delta {
                        camera.zoom(y);
                    }
                }

                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(key) = input.virtual_keycode {
                        match key {
                            VirtualKeyCode::Escape if input.state == ElementState::Pressed => {
                                quit = true;
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
            },

            Event::DeviceEvent { event, .. } if camera.has_mouse() => match event {
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

    !quit
}
