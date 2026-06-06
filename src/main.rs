use std::{
    cell::RefCell, 
    error::Error, time::Instant};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{
        ActiveEventLoop,
        EventLoop
    },
    window::{
        Window,
        WindowAttributes,
        WindowId
    },
};
use log::*;
use ash::{
    ext::debug_utils, 
    khr::{
        surface, 
        swapchain},
    vk, Device, Entry, Instance,
};

#[path = "util/file.rs"]
mod fill; // Only used for testing window creation
mod engine_functions;

fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    println!("Starting main function");

    let event_loop = EventLoop::new().expect("Failed to start event loop.");
    println!("Running event loop.");

    engine_functions::test();
    Engine::main();
    event_loop.run_app(App::default())?;
        
    Ok(()) 
}

#[derive(Default, Debug)]
struct App {
    window: Option<Box<dyn Window>>
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

//====================
// Video Engine
//====================

// Constants and structures
const MAX_FRAMES_IN_FLIGHT: usize = 3;

type Vec2 = cgmath::Vector2<f32>;
type Vec3 = cgmath::Vector3<f32>;
type Mat4 = cgmath::Matrix4<f32>;

struct Engine {
    // Vulkan Stuff
    entry: Entry,
    instance: Instance,
    data: EngineData,
    device: Device,
    frame: usize,
    resized: bool,
    start: Instant,
    models: usize,
}

impl Engine {
    fn main() {
        println!("Starting Engine.")
    }
    // Create the Vulkan App
    
    // Render a frame

    // Update Command Buffer

    // Update Secondary Command Buffer

    // Update Uniform Buffer Object

    // Recreate the Swapchain

    // Destroy the Vulkan app

    // Destroy Swapchain

    //
}

struct EngineData {

}
