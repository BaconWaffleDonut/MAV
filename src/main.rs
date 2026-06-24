use std::time::Instant;
use std::ptr::copy_nonoverlapping as memcpy;
use std::result::Result::Ok;
use anyhow::{anyhow, Result};
use ash::khr::surface;
use cgmath::{Deg, point3, vec3};
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop}, window::{Window, WindowAttributes, WindowId}};
use log::*;
use ash::{Device, Entry, Instance, khr::swapchain, vk::Handle, vk};
use crate::engine_functions::*;

const MAX_FRAMES_IN_FLIGHT: usize = 3;
type Mat4 = cgmath::Matrix4<f32>;


#[path = "util/file.rs"]
mod fill; // Only used for testing window creation
mod engine_functions;

fn main() -> Result<()> {
    pretty_env_logger::init();
    println!("Starting main function");

    let event_loop = engine_functions::Utils::event_loop().expect("MAIN: Failed to import event loop.");
    println!("Running event loop.");

    engine_functions::test().expect("Failed to load engine test function.");
    Engine::main();
    EventLoop::run_app(event_loop, App::default());
        
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
                /* fill::fill_window(window.as_ref()); */
                let mut app = unsafe { Engine::create(window.as_ref()).expect("Failed to create application.") };
                unsafe{app.render(window.as_ref()).unwrap()};
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
    unsafe fn create(window: &dyn Window) -> Result<Self> {
        let mut data = EngineData::default() ;
        let entry = unsafe { Entry::load().map_err(|b| anyhow!("{}", b))? };
        let instance = unsafe { create_instance(&mut data, window, &entry) }?;
        data.surface = engine_functions::Utils::surface(&entry, window, &instance)?;
        unsafe { pick_physical_device(&instance, &entry, window)? };
        let device = create_logical_device(&entry, &instance, window, &data)?;
        create_swapchain(window, &instance, &device, &mut data, &entry)?;
        create_swapchain_image_views(&device, &mut data)?;
        create_render_pass(&instance, &device, &mut data)?;
        create_descriptor_set_layout(&device, &mut data)?;
        create_pipeline(&device, &mut data)?;
        create_command_pools(&instance, &device, &mut data, &entry, window)?;
        create_color_objects(&instance, &device, &mut data)?;
        create_depth_objects(&instance, &device, &mut data)?;
        create_framebuffers(&device, &mut data)?;
        create_texture_image(&instance, &device, &mut data)?;
        create_texture_image_view(&device, &mut data)?;
        create_texture_sampler(&device, &mut data)?;
        load_model(&mut data)?;
        create_vertex_buffer(&instance, &device, &mut data)?;
        create_index_buffer(&instance, &device, &mut data)?;
        create_uniform_buffers(&instance, &device, &mut data)?;
        create_descriptor_pool(&device, &mut data)?;
        create_descriptor_sets(&device, &mut data)?;
        create_command_buffers(&device, &mut data)?;
        create_sync_objects(&device, &mut data)?;
        Ok(Self {
            entry,
            instance,
            data,
            device,
            frame: 0,
            resized: false,
            start: Instant::now(),
            models: 1,
        })
    }
    
    // Render a frame
    unsafe fn render(&mut self, window: &dyn Window) -> Result<()> {
        let in_flight_fence = self.data.in_flight_fences[self.frame];
        (unsafe { self.device.wait_for_fences(&[in_flight_fence], true, u64::MAX) })?;
        let result = unsafe { self.data.swapchain_loader.acquire_next_image(self.data.swapchain, u64::MAX, self.data.image_available_semaphores[self.frame], vk::Fence::null()) };
        
        let image_index = match result {
            Ok((image_index, _)) => image_index as usize,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => return unsafe { self.recreate_swapchain(window) },
            Err(e) => return Err(anyhow!("MAIN: {}", e)),
        };

        let image_in_flight = self.data.images_in_flight[image_index];
        if !image_in_flight.is_null() {
            (unsafe { self.device.wait_for_fences(&[image_in_flight], true, u64::MAX) })?;
        }

        self.data.images_in_flight[image_index] = in_flight_fence;
        (unsafe { self.update_command_buffer(image_index) })?;
        (unsafe { self.update_uniform_buffer(image_index) })?;

        let wait_semaphores = &[self.data.image_available_semaphores[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.data.command_buffers[image_index]];
        let signal_semaphores = &[self.data.render_finished_semaphores[self.frame]];
        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        (unsafe { self.device.reset_fences(&[in_flight_fence]) })?;
        (unsafe { self.device.queue_submit(self.data.graphics_queue, &[submit_info], in_flight_fence) })?;

        let swapchains = &[self.data.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        let result = unsafe { self.data.swapchain_loader.queue_present(self.data.present_queue, &present_info) };
        let changed = result == Err(vk::Result::SUBOPTIMAL_KHR) || result == Err(vk::Result::ERROR_OUT_OF_DATE_KHR);
        if self.resized || changed {
            self.resized = false;
            (unsafe { self.recreate_swapchain(window)})?;
        } else if let Err(e) = result {
            return Err(anyhow!("MAIN: {}", e));
        }

        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }

    // Update Command Buffer
    unsafe fn update_command_buffer(&mut self, image_index: usize) -> Result<()> {
        // Reset
        let command_pool = self.data.command_pools[image_index];
        (unsafe { self.device.reset_command_pool(command_pool, vk::CommandPoolResetFlags::empty()) })?;
        let command_buffer = self.data.command_buffers[image_index];

        // Commands
        let info = vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        (unsafe { self.device.begin_command_buffer(command_buffer, &info) })?;

        let render_area = vk::Rect2D::default()
            .offset(vk::Offset2D::default())
            .extent(self.data.swapchain_extent);
        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };
        let depth_stencil_clear_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {depth: 1.0, stencil: 0},
        };
        let clear_values = &[color_clear_value, depth_stencil_clear_value];
        let info = vk::RenderPassBeginInfo::default()
            .render_pass(self.data.render_pass)
            .framebuffer(self.data.framebuffers[image_index])
            .render_area(render_area)
            .clear_values(clear_values);
        unsafe { self.device.cmd_begin_render_pass(command_buffer, &info, vk::SubpassContents::SECONDARY_COMMAND_BUFFERS) };

        let secondary_command_buffers = (0..self.models)
            .map(|i| unsafe { self.update_secondary_command_buffers(image_index, i) })
            .collect::<Result<Vec<_>, _>>()?;
        unsafe { self.device.cmd_execute_commands(command_buffer, &secondary_command_buffers[..]) };
        unsafe { self.device.cmd_end_render_pass(command_buffer) };
        (unsafe { self.device.end_command_buffer(command_buffer) })?;

        Ok(())
    }

    // Update Secondary Command Buffer
    unsafe fn update_secondary_command_buffers(&mut self, image_index: usize, model_index: usize) -> Result<vk::CommandBuffer> {
        // Allocate
        let command_buffers = &mut self.data.secondary_command_buffers[image_index];
        while model_index >= command_buffers.len() {
            let allocate_info = vk::CommandBufferAllocateInfo::default()
                .command_pool(self.data.command_pools[image_index])
                .level(vk::CommandBufferLevel::SECONDARY)
                .command_buffer_count(1);
            let command_buffer = unsafe { self.device.allocate_command_buffers(&allocate_info) }?[0];
            command_buffers.push(command_buffer);
        }
        let command_buffer = command_buffers[model_index];

        // Model 
        let y = (((model_index % 2) as f32) * 2.5) - 1.25;
        let z = (((model_index / 2) as f32) * -2.0) + 1.0;

        let time = self.start.elapsed().as_secs_f32();
        let model = Mat4::from_translation(vec3(0.0, y, z)) * Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), Deg(90.0) * time);
        let model_bytes = unsafe { std::slice::from_raw_parts(&model as *const Mat4 as *const u8, size_of::<Mat4>()) };
        let opacity = (model_index + 1) as f32 * 0.25;
        let opacity_bytes = &opacity.to_ne_bytes()[..];

        // Commands
        let inheritance_info = vk::CommandBufferInheritanceInfo::default()
            .render_pass(self.data.render_pass)
            .subpass(0)
            .framebuffer(self.data.framebuffers[image_index]);
        let info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE)
            .inheritance_info(&inheritance_info);

        unsafe { self.device.begin_command_buffer(command_buffer, &info) }?;
        unsafe { self.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.data.pipeline) };
        unsafe { self.device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.data.vertex_buffer], &[0]) };
        unsafe { self.device.cmd_bind_index_buffer(command_buffer, self.data.index_buffer, 0, vk::IndexType::UINT32) };
        unsafe { self.device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.data.pipeline_layout, 0, &[self.data.descriptor_sets[image_index]], &[]) };
        unsafe { self.device.cmd_push_constants(command_buffer, self.data.pipeline_layout, vk::ShaderStageFlags::VERTEX, 0, model_bytes) };
        unsafe { self.device.cmd_push_constants(command_buffer, self.data.pipeline_layout, vk::ShaderStageFlags::FRAGMENT, 64, opacity_bytes) };
        unsafe { self.device.cmd_draw_indexed(command_buffer, self.data.indices.len() as u32, 1, 0, 0, 0) };
        unsafe { self.device.end_command_buffer(command_buffer) }?;

        Ok(command_buffer)

    }

    // Update Uniform Buffer Object
    unsafe fn update_uniform_buffer(&mut self, image_index: usize) -> Result<()> {
        // MVP
        let view = Mat4::look_at_rh(
            point3::<f32>(6.0, 0.0, 2.0), 
            point3::<f32>(0.0, 0.0, 0.0), 
            vec3(0.0, 0.0, 1.0));
        let correction = Mat4::new(
            1.0, 0.0, 0.0, 0.0, 
            0.0, -1.0, 0.0, 0.0, 
            0.0, 0.0, 1.0 / 2.0, 0.0, 
            0.0, 0.0, 1.0 / 2.0, 1.0);
        let proj = correction * cgmath::perspective(Deg(45.0), self.data.swapchain_extent.width as f32 / self.data.swapchain_extent.height as f32, 0.1, 10.0);
        let ubo = UniformBufferObject { view, proj };

        // Copy
        let memory = unsafe { self.device.map_memory(self.data.uniform_buffers_memory[image_index], 0, size_of::<UniformBufferObject> as u64, vk::MemoryMapFlags::empty()) }?;
        unsafe { memcpy(&ubo, memory.cast(), 1) };
        unsafe { self.device.unmap_memory(self.data.uniform_buffers_memory[image_index]) };

        Ok(())
    }

    // Recreate the Swapchain
    unsafe fn recreate_swapchain(&mut self, window: &dyn Window) -> Result<()> {
        (unsafe { self.device.device_wait_idle() })?;
        self.destroy_swapchain();
        create_swapchain(window, &self.instance, &self.device, &mut self.data, &self.entry)?;
        create_swapchain_image_views(&self.device, &mut self.data)?;
        create_render_pass(&self.instance, &self.device, &mut self.data)?;
        create_pipeline(&self.device, &mut self.data)?;
        create_color_objects(&self.instance, &self.device, &mut self.data)?;
        create_depth_objects(&self.instance, &self.device, &mut self.data)?;
        create_framebuffers(&self.device, &mut self.data)?;
        create_uniform_buffers(&self.instance, &self.device, &mut self.data)?;
        create_descriptor_pool(&self.device, &mut self.data)?;
        create_descriptor_sets(&self.device, &mut self.data)?;
        self.data.images_in_flight.resize(self.data.swapchain_images.len(), vk::Fence::null());
        Ok(())

    }

    // Destroy the Vulkan app
    unsafe fn destroy(&mut self) {
        unsafe { self.device.device_wait_idle().unwrap() };
        
        self.destroy_swapchain();

        self.data.in_flight_fences.iter().for_each(|f| unsafe { self.device.destroy_fence(*f, None) });
        self.data.render_finished_semaphores.iter().for_each(|s| unsafe { self.device.destroy_semaphore(*s, None) });
        self.data.image_available_semaphores.iter().for_each(|s| unsafe { self.device.destroy_semaphore(*s, None) });
        self.data.command_pools.iter().for_each(|p| unsafe { self.device.destroy_command_pool(*p, None) });
        unsafe { self.device.free_memory(self.data.index_buffer_memory, None) };
        unsafe { self.device.destroy_buffer(self.data.index_buffer, None) };
        unsafe { self.device.free_memory(self.data.vertex_buffer_memory, None) };
        unsafe { self.device.destroy_buffer(self.data.vertex_buffer, None) };
        unsafe { self.device.destroy_sampler(self.data.texture_sampler, None) };
        unsafe { self.device.destroy_image_view(self.data.texture_image_view, None) };
        unsafe { self.device.free_memory(self.data.texture_image_memory, None) };
        unsafe { self.device.destroy_image(self.data.texture_image, None) };
        unsafe { self.device.destroy_command_pool(self.data.command_pool, None) };
        unsafe { self.device.destroy_descriptor_set_layout(self.data.descriptor_set_layout, None) };
        unsafe { self.device.destroy_device(None) };
        unsafe { self.data.surface_loader.destroy_surface(self.data.surface, None) };

        if VALIDATION_ENABLED {
            unsafe { self.data.debug_utils_loader.destroy_debug_utils_messenger(self.data.debug_call_back, None) };
        }
        
        unsafe { self.instance.destroy_instance(None) };

        
    }

    // Destroy Swapchain
    unsafe fn destroy_swapchain(&mut self) {
        unsafe { self.device.destroy_descriptor_pool(self.data.descriptor_pool, None) };
        self.data.uniform_buffers_memory.iter().for_each(|m| unsafe { self.device.free_memory(*m, None) });
        self.data.uniform_buffers.iter().for_each(|b| unsafe { self.device.destroy_buffer(*b, None) });
        unsafe { self.device.destroy_image_view(self.data.depth_image_view, None) };
        unsafe { self.device.free_memory(self.data.depth_image_memory, None) };
        unsafe { self.device.destroy_image_view(self.data.color_image_view, None) };
        unsafe { self.device.free_memory(self.data.color_image_memory, None) };
        unsafe { self.device.destroy_image(self.data.color_image, None) };
        self.data.framebuffers.iter().for_each(|f| unsafe { self.device.destroy_framebuffer(*f, None) });
        unsafe { self.device.destroy_pipeline(self.data.pipeline, None) };
        unsafe { self.device.destroy_pipeline_layout(self.data.pipeline_layout, None) };
        unsafe { self.device.destroy_render_pass(self.data.render_pass, None) };
        self.data.swapchain_image_views.iter().for_each(|v| unsafe { self.device.destroy_image_view(*v, None) });
        unsafe { self.data.swapchain_loader.destroy_swapchain(self.data.swapchain, None) };
    }

    //
}

#[derive(Debug, Clone, Default)]
struct EngineData {
    // Debug
    messenger: vk::DebugUtilsMessengerEXT,
    debug_call_back: vk::DebugUtilsMessengerEXT,
    debug_utils_loader: ash::ext::debug_utils::Instance,

    // Surface
    surface: vk::SurfaceKHR,
    surface_format: vk::SurfaceFormatKHR,
    surface_loader: surface::Instance,

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
    swapchain_loader: swapchain::Device,

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
