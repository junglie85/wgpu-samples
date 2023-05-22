use std::{borrow::Cow, iter::once, mem::size_of};

use bytemuck::cast_slice;
use bytemuck_derive::{Pod, Zeroable};
use futures::executor::block_on;
use wgpu::{
    BufferDescriptor, BufferUsages, Color, CommandEncoderDescriptor, DeviceDescriptor, Features,
    FragmentState, IndexFormat, Instance, LoadOp, MultisampleState, Operations,
    PipelineLayoutDescriptor, PolygonMode, PresentMode, PrimitiveState, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor,
    ShaderSource, SurfaceConfiguration, TextureFormat, TextureUsages, TextureViewDescriptor,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::EventLoop,
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};

#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    position: [f32; 3],
}

impl Vertex {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: [x, y, z],
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

fn main() {
    let mut event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(1280, 720))
        .with_title("Hello rectangle")
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

    let mut features = Features::empty();
    features.insert(
        Features::from_name("POLYGON_MODE_LINE").expect("POLYGON_MODE_LINE is not supported"),
    );

    let (device, queue) = block_on(adapter.request_device(
        &DeviceDescriptor {
            label: None,
            features,
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

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

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

    let filled_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("filled_pipeline"),
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[Vertex::layout()],
        },
        primitive: PrimitiveState::default(),
        depth_stencil: None,
        multisample: MultisampleState::default(),
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(swapchian_format.into())],
        }),
        multiview: None,
    });

    let wireframe_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("wireframme_pipeline"),
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[Vertex::layout()],
        },
        primitive: PrimitiveState {
            polygon_mode: PolygonMode::Line,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: MultisampleState::default(),
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(swapchian_format.into())],
        }),
        multiview: None,
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

    let vertices = [
        Vertex::new(-0.5, 0.5, 0.0),
        Vertex::new(-0.5, -0.5, 0.0),
        Vertex::new(0.5, -0.5, 0.0),
        Vertex::new(0.5, 0.5, 0.0),
    ];

    let indices = [0_u32, 1, 3, 1, 2, 3];

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

    queue.write_buffer(&vbo, 0, cast_slice(&vertices));
    queue.write_buffer(&ibo, 0, cast_slice(&indices));

    window.set_visible(true);
    let mut running = true;
    let mut wireframe = false;
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
                    }

                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        config.width = new_inner_size.width;
                        config.height = new_inner_size.height;
                        surface.configure(&device, &config);
                    }

                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(virtual_keycode) = input.virtual_keycode {
                            if virtual_keycode == VirtualKeyCode::Space
                                && input.state == ElementState::Pressed
                            {
                                wireframe = !wireframe;
                            }
                        }
                    }

                    _ => (),
                },

                Event::MainEventsCleared => control_flow.set_exit(),

                _ => (),
            }
        });

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
                depth_stencil_attachment: None,
            });

            if wireframe {
                rpass.set_pipeline(&wireframe_pipeline);
            } else {
                rpass.set_pipeline(&filled_pipeline);
            }
            rpass.set_vertex_buffer(0, vbo.slice(..));
            rpass.set_index_buffer(ibo.slice(..), IndexFormat::Uint32);
            rpass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        }

        queue.submit(once(encoder.finish()));
        frame.present();
    }
}
