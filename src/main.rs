mod cvulkan;

use std::{
    os,
    sync::Arc,
    time::{Duration, Instant},
};

use cvulkan::init::Volcan;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, Size},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

pub struct App {
    window: Option<Arc<Window>>,
    frame_count: u32,
    last_update_time: Instant,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            frame_count: 0,
            last_update_time: Instant::now(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut attributes = Window::default_attributes();
        attributes.title = "VOXRT".into();
        attributes.inner_size = Some(Size::Logical(LogicalSize::new(1920.0, 1080.0)));

        let window = Arc::new(event_loop.create_window(attributes).unwrap());
        self.window = Some(window.clone());

        let mut volcan = Volcan::new(&window);
        volcan.create_swapchain(1920, 1080);
        volcan.create_swapchain_images();

        volcan.unload();
        std::process::exit(0);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }

            // WindowEvent::Focused(WindowId) => {}
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();

                self.frame_count += 1;
                let now = Instant::now();
                let elapsed = now.duration_since(self.last_update_time);
                if elapsed >= Duration::from_secs(1) {
                    let title_str = format!("VOXRT. FPS: {}", self.frame_count);
                    self.window.as_ref().unwrap().set_title(&title_str);
                    self.frame_count = 0;
                    self.last_update_time = now;
                }
            }
            _ => (),
        }
    }
}

fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new();
    event_loop.run_app(&mut app).expect("TODO: panic message");
}
