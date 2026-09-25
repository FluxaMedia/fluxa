use std::{sync::Arc, time::Instant};

use fluxa_renderer::{WgpuRenderer, sample_home_scene};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<WgpuRenderer>,
    started_at: Instant,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            renderer: None,
            started_at: Instant::now(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title("Fluxa Native Renderer")
            .with_inner_size(winit::dpi::LogicalSize::new(1152.0, 720.0));
        let window = Arc::new(event_loop.create_window(attributes).expect("create window"));
        let renderer =
            pollster::block_on(WgpuRenderer::new(window.clone())).expect("create renderer");
        self.window = Some(window.clone());
        self.renderer = Some(renderer);
        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = self.renderer.as_mut() {
                    let elapsed = self.started_at.elapsed().as_secs_f32();
                    renderer
                        .render(&sample_home_scene(elapsed, 2))
                        .expect("render frame");
                    renderer.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), winit::error::EventLoopError> {
    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut App::default())
}
