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
    Device, Entry, Instance, khr::surface, vk
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

#[derive(Clone, Debug, Default)]
struct EngineData {
    // Debug
    messenger: vk::DebugUtilsMessengerEXT,
    debug_call_back: vk::DebugUtilsMessengerEXT,

    // Surface
    surface: vk::SurfaceKHR,
    surface_format: vk::SurfaceFormatKHR,

    // Physical & Logical Device
    physical_device: vk::PhysicalDevice,
    msaa_samples: vk::SampleCountFlags,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    device_extension_names_raw: vk::PhysicalDeviceFeatures,

    // Swapchain
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,

    // Pipeline
    render_pass: vk::RenderPass,
    descriptor_set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,

    // Framebuffers
    framebuffers: Vec<vk::Framebuffer>,

    // Command Pool
    command_pool: vk::CommandPool,

    // Color
    color_image: vk::Image,
    color_image_memory: vk::DeviceMemory,
    color_image_view: vk::ImageView,

    // Depth
    depth_image: vk::Image,
    depth_image_memory: vk::DeviceMemory,
    depth_image_view: vk::ImageView,

    // Texture
    mip_levels: u32,
    texture_image: vk::Image,
    texture_image_memory: vk::DeviceMemory,
    texture_image_view: vk::ImageView,
    texture_sampler: vk::Sampler,

    // Model
    vertices: Vec<engine_functions::Vertex>,
    indices: Vec<u32>,

    // Buffers
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,
    uniform_buffers: Vec<vk::Buffer>,
    uniform_buffers_memory: Vec<vk::DeviceMemory>,

    // Descriptors
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,

    // Command Buffers
    command_pools: Vec<vk::CommandPool>,
    command_buffers: Vec<vk::CommandBuffer>,
    secondary_command_buffers: Vec<Vec<vk::CommandBuffer>>,

    // Sync Objects
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    images_in_flight: Vec<vk::Fence>,

    // MISC
    window_height: u32,
    window_width: u32,
}
