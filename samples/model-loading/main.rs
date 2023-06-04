// todo: draw triangle.
// todo: draw a quad.
// todo: add a camera.
// todo: add mesh.
// todo: add model.
// todo: assimp stuff.

use std::iter::once;

use futures::executor::block_on;
use wgpu::{
    Adapter, Backends, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Extent3d,
    Instance, InstanceDescriptor, LoadOp, Operations, PowerPreference, PresentMode, Queue,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    RequestAdapterOptions, Surface, SurfaceConfiguration, Texture, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    platform::run_return::EventLoopExtRunReturn,
    window::{Window, WindowBuilder},
};

const SCREEN_WIDTH: u32 = 1280;
const SCREEN_HEIGHT: u32 = 720;
const TITLE: &'static str = "Model loading";

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

    // Define pipelines.

    let (mut depth_texture, mut depth_texture_view) =
        create_depth_texture(&device, physical_size.width, physical_size.height);

    // grab mouse.
    window.set_visible(true);
    let mut running = true;
    while running {
        running = process_events(
            &mut event_loop,
            &window,
            &device,
            &surface,
            &mut surface_config,
            &mut depth_texture,
            &mut depth_texture_view,
        );

        // Update.

        let frame = surface
            .get_current_texture()
            .expect("failed to get current swapchain texture");
        let output_texture_view = frame.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("command_encoder"),
        });

        {
            let rpass = encoder.begin_render_pass(&RenderPassDescriptor {
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

fn create_depth_texture(device: &Device, width: u32, height: u32) -> (Texture, TextureView) {
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
    depth_texture: &mut Texture,
    depth_texture_view: &mut TextureView,
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

                _ => (),
            },

            Event::MainEventsCleared => control_flow.set_exit(),

            _ => (),
        }
    });

    !quit
}
