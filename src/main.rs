use std::f32::consts::PI;
use std::{time::Instant, u64};
use std::result::Result::Ok;
use anyhow::{anyhow, Result};
use ash::khr::surface;
use ash::vk::DeviceMemory;
use cgmath::num_traits::{Signed, ToPrimitive};
use cgmath::{Deg, Transform, point3, vec3};
use vk_mem::{Allocation, Allocator, AllocatorCreateInfo};
use winit::dpi::{LogicalPosition, PhysicalSize};
use winit::event::{ButtonSource, DeviceEvent, ElementState, MouseButton};
use winit::event_loop::ControlFlow::Poll;
use winit::keyboard::KeyCode::ArrowRight;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::CursorGrabMode;
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop}, raw_window_handle::{HasDisplayHandle, HasWindowHandle}, window::{Window, WindowAttributes, WindowId}};
use log::*;
use ash::{Device, Entry, Instance, khr::swapchain, vk};
use crate::engine_functions::*;
use crate::util::camera::{self, *};
use crate::util::math::{self, matrix_mult, rotate_x, rotate_y, rotate_z, translate};
mod util;

type Mat4 = cgmath::Matrix4<f32>;
const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;


#[path = "util/file.rs"]
mod fill; // Only used for testing window creation
mod engine_functions;

fn main() -> Result<()> {
    pretty_env_logger::init();
    println!("Starting main function");

    let event_loop = EventLoop::new()?;
    println!("Running event loop.");

    engine_functions::test().expect("Failed to load engine test function.");
    Engine::main();
    EventLoop::run_app(event_loop, App::default()).expect("Failed to run app.");
        
    Ok(())
}

#[derive(Default)]
struct App {
    window: Option<Box<dyn Window>>,
    vulkan: Option<EngineData>,
    app: Option<Engine>
}

impl ApplicationHandler for App {
    fn window_event(&mut self, event_loop: &dyn ActiveEventLoop, _: WindowId, event: WindowEvent) {
        info!("{event:?}");
        event_loop.set_control_flow(Poll);
        match event {
            WindowEvent::CloseRequested => {
                info!("Close was requested; stopping");
                event_loop.exit();
                unsafe { self.app.as_mut().unwrap().destroy() };
            },

            WindowEvent::SurfaceResized(size) => {
                if size.width == 0 || size.height == 0 {
                    self.app.as_mut().unwrap().minimized = true;
                } else {
                    self.app.as_mut().unwrap().minimized = false;
                    self.window.as_ref().expect("Resize without a window").request_redraw();
                }
            },

            WindowEvent::RedrawRequested if !event_loop.exiting() && !self.app.as_ref().unwrap().minimized => {
                // Redraw the application here
                let window = self.window.as_ref().expect("Redraw requested without a window");

                // Notify that youre about to redraw
                window.pre_present_notify();

                //Draw, using temporary full color window for testing
                unsafe{self.app.as_mut().unwrap().render().expect("Failed to render...")};

            },

            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::KeyW) => self.app.as_mut().unwrap().camera.pos_x += 0.1,
                        PhysicalKey::Code(KeyCode::KeyS) => self.app.as_mut().unwrap().camera.pos_x -= 0.1,
                        PhysicalKey::Code(KeyCode::KeyA) => self.app.as_mut().unwrap().camera.pos_z += 0.1,
                        PhysicalKey::Code(KeyCode::KeyD) => self.app.as_mut().unwrap().camera.pos_z -= 0.1,
                        PhysicalKey::Code(KeyCode::Space) => self.app.as_mut().unwrap().camera.pos_y += 0.1,
                        PhysicalKey::Code(KeyCode::ShiftLeft) => self.app.as_mut().unwrap().camera.pos_y -= 0.1,
                        PhysicalKey::Code(KeyCode::KeyQ) => self.app.as_mut().unwrap().camera.rot_y -= 0.1,
                        PhysicalKey::Code(KeyCode::KeyE) => self.app.as_mut().unwrap().camera.rot_y += 0.1,
                        _ => {}
                    }
                }
            },

            _ => (),
        }
    }

    fn device_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        device_id: Option<winit::event::DeviceId>,
        event: DeviceEvent,
    )
    {
        match event {
            DeviceEvent::PointerMotion { delta } => {
                let camera = self.app.as_ref().unwrap().camera;
                if delta.1.is_sign_positive() == true {
                    if camera.rot_x <= PI {
                        self.app.as_mut().unwrap().camera.rot_x += (delta.1 / 1000.to_f64().unwrap()) as f32;
                    }
                }
                if delta.1.is_sign_negative() == true {
                    if camera.rot_x >= 0.0 {
                        self.app.as_mut().unwrap().camera.rot_x += (delta.1 / 1000.to_f64().unwrap()) as f32;
                    }
                }
                self.app.as_mut().unwrap().camera.rot_z -= (delta.0 / 1000.to_f64().unwrap()) as f32;
            },

            _ => {},
        }
    }
    
    fn new_events(&mut self, event_loop: &dyn ActiveEventLoop, cause: winit::event::StartCause) {
        if let Some(app) = self.app.as_mut() {
            self.window.as_ref().unwrap().set_cursor_position(LogicalPosition::new(100, 100).into());
        }
    }

    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let window_attributes = WindowAttributes::default().with_title("M.A.V").with_surface_size(PhysicalSize::new(WIDTH, HEIGHT));
        let window = match event_loop.create_window(window_attributes) {
            Ok(window) => window,
            Err(err) => {
                error!("Error creating window: {err}");
                event_loop.exit();
                return;
            },
        };
        let app = unsafe { Engine::create( window.as_ref(), event_loop).expect("Failed to create application.") };
        window.as_ref().set_cursor_grab(winit::window::CursorGrabMode::Locked);
        window.as_ref().set_cursor_visible(false);
        self.app = Some(app);
        self.window = Some(window);
    }

    fn about_to_wait(&mut self, _: &dyn ActiveEventLoop) {
        let app = self.app.as_mut().unwrap();
        let window = self.window.as_ref().unwrap();

        if app.resized {
            let size = window.surface_size();
            if size.width > 0 && size.height > 0 {
                app.resize_dimensions = [size.width, size.height];
                unsafe { app.recreate_swapchain() };
            } else {
                return;
            }
        }
        app.resized = unsafe { app.render().unwrap() };
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
    resize_dimensions: [u32; 2],
    minimized: bool,
    camera: Camera,
    wheel_delta: Option<f32>,
    cursor_delta: Option<[i32; 2]>,
    cursor_pos: [i32; 2],
    left_clicked: bool,
}

impl Engine {
    fn main() {
        println!("Starting Engine.")
    }
    // Create the Vulkan App
    unsafe fn create(window: &dyn Window, event_loop: &dyn ActiveEventLoop) -> Result<Self> {
        let mut data = EngineData::default() ;
        println!("Create mut Data");
        let entry = unsafe { Entry::load().map_err(|b| anyhow!("{}", b))? };
        println!("Created Entry");
        let instance = create_instance(&mut data, event_loop).expect("MAIN: Failed to create Instace.");
        println!("Created Instace");
        data.surface = unsafe{ash_window::create_surface(&entry, &instance, event_loop.display_handle()?.as_raw(), window.window_handle()?.as_raw(), None)}.expect("Failed to create surface.");
        pick_physical_device(&instance, &entry, &mut data).expect("MAIN: Failed to pick Physical Device");
        println!("Picked phyiscal device");
        let device = create_logical_device(&instance, &mut data).expect("MAIN: Failed to create Logical Device");
        println!("Created Device");
        let allocator = unsafe { Allocator::new(AllocatorCreateInfo::new(&instance, &device, data.physical_device)).expect("Failed to create Allocator.") };
        println!("Created and pushed Allocator");
        create_swapchain(&mut data, &instance, &device, [WIDTH, HEIGHT])/* .expect("MAIN: Failed to create Swapchain") */;
        println!("Created Swapchain");
        create_swapchain_image_views(&device, &mut data).expect("MAIN: Failed to create Swapchain Image Views.");
        println!("Created Swapchain Image Views");
        create_render_pass(&instance, &device, &mut data).expect("MAIN: Failed to create Render Pass");
        println!("Created Render Pass");
        create_descriptor_set_layout(&device, &mut data).expect("MAIN: Failed to create Descriptor Set Layout.");
        println!("Created Descriptor Set Layout");
        create_pipeline(&device, &mut data).expect("MAIN: Failed to create Graphics Pipeline.");
        println!("Created Pipeline");
        create_command_pools(&instance, &device, &mut data).expect("MAIN: Failed to create Command Pools.");
        println!("Created Command Pools");
        create_color_objects(&instance, &device, &mut data).expect("MAIN: Failed to create Color Objects.");
        println!("Created Color Objects");
        create_depth_objects(&instance, &device, &mut data).expect("MAIN: Failed to create Depth Objects.");
        println!("Created Depth Objects");
        create_framebuffers(&device, &mut data).expect("MAIN: Failed to create Framebuffers.");
        println!("Created Framebuffers");
        create_texture_image(&instance, &device, &mut data, &allocator).expect("MAIN: Failed to create Texture Image.");
        println!("Created Texture Image");
        create_texture_image_view(&device, &mut data).expect("MAIN: Failed to create Texture Image View.");
        println!("Created Texture Image View");
        create_texture_sampler(&device, &mut data).expect("MAIN: Failed to create Texture Sampler.");
        println!("Created Texture Sampler");
        load_model(&mut data).expect("MAIN: Failed to load Model.");
        println!("Loaded Model");
        create_vertex_buffer(&instance, &device, &mut data, &allocator).expect("MAIN: Failed to create Vertex Buffer.");
        println!("Created Vertex Buffer");
        create_index_buffer(&instance, &device, &mut data, &allocator).expect("MAIN: Failed to create Index Buffer.");
        println!("Created Index Buffer");
        create_uniform_buffers(&instance, &device, &mut data, &allocator).expect("MAIN: Failed to create Uniform Buffers.");
        println!("Created Uniform Buffers");
        create_descriptor_pool(&device, &mut data).expect("MAIN: Failed to create Descriptor Pool.");
        println!("Created Descriptor Pool");
        create_descriptor_sets(&device, &mut data).expect("MAIN: Failed to create Descriptor Sets.");
        println!("Created Descriptor Sets");
        create_command_buffers(&device, &mut data).expect("MAIN: Failed to create Command Buffers.");
        println!("Created Command Buffers");
        create_sync_objects(&device, &mut data).expect("MAIN: Failed to create Sync Objects.");
        println!("Created Sync Objects");
        data.allocator = Some(allocator);
        Ok(Self {
            entry,
            instance,
            data,
            device,
            frame: 0,
            resized: false,
            start: Instant::now(),
            models: 9,
            resize_dimensions: [WIDTH, HEIGHT],
            minimized: false,
            camera: Default::default(),
            wheel_delta: None,
            cursor_delta: None,
            cursor_pos: [0, 0],
            left_clicked: false,
        })
    }
    
    // Render a frame
    unsafe fn render(&mut self) -> Result<bool> {
        let in_flight_fence = self.data.in_flight_fences[self.frame];
        let image_available_semaphores = self.data.image_available_semaphores[self.frame];
        let render_finished_semaphores = self.data.render_finished_semaphores[self.frame];
        let wait_fences = [in_flight_fence];

        self.device.wait_for_fences(&wait_fences, true, u64::MAX).unwrap();

        let result = self.data.swapchain_loader.as_ref().unwrap().acquire_next_image(self.data.swapchain, u64::MAX, image_available_semaphores, vk::Fence::null());
        
        let image_index = match result {
            Ok((image_index, _)) => image_index,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => return unsafe { self.recreate_swapchain() },
            Err(e) => return Err(anyhow!("MAIN: {}", e)),
        };
        self.device.reset_fences(&wait_fences).unwrap();

        self.update_command_buffer(image_index as usize);
        self.update_uniform_buffer(image_index);     
        let device = &self.device;
        let wait_semaphores = [image_available_semaphores];
        let signal_sempahores = [render_finished_semaphores];
        {
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = [self.data.command_buffers[image_index as usize]];
            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_sempahores);
            let submit_infos = [submit_info];
            
            unsafe { device.queue_submit(self.data.graphics_queue, &submit_infos, in_flight_fence).unwrap() };
        }
        
        let swapchains = [self.data.swapchain];
        let image_indices = [image_index];
    
        {
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_sempahores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);
            let result = unsafe { self.data.swapchain_loader.as_ref().unwrap().queue_present(self.data.present_queue, &present_info) };
            match result {
                Ok(true) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    return self.recreate_swapchain()
                }
                Err(error) => panic!("Faield to present queu"),
                _ => {}
            }
        }

        Ok(true)
    }

    // Update Command Buffer
    unsafe fn update_command_buffer(&mut self, image_index: usize) -> Result<()> {
        // Reset
        let command_pool = self.data.command_pools[image_index as usize];
        (unsafe { self.device.reset_command_pool(command_pool, vk::CommandPoolResetFlags::empty()) })?;
        let command_buffer = self.data.command_buffers[image_index as usize];

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
        let model = Mat4::from_translation(vec3(0.0, y, z)) * Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), Deg(0.0) );
        let model_bytes = unsafe { std::slice::from_raw_parts(&model as *const Mat4 as *const u8, size_of::<Mat4>()) };
        let opacity = 1 as f32;
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
    unsafe fn update_uniform_buffer(&mut self, image_index: u32) -> Result<()> {
        // Camera
/*         if self.left_clicked && self.cursor_delta.is_some() {
            let delta = self.cursor_delta.take().unwrap();
            let x_ratio = delta[0] as f32 / self.data.swapchain_extent.width as f32;
            let y_ratio = delta[1] as f32 / self.data.swapchain_extent.height as f32;
            let theta = x_ratio * 180.0_f32.to_radians();
            let phi = y_ratio * 90.0_f32.to_radians();
            self.camera.rotate(theta, phi);
        }

        if let Some(wheel_delta) = self.wheel_delta {
            self.camera.foward(wheel_delta * 0.01);
        } */
        
        // MVP
/*         let aspect =self.data.swapchain_extent.width as f32 / self.data.swapchain_extent.height as f32;
        let view = Mat4::look_at_rh(
            // point3::<f32>(6.0, 0.0, 2.0), 
            self.camera.position(),
            point3::<f32>(0.0, 0.0, 0.0), 
            vec3(0.0, 0.0, 1.0));
        let correction = Mat4::new(
            1.0, 0.0, 0.0, 0.0, 
            0.0, -1.0, 0.0, 0.0, 
            0.0, 0.0, 1.0 / 2.0, 0.0, 
            0.0, 0.0, 1.0 / 2.0, 1.0);
        let proj = correction * cgmath::perspective(Deg(45.0), aspect, 0.1, 40.0); */
        let fov_angle = PI / 3.0;
        let aspect_ratio = self.data.swapchain_extent.width as f32 / self.data.swapchain_extent.height as f32;
        let near = 0.1;
        let far = 100.0;
        let projection_matrix = math::perspective(fov_angle, aspect_ratio, far, near);
        let proj = math::matrix_mult(projection_matrix, math::scale(1.0, -1.0, -1.0));
        let rotation = matrix_mult(rotate_x(-self.camera.rot_x), rotate_y(self.camera.rot_y));
        let rotation =  matrix_mult(rotation, rotate_z(self.camera.rot_z));
        let proj = Mat4::new(proj[0], proj[1], proj[2], proj[3], proj[4], proj[5], proj[6], proj[7], proj[8], proj[9], proj[10], proj[11], proj[12], proj[13], proj[14], proj[15]);
        let view = math::matrix_mult(rotation, translate(-self.camera.pos_x, -self.camera.pos_y, self.camera.pos_z));
        let view = Mat4::new(view[0], view[1], view[2], view[3], view[4], view[5], view[6], view[7], view[8], view[9], view[10], view[11], view[12], view[13], view[14], view[15]);

        let ubo = UniformBufferObject { view, proj };
        
        let ubos = [ubo];
        let size = size_of::<UniformBufferObject>() as vk::DeviceSize;
        (unsafe { self.data.allocator.as_mut().unwrap().map_memory(&mut self.data.uniform_allocations[image_index as usize]) })?;
        let data_ptr = self.data.allocator.as_ref().unwrap().get_allocation_info(&self.data.uniform_allocations[image_index as usize]).mapped_data;
        let mut align = unsafe { ash::util::Align::new(data_ptr, align_of::<f32>() as _, size) };
        align.copy_from_slice(&ubos);
        unsafe { self.data.allocator.as_mut().unwrap().unmap_memory(&mut self.data.uniform_allocations[image_index as usize]) };

        Ok(())
    }

    // Recreate the Swapchain
    unsafe fn recreate_swapchain(&mut self) -> Result<bool> {
        unsafe { self.device.device_wait_idle() }?;
        unsafe { self.destroy_swapchain() };
        let dimensions = self.resize_dimensions;
        create_swapchain(&mut self.data, &self.instance, &self.device, dimensions);
        create_swapchain_image_views(&self.device, &mut self.data)?;
        create_render_pass(&self.instance, &self.device, &mut self.data)?;
        create_pipeline(&self.device, &mut self.data)?;
        create_color_objects(&self.instance, &self.device, &mut self.data)?;
        create_depth_objects(&self.instance, &self.device, &mut self.data)?;
        create_framebuffers(&self.device, &mut self.data)?;
        self.data.images_in_flight.resize(self.data.swapchain_images.len(), vk::Fence::null());
        Ok(false)

    }

    // Destroy the Vulkan app
    unsafe fn destroy(&mut self) {
        unsafe { self.device.device_wait_idle().unwrap() };
        
        unsafe { self.destroy_swapchain() };

        let allocator = self.data.allocator.as_mut().unwrap();

        self.data.in_flight_fences.iter().for_each(|f| unsafe { self.device.destroy_fence(*f, None) });
        self.data.render_finished_semaphores.iter().for_each(|s| unsafe { self.device.destroy_semaphore(*s, None) });
        self.data.image_available_semaphores.iter().for_each(|s| unsafe { self.device.destroy_semaphore(*s, None) });
        self.data.command_pools.iter().for_each(|p| unsafe { self.device.destroy_command_pool(*p, None) });
        unsafe { allocator.destroy_buffer(self.data.index_buffer, self.data.index_allocation.as_mut().unwrap()) };
        unsafe { allocator.destroy_buffer(self.data.vertex_buffer, self.data.vertex_allocation.as_mut().unwrap()) };
        let ubo_count= self.data.uniform_buffers.len() ;
        for n in 0..ubo_count {
            let allocation = &mut self.data.uniform_allocations;
            unsafe { allocator.destroy_buffer(self.data.uniform_buffers[n as usize], &mut allocation[n as usize]) };
        }
        unsafe { self.device.destroy_sampler(self.data.texture_sampler, None) };
        unsafe { self.device.destroy_image_view(self.data.texture_image_view, None) };
        unsafe { self.device.free_memory(self.data.texture_image_memory, None) };
        unsafe { self.device.destroy_image(self.data.texture_image, None) };
        unsafe { self.device.destroy_command_pool(self.data.command_pool, None) };
        self.device.destroy_descriptor_pool(self.data.descriptor_pool, None);
        unsafe { self.device.destroy_descriptor_set_layout(self.data.descriptor_set_layout, None) };
        unsafe { self.device.destroy_device(None) };
        unsafe { self.data.surface_loader.as_ref().unwrap().destroy_surface(self.data.surface, None) };
        
        if VALIDATION_ENABLED {
            self.data.debug_utils_loader.as_mut().unwrap().destroy_debug_utils_messenger(self.data.debug_call_back, None);
        }
        
        unsafe { self.instance.destroy_instance(None) };

        
    }

    // Destroy Swapchain
    unsafe fn destroy_swapchain(&mut self) {
        unsafe { self.device.destroy_image_view(self.data.depth_image_view, None) };
        unsafe { self.device.free_memory(self.data.depth_image_memory, None) };
        self.device.destroy_image(self.data.depth_image, None);
        unsafe { self.device.destroy_image_view(self.data.color_image_view, None) };
        unsafe { self.device.free_memory(self.data.color_image_memory, None) };
        unsafe { self.device.destroy_image(self.data.color_image, None) };
        self.data.framebuffers.iter().for_each(|f| unsafe { self.device.destroy_framebuffer(*f, None) });
        unsafe { self.device.destroy_pipeline(self.data.pipeline, None) };
        unsafe { self.device.destroy_pipeline_layout(self.data.pipeline_layout, None) };
        unsafe { self.device.destroy_render_pass(self.data.render_pass, None) };
        self.data.swapchain_image_views.iter().for_each(|v| unsafe { self.device.destroy_image_view(*v, None) });
        unsafe { self.data.swapchain_loader.as_ref().unwrap().destroy_swapchain(self.data.swapchain, None) };
    }

    //
}

#[derive(Default)]
struct EngineData {
    // Debug
    debug_call_back: vk::DebugUtilsMessengerEXT,
    debug_utils_loader: Option<ash::ext::debug_utils::Instance>,

    // Surface
    surface: vk::SurfaceKHR,
    surface_format: vk::SurfaceFormatKHR,
    surface_loader: Option<surface::Instance>,

    // Physical & Logical Device
    physical_device: vk::PhysicalDevice,
    msaa_samples: vk::SampleCountFlags,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    // device_extension_names_raw: vk::PhysicalDeviceFeatures,

    // Swapchain
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,
    swapchain_loader: Option<swapchain::Device>,

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
    vertex_allocation: Option<Allocation>,
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    index_allocation: Option<Allocation>,
    index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,
    uniform_allocations: Vec<Allocation>,
    uniform_buffers: Vec<vk::Buffer>,
    uniform_buffers_memory: Vec<DeviceMemory>,

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
    allocator: Option<Allocator>,
}
