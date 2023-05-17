use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};

fn main() {
    let mut event_loop = EventLoop::new();

    let _window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(1280, 720))
        .with_title("Hello triangle")
        .with_visible(true)
        .build(&event_loop)
        .expect("failed to create a window");

    let mut running = true;
    while running {
        event_loop.run_return(|event, _, control_flow| {
            control_flow.set_wait();

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => running = false,

                    _ => (),
                },

                Event::MainEventsCleared => control_flow.set_exit(),

                _ => (),
            }
        });
    }
}
