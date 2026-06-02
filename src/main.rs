// figure out functions and stuff later, just get functions and stuff down
use std::error::Error;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};
use log::*;

#[path = "util/file.rs"]
mod fill;

fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    println!("Starting main function");

    let event_loop = EventLoop::new().expect("Failed to start event loop.");

    event_loop.run_app(App::default())?;
    
    Ok(()) 
}

#[derive(Default, Debug)]
struct App {
    window: Option<Box<dyn Window>>,
}

impl ApplicationHandler for App {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let window_attributes = WindowAttributes::default().with_title("M.A.V");
        self.window = match event_loop.create_window(window_attributes) {
            Ok(window) => Some(window),
            Err(err) => {
                error!("Error creating window: {err}");
                event_loop.exit();
                return;
            },
        }
    }

    fn window_event(&mut self, event_loop: &dyn ActiveEventLoop, _: WindowId, event: WindowEvent) {
        info!("{event:?}");
        match event {
            WindowEvent::CloseRequested => {
                info!("Close was requested; stopping");
                event_loop.exit();
            },
            WindowEvent::SurfaceResized(_) => {
                self.window.as_ref().expect("Resize without a window").request_redraw();
            },
            WindowEvent::RedrawRequested => {
                // Redraw the application here
                let window = self.window.as_ref().expect("Redraw requested without a window");

                // Notify that youre about to redraw
                window.pre_present_notify();

                //Draw, using temporary full color window for testing
                fill::fill_window(window.as_ref());
                //Can use window.request_redraw(); for continous loop
            },
            _ => (),
        }
    }
}

