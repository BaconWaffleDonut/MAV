use std::{borrow::Cow, collections::{HashMap, HashSet}, ffi::{self, CStr}, fs::File, io::{BufReader, Cursor}, ops::Index};
use core::ffi::c_char;
use ash::{
    Device, Entry, Instance, ext::debug_utils, khr::{surface, swapchain}, util::read_spv, vk::{self, PhysicalDevice, SurfaceKHR} };
use cgmath::{vec3, vec2};
use winit::{
    event_loop::EventLoop,
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,};
use anyhow::{Ok, Result, anyhow};
use thiserror::Error;
use crate::EngineData;

const APP_NAME: &CStr = c"Testing";
const ENGINE_NAME: &CStr = c"M.A.V.";
const VALIDATION_ENABLED: bool = cfg!(debug_assertions);
const VALIDATION_LAYERS: [&CStr; 1] = [c"VK_LAYER_KHRONOS_validation"];

pub fn test() -> Result<()> {
    if VALIDATION_ENABLED {
        println!("Validation enabled. Importation successful")
    } else {
        println!("Validation not enabled. Importation successful")
    }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct QueueFamilyIndices {
    present: u32,
    graphics: u32,
}

impl QueueFamilyIndices {
    unsafe fn get(entry: &Entry, instance: &Instance, physical_device: vk::PhysicalDevice, window: &dyn Window) -> Result<Self> {
        let event_loop = EventLoop::new()?;
        let surface = unsafe{ash_window::create_surface(&entry, &instance, event_loop.display_handle()?.as_raw(), window.window_handle()?.as_raw(), None)}.expect("Failed to create surface.");
        let surface_loader = surface::Instance::new(&entry, &instance);
        let properties = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        let graphics = properties
            .iter()
            .position(|p| p.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|i| i as u32);
        let mut present = None;
        for (index, properties) in properties.iter().enumerate() {
            if unsafe { surface_loader.get_physical_device_surface_support(physical_device, index as u32, surface)? } {
                present = Some(index as u32);
                break;
            }
        }
        if let (Some(graphics), Some(present)) = (graphics, present) {
            Ok(Self {graphics, present })
        } else {
            Err(anyhow!(SuitabilityError("Missing required queue families.")))
        }
    }
}

const MAX_FRAMES_IN_FLIGHT: usize = 3;

type Vec2 = cgmath::Vector2<f32>;
type Vec3 = cgmath::Vector3<f32>;
type Mat4 = cgmath::Matrix4<f32>;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pos: Vec3,
    color: Vec3,
    tex_coord: Vec2,
}

impl Vertex {
    fn new(pos: Vec3, color: Vec3, tex_coord: Vec2) -> Self {
        Self { pos, color, tex_coord }
    }
    fn binding_descriptions() -> vk::VertexInputBindingDescription {
        let binding_descriptions = vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX);
        return binding_descriptions;
    }
    fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
        let pos = vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0);
        let color = vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(size_of::<Vec3>() as u32);
        let tex_coord = vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(2)
            .format(vk::Format::R32G32_SFLOAT)
            .offset((size_of::<Vec3>() + size_of::<Vec3>()) as u32);
        [pos, color, tex_coord]
    }
}

struct Utils {
}

impl Utils {
    pub fn event_loop() -> Result<EventLoop> {
        let event_loop = EventLoop::new()?;
        Ok(event_loop)
    }
    pub fn surface(entry: &Entry, window: &dyn Window, instance: &Instance,) -> Result<SurfaceKHR> {
        let event_loop = Utils::event_loop().expect("Failed to call event loop");
        let surface = unsafe{ash_window::create_surface(&entry, &instance, event_loop.display_handle()?.as_raw(), window.window_handle()?.as_raw(), None)}.expect("Failed to create surface.");
        Ok(surface)
    }
    pub fn surface_loader(entry: &Entry, instance: &Instance) -> Result<surface::Instance> {
        let surface_loader = surface::Instance::new(&entry, &instance);
        Ok(surface_loader)
    }
}

#[derive(Debug, Error)]
#[error("{0}")]
pub struct SuitabilityError(pub &'static str);

//====================
// Instance
//====================

pub unsafe fn create_instance() -> Result<Instance> {
    let entry = unsafe{Entry::load().expect("Failed to load vulkan Entry.")};
    let event_loop = Utils::event_loop().expect("Failed to fetch event loop.");
    // Application Info
    let application_info = vk::ApplicationInfo::default()
        .application_name(APP_NAME)
        .application_version(0)
        .engine_name(ENGINE_NAME)
        .engine_version(0)
        .api_version(vk::make_api_version(0, 1, 0, 0));

    // Layers
    let layer_names = VALIDATION_LAYERS;
    let layer_names_raw: Vec<*const c_char> = layer_names
        .iter()
        .map(|raw_name| raw_name.as_ptr())
        .collect();

    // Extensions
    let mut extension_names = 
        ash_window::enumerate_required_extensions(event_loop.display_handle()?.as_raw())
            .unwrap()
            .to_vec();
        extension_names.push(debug_utils::NAME.as_ptr());

    // Create Instance
    let create_flags = vk::InstanceCreateFlags::default();

    let mut create_info = vk::InstanceCreateInfo::default()
        .application_info(&application_info)
        .enabled_layer_names(&layer_names_raw)
        .enabled_extension_names(&extension_names)
        .flags(create_flags);

    // Debug Messenger

    let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION  | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .pfn_user_callback(Some(vulkan_debug_callback));
        
    if VALIDATION_ENABLED {
        create_info = create_info.push_next(&mut debug_info);
    }

    let instance: Instance = unsafe {entry
        .create_instance(&create_info, None)
        .expect("Failed to create Instace.")};
    
    let debug_utils_loader = debug_utils::Instance::new(&entry, &instance);
    let debug_call_back = unsafe{debug_utils_loader
        .create_debug_utils_messenger(&debug_info, None)
        .unwrap()};



    Ok(instance)
}

// Debug Callback

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = unsafe{ *p_callback_data};
    let message_id_number = callback_data.message_id_number;

    let message_id_name = unsafe { if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        ffi::CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    }};

    let message = unsafe { if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        ffi::CStr::from_ptr(callback_data.p_message).to_string_lossy()
    }};

    println!(
        "{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n",
    );

    vk::FALSE
}

//====================
// Physical Device
//====================

pub unsafe fn pick_physical_device(instance: &Instance, entry: &Entry, window: &dyn Window) -> Result<(u32, PhysicalDevice)> {
    // Import surface and surface loader to get requirements. 
    let surface = Utils::surface(&entry, window, &instance).expect("Failed fetching surface.");
    let surface_loader = Utils::surface_loader(&entry, &instance).expect("Failed fetching surface loader.");

    // Select and check physical device.
    let physical_devices = unsafe{instance.enumerate_physical_devices().expect("Physical Device Error")};
    let (physical_device, queue_family_index) = physical_devices
        .iter()
        .find_map(|physical_device| {
            unsafe { instance
                .get_physical_device_queue_family_properties(*physical_device)
                .iter()
                .enumerate()
                .find_map(|(index, info)| {
                    let supports_graphics_and_surface = 
                        info.queue_flags.contains(vk::QueueFlags::GRAPHICS) && surface_loader.get_physical_device_surface_support(*physical_device, index as u32, surface).unwrap();
                    if supports_graphics_and_surface {
                        Some((*physical_device, index))
                    } else {
                        None
                    }
                }) }
        }).expect("Failed to find suitable phyiscal device.");

        let queue_family_index = queue_family_index as u32;

        Ok((queue_family_index, physical_device))
 }

/* pub fn check_physical_device_extensions() -> Result<([i8], PhysicalDeviceFeatures/*figure out how to pass out features*/)> {
    let device_extension_names_raw = [
        swapchain::NAME.as_ptr(),
    ];
    let features = vk::PhysicalDeviceFeatures {
        shader_clip_distance: 1,
        ..Default::default()
    };
    Ok((device_extension_names_raw, features))
} */
// Rebuild this section now that I managed to implement QueueFamilyIndices

pub fn get_msaa_samples(instance: &Instance, data: &EngineData) -> vk::SampleCountFlags {
    let properties = unsafe { instance.get_physical_device_properties(data.physical_device) };
    let counts = properties.limits.framebuffer_color_sample_counts & properties.limits.framebuffer_depth_sample_counts;
    let sample_counts = [
        vk::SampleCountFlags::TYPE_64,
        vk::SampleCountFlags::TYPE_32,
        vk::SampleCountFlags::TYPE_16,
        vk::SampleCountFlags::TYPE_8,
        vk::SampleCountFlags::TYPE_4,
        vk::SampleCountFlags::TYPE_2,
    ]
    .iter()
    .cloned()
    .find(|c| counts.contains(*c))
    .unwrap_or(vk::SampleCountFlags::TYPE_1);
    return sample_counts;
}

//====================
// Logical Device
//====================

pub fn create_logical_device(entry: &Entry, instance: &Instance, window: &dyn Window, physical_device: vk::PhysicalDevice) -> Result<Device> {
    // Queue Create Info
    let indices = unsafe { QueueFamilyIndices::get(&entry, &instance, physical_device, window)? };
    let mut unique_indices = HashSet::new();
    unique_indices.insert(indices.graphics);
    unique_indices.insert(indices.present);

    let queue_priorities = &[1.0];
    let queue_infos = unique_indices
        .iter()
        .map(|i| {
            vk::DeviceQueueCreateInfo::default()
                .queue_family_index(*i) 
                .queue_priorities(queue_priorities) 
        }).collect::<Vec<_>>();

    // Extensions
    let extensions = [swapchain::NAME.as_ptr()];
    
    // Features
    let features  = vk::PhysicalDeviceFeatures {
        shader_clip_distance: 1,
        ..Default::default()
    };

    // Create

    let create_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(std::slice::from_ref(&queue_infos.index(1)))
        .enabled_extension_names(&extensions)
        .enabled_features(&features);
    
    let device = unsafe { instance
        .create_device(physical_device, &create_info, None)
        .unwrap() };

    Ok(device)
}

//====================
// Swapchain
//====================

pub fn create_swapchain(window: &dyn Window, instance: &Instance, device: &Device, data: &mut EngineData, entry: &Entry) -> Result<()> {
    // Setup
    let surface_loader = Utils::surface_loader(&entry, &instance).expect("Failed fetching surface loader.");
    let indices = unsafe { QueueFamilyIndices::get(entry, instance, data.physical_device, window) }?;
    let surface_capabilites = unsafe { surface_loader 
        .get_physical_device_surface_capabilities(data.physical_device, data.surface).unwrap() };
    data.surface_format = unsafe {
        surface_loader.get_physical_device_surface_formats(data.physical_device, data.surface).unwrap()[0]
    };
    let mut image_count = surface_capabilites.min_image_count + 1;
    if surface_capabilites.max_image_count != 0 && image_count > surface_capabilites.max_image_count {
        image_count = surface_capabilites.max_image_count;
    } else {
        vk::SharingMode::EXCLUSIVE;
    };
    let surface_resolution = match surface_capabilites.current_extent.width {
        u32::MAX => vk::Extent2D {
            width: data.window_width,
            height: data.window_height,
        },
        _ => surface_capabilites.current_extent,
    };
    let pre_transform = if surface_capabilites
        .supported_transforms
        .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            surface_capabilites.current_transform
        };
    let present_modes = unsafe { surface_loader
        .get_physical_device_surface_present_modes(data.physical_device, data.surface)
        .unwrap() };
    let present_mode = present_modes
        .iter()
        .cloned()
        .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
        .unwrap_or(vk::PresentModeKHR::FIFO);
    let mut queue_family_indices = vec![];
    let image_sharing_mode = if indices.graphics != indices.present {
        queue_family_indices.push(indices.graphics);
        queue_family_indices.push(indices.present);
        vk::SharingMode::CONCURRENT
    } else {
        vk::SharingMode::EXCLUSIVE
    };
    let swapchain_loader = swapchain::Device::new(&instance, &device);

    // Create
    let info =  vk::SwapchainCreateInfoKHR::default()
        .surface(data.surface)
        .min_image_count(image_count)
        .image_format(data.surface_format.format)
        .image_color_space(data.surface_format.color_space)
        .image_extent(surface_resolution)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(image_sharing_mode)
        .queue_family_indices(&queue_family_indices)
        .pre_transform(pre_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true)
        .old_swapchain(vk::SwapchainKHR::null());

    data.swapchain = unsafe { swapchain_loader.create_swapchain(&info, None).unwrap() };
    data.swapchain_images = unsafe {swapchain_loader.get_swapchain_images(data.swapchain)?};

    Ok(()) 

}

unsafe fn create_swapchain_image_views(device: &Device, data: &mut EngineData) -> Result<()> {
    let present_image_views: Vec<vk::ImageView> = data.swapchain_images
        .iter()
        .map(|&image| {
            let info = vk::ImageViewCreateInfo::default()
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(data.surface_format.format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::R,
                    g: vk::ComponentSwizzle::G,
                    b: vk::ComponentSwizzle::B,
                    a: vk::ComponentSwizzle::A,
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .image(image);
                unsafe { device.create_image_view(&info, None).unwrap() }
        })
        .collect();
        
    data.swapchain_image_views = present_image_views;

    Ok(())
}

//====================
// Pipeline
//====================

pub fn create_render_pass(instance: &Instance, device: &Device, data: &mut EngineData) -> Result<()> {
    // Attachements
    let color_attachment = vk::AttachmentDescription::default()
        .format(data.swapchain_format)
        .samples(data.msaa_samples)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
    let depth_stencil_attachement = vk::AttachmentDescription::default()
        .format(get_depth_format(instance, data)?)
        .samples(data.msaa_samples)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
    let color_resolve_attachment = vk::AttachmentDescription::default()
        .format(data.swapchain_format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::DONT_CARE)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);
    
    // Subpasses
    let color_attachment_ref = vk::AttachmentReference::default()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
    let depth_stencil_attachment_ref = vk::AttachmentReference::default()
        .attachment(1)
        .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
    let color_resolve_attachment_ref = vk::AttachmentReference::default()
        .attachment(2)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let color_attachments = &[color_attachment_ref];
    let resolve_attachments = &[color_resolve_attachment_ref];
    let subpass = vk::SubpassDescription::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(color_attachments)
        .depth_stencil_attachment(&depth_stencil_attachment_ref)
        .resolve_attachments(resolve_attachments);

    // Dependencies
    let dependency = vk::SubpassDependency::default()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE);
    
    // Create
    let attachments = &[color_attachment, depth_stencil_attachement, color_resolve_attachment];
    let subpasses = &[subpass];
    let dependencies = &[dependency];
    let info = vk::RenderPassCreateInfo::default()
        .attachments(attachments)
        .subpasses(subpasses)
        .dependencies(dependencies);

    data.render_pass = unsafe { device.create_render_pass(&info, None)? };
    
    Ok(())

}

pub fn create_descriptor_set_layout(device: &Device, data: &mut EngineData) -> Result<()> {
    let ubo_binding = vk::DescriptorSetLayoutBinding::default()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX);
    let sampler_binding = vk::DescriptorSetLayoutBinding::default()
        .binding(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);
    let bindings = &[ubo_binding, sampler_binding];
    let info = vk::DescriptorSetLayoutCreateInfo::default().bindings(bindings);
    data.descriptor_set_layout = unsafe { device.create_descriptor_set_layout(&info, None) }?;

    Ok(())
}

pub fn create_pipeline(device: &Device, data: &mut EngineData) -> Result<()> {
    // Stages

    let mut vert_spv = Cursor::new(&include_bytes!("../shader/texture/vert.spv")); //need to create folders and files
    let mut frag_spv = Cursor::new(&include_bytes!("../shader/texture/frag.spv"));
    let vert_code = read_spv(&mut vert_spv).expect("Failed to read Vertex SPV file.");
    let frag_code = read_spv(&mut frag_spv).expect("Failed to read Fragment SPV file.");
    let vert_shader_info = vk::ShaderModuleCreateInfo::default().code(&vert_code);
    let frag_shader_info = vk::ShaderModuleCreateInfo::default().code(&frag_code);
    let vert_shader_module = unsafe { device.create_shader_module(&vert_shader_info, None).expect("Shader module error: Vertex.") };
    let frag_shader_module = unsafe { device.create_shader_module(&frag_shader_info, None).expect("Shader module error: Fragment.") };
    

    let vert_stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_shader_module)
        .name(c"main");
    let frag_stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_shader_module)
        .name(c"main");

    // Vertex Input State
    let binding_descriptions = &[Vertex::binding_descriptions()];
    let attribute_descriptions = Vertex::attribute_descriptions();
    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(binding_descriptions)
        .vertex_attribute_descriptions(&attribute_descriptions);

    // Input Assembly
    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);

    // Viewport State 
    let viewport = vk::Viewport::default()
        .x(0.0)
        .y(0.0)
        .width(data.swapchain_extent.width as f32)
        .height(data.swapchain_extent.height as f32)
        .min_depth(0.0)
        .max_depth(1.0);
    let scissor = vk::Rect2D::default()
        .offset(vk::Offset2D {x: 0, y: 0} )
        .extent(data.swapchain_extent);

    let viewports = &[viewport];
    let scissors = &[scissor];
    let viewport_state = vk::PipelineViewportStateCreateInfo::default()
        .viewports(viewports)
        .scissors(scissors);

    // Rasterization State
    let rasterization_state = vk::PipelineRasterizationStateCreateInfo::default()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_bias_enable(false);

    // Multisample State
    let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
        .sample_shading_enable(true)
        .min_sample_shading(0.2)
        .rasterization_samples(data.msaa_samples);
    
    // Depth Stencil State
    let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::default()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS)
        .depth_bounds_test_enable(false)
        .stencil_test_enable(false);

    // Color Blend State
    let attachement = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD);
    let attachments = &[attachement];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0]);

    // Push Constant Ranges
    let vert_push_constant_range = vk::PushConstantRange::default()
        .stage_flags(vk::ShaderStageFlags::VERTEX)
        .offset(0)
        .size(64);
    let frag_push_constant_range = vk::PushConstantRange::default()
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        .offset(64)
        .size(4);

    // Layout
    let set_layouts = &[data.descriptor_set_layout];
    let push_constant_ranges = &[vert_push_constant_range, frag_push_constant_range];
    let layout_info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(set_layouts)
        .push_constant_ranges(push_constant_ranges);

    data.pipeline_layout = unsafe { device.create_pipeline_layout(&layout_info, None)? };

    // Dynamic State 
    let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_state);

    // Create

    let stages = &[vert_stage, frag_stage];
    let info = vk::GraphicsPipelineCreateInfo::default()
        .stages(stages)
        .vertex_input_state(&vertex_input_state)
        .input_assembly_state(&input_assembly_state)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization_state)
        .multisample_state(&multisample_state)
        .depth_stencil_state(&depth_stencil_state)
        .color_blend_state(&color_blend_state)
        .dynamic_state(&dynamic_state_info)
        .layout(data.pipeline_layout)
        .render_pass(data.render_pass)
        .subpass(0);

    let graphics_pipelines = unsafe { device.create_graphics_pipelines(vk::PipelineCache::null(), &[info], None).unwrap() };
    data.pipeline = graphics_pipelines[0];

    // Cleanup
    unsafe { device.destroy_shader_module(vert_shader_module, None) };
    unsafe { device.destroy_shader_module(frag_shader_module, None) };

    Ok(())
}

//====================
// Framebuffers
//====================

pub fn create_framebuffers(device: &Device, data: &mut EngineData) -> Result<()> {
    data.framebuffers = data.swapchain_image_views
        .iter()
        .map(|i| {
            let attachments = &[data.color_image_view, data.depth_image_view, *i];
            let info = vk::FramebufferCreateInfo::default()
                .render_pass(data.render_pass)
                .attachments(attachments)
                .width(data.swapchain_extent.width)
                .height(data.swapchain_extent.height)
                .layers(1);
            unsafe { device.create_framebuffer(&info, None) }
        }).collect::<Result<Vec<_>, _>>()?;
    Ok(())
}

//====================
// Command Pool
//====================

pub fn create_command_pools(instance: &Instance, device: &Device, data: &mut EngineData) -> Result<()> {
    // Global 
    data.command_pool = create_command_pool(instance, device, data)?;

    // Per-Framebuffer
    let num_images = data.swapchain_images.len();
    for _ in 0..num_images {
        let command_pool = create_command_pool(instance, device, data)?;
        data.command_pools.push(command_pool);
    }
    Ok(())
}

pub fn create_command_pool(instance: &Instance, device: &Device, data: &mut EngineData, entry: &Entry, window: &dyn Window) -> Result<vk::CommandPool> {
    let indices = unsafe { QueueFamilyIndices::get(entry, instance, data.physical_device, window)? };
    let info = vk::CommandPoolCreateInfo::default()
        .flags(vk::CommandPoolCreateFlags::TRANSIENT)
        .queue_family_index(indices.graphics);
    Ok(unsafe { device.create_command_pool(&info, None)? })
}

//====================
// Color Objects
//====================

pub fn create_color_objects(instance: &Instance, device: &Device, data: &mut EngineData) -> Result<()> {
    // Image + Image Memory
    let (color_image, color_image_memory) = create_image(
        instance, 
        device, 
        data, 
        data.swapchain_extent.width, 
        data.swapchain_extent.height, 
        1, 
        data.msaa_samples, 
        data.swapchain_format, 
        vk::ImageTiling::OPTIMAL, 
        vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT, 
        vk::MemoryPropertyFlags::DEVICE_LOCAL)?;

    data.color_image = color_image;
    data.color_image_memory = color_image_memory;

    // Image View
    data.color_image_view = create_image_view(
        device, 
        data.color_image, 
        data.swapchain_format, 
        vk::ImageAspectFlags::COLOR, 
        1)?;
    
    Ok(())
}

//====================
// Depth Objects
//====================

pub fn create_depth_objects(instance: &Instance, device: &Device, data: &mut EngineData) -> Result<()> {
    // Image + Image Memory
    let format = get_depth_format(instance, data)?;
    let (depth_image, depth_image_memory) = create_image(
        instance, 
        device, 
        data, 
        data.swapchain_extent.width, 
        data.swapchain_extent.height, 
        1, 
        data.msaa_samples, 
        format, 
        vk::ImageTiling::OPTIMAL, 
        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT, 
        vk::MemoryPropertyFlags::DEVICE_LOCAL)?;

        data.depth_image = depth_image;
        data.depth_image_memory = depth_image_memory;

        data.depth_image_view = create_image_view(device, data.depth_image, format, vk::ImageAspectFlags::DEPTH, 1)?;

        Ok(())
}

pub fn get_depth_format(instance: &Instance, data: &EngineData) -> Result<vk::Format> {
    let candidates = &[
        vk::Format::D32_SFLOAT,
        vk::Format::D32_SFLOAT_S8_UINT,
        vk::Format::D24_UNORM_S8_UINT,
    ];

    get_supported_format(
        instance, 
        data, 
        candidates, 
        vk::ImageTiling::OPTIMAL, 
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT)
}

pub fn get_supported_format(instance: &Instance, data: &EngineData, candidates: &[vk::Format], tiling: vk::ImageTiling, features: vk::FormatFeatureFlags) -> Result<vk::Format> {
    candidates
        .iter()
        .cloned()
        .find(|f| {
            let properties = unsafe { instance.get_physical_device_format_properties(data.physical_device, *f) };
            match tiling {
                vk::ImageTiling::LINEAR => properties.linear_tiling_features.contains(features),
                vk::ImageTiling::OPTIMAL => properties.optimal_tiling_features.contains(features),
                _ => false,
            }   
        })
        .ok_or_else(|| anyhow!("Failed to find supported format!"))
}

//====================
// Texture
//====================

pub fn create_texture_image(instance: &Instance, device: &Device, data: &mut EngineData) -> Result<()> {
    // Load 
    let image = File::open("../resources/viking_room.png")?;
    let decoder = png::Decoder::new(image);
    
    let mut reader = decoder.read_info()?;
    let mut pixels = vec![0; reader.info().raw_bytes()];
    reader.next_frame(&mut pixels)?;
    
    let size = reader.info().raw_bytes() as u64;
    let (width, height) = reader.info().size();
    data.mip_levels = (width.max(height) as f32).log2().floor() as u32 + 1;

    if width != 1024 || height != 1024 || reader.info().color_type != png::ColorType::Rgba {
        panic!("Invalid texture image used.")
    }

    // Create Staging
    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance, 
        device, 
        data, 
        size, 
        vk::BufferUsageFlags::TRANSFER_SRC, 
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE)?;

    // Copy Stating
    let memory = unsafe { device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty()) }?;
    memcpy(pixels.as_ptr(), memory.cast(), pixels.len());
    unsafe { device.unmap_memory(staging_buffer_memory) };

    // Create Image
    let (texture_image, texture_image_memory) = create_image(
        instance, 
        device, 
        data, 
        width, 
        height, 
        data.mip_levels, 
        vk::SampleCountFlags::TYPE_1, 
        vk::Format::R8G8B8A8_SRGB, 
        vk::ImageTiling::OPTIMAL, 
        vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags:: TRANSFER_SRC, 
        vk::MemoryPropertyFlags::DEVICE_LOCAL)?;
    data.texture_image = texture_image;
    data.texture_image_memory = texture_image_memory;

    // Transition + Copy Image
    transition_image_layout(
        device, 
        data, 
        data.texture_image, 
        vk::Format::R8G8B8A8_SRGB, 
        vk::ImageLayout::UNDEFINED, 
        vk::ImageLayout::TRANSFER_DST_OPTIMAL, 
        data.mip_levels)?;
    copy_buffer_to_image(device, data, staging_buffer, data.texture_image, width, height)?;

    // Cleanup
    unsafe { device.destroy_buffer(staging_buffer, None) };
    unsafe { device.free_memory(staging_buffer_memory, None) };

    // Mipmaps
    generate_mipmaps(
        instance, 
        device, 
        data, 
        data.texture_image, 
        vk::Format::R8G8B8A8_SRGB, 
        width, 
        height, 
        data.mip_levels)?;
    
    Ok(())
}

pub fn generate_mipmaps(instance: &Instance, device: &Device, data: &EngineData, image: vk::Image, format: vk::Format, width: u32, height: u32, mip_levels: u32) -> Result<()> {
    // Support
    if unsafe { !instance
        .get_physical_device_format_properties(data.physical_device, format)
        .optimal_tiling_features
        .contains(vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR)}
        {
            return Err(anyhow!("Texture Image format does not support linear blitting."));
        }

        // Mipmaps
        let command_buffer = begin_single_time_commands(device, data)?;
        let subresource = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_array_layer(0)
            .layer_count(1)
            .level_count(1);
        let mut barrier = vk::ImageMemoryBarrier::default()
            .image(image)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .subresource_range(subresource);
        let mut mip_width = width;
        let mut mip_height = height;

        for i in 1..mip_levels {
            barrier.subresource_range.base_mip_level = i - 1;
            barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
            barrier.new_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            barrier.dst_access_mask = vk::AccessFlags::TRANSFER_READ;

            unsafe { device.cmd_pipeline_barrier(
                command_buffer, 
                vk::PipelineStageFlags::TRANSFER, 
                vk::PipelineStageFlags::TRANSFER, 
                vk::DependencyFlags::empty(), 
                &[] as &[vk::MemoryBarrier], 
                &[] as &[vk::BufferMemoryBarrier], 
                &[barrier]) };
            
            let src_subresource = vk::ImageSubresourceLayers::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .mip_level(i - 1)
                .base_array_layer(0)
                .layer_count(1);
            let dst_subresource = vk::ImageSubresourceLayers::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .mip_level(i)
                .base_array_layer(0)
                .layer_count(1);
            let blit =  vk::ImageBlit::default()
                .src_offsets([
                    vk::Offset3D {x: 0, y: 0, z: 0},
                    vk::Offset3D {
                        x: mip_width as i32,
                        y: mip_height as i32,
                        z: 1,
                    },
                ])
                .src_subresource(src_subresource)
                .dst_offsets([
                    vk::Offset3D {x:0, y:0, z:0},
                    vk::Offset3D {
                        x: (if mip_width > 1 {mip_width / 2} else {1}) as i32,
                        y: (if mip_height > 1 {mip_height / 2} else {1}) as i32,
                        z: 1,
                    },
                ])
                .dst_subresource(dst_subresource);
            unsafe { device.cmd_blit_image(
                command_buffer, 
                image, 
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL, 
                image, 
                vk::ImageLayout::TRANSFER_DST_OPTIMAL, 
                &[blit], 
                vk::Filter::LINEAR) };
            
            barrier.old_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
            barrier.src_access_mask = vk::AccessFlags::TRANSFER_READ;
            barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

            unsafe { device.cmd_pipeline_barrier(
                command_buffer, 
                vk::PipelineStageFlags::TRANSFER, 
                vk::PipelineStageFlags::FRAGMENT_SHADER, 
                vk::DependencyFlags::empty(), 
                &[] as &[vk::MemoryBarrier], 
                &[] as &[vk::BufferMemoryBarrier], 
                &[barrier]) };
            
            if mip_width > 1 {
                mip_width /= 2;
            } 
            if mip_height > 1 {
                mip_height /= 2;
            }
        }
    barrier.subresource_range.base_mip_level = mip_levels - 1;
    barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
    barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
    barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
    barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;
    unsafe { device.cmd_pipeline_barrier(
        command_buffer, 
        vk::PipelineStageFlags::TRANSFER, 
        vk::PipelineStageFlags::FRAGMENT_SHADER, 
        vk::DependencyFlags::empty(), 
        &[] as &[vk::MemoryBarrier], 
        &[] as &[vk::BufferMemoryBarrier], 
        &[barrier]) };
    
    end_single_time_commands(device, data, command_buffer)?;
            
    Ok(())
}

pub fn create_texture_image_view(device: &Device, data: &mut EngineData) -> Result<()> {
    data.texture_image_view = create_image_view  (
        device, 
        data.texture_image, 
        vk::Format::R8G8B8A8_SRGB, 
        vk::ImageAspectFlags::COLOR, 
        data.mip_levels)?;

    Ok(())
}

pub fn create_texture_sampler(device: &Device, data: &mut EngineData) -> Result<()> {
    let info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::REPEAT)
        .address_mode_v(vk::SamplerAddressMode::REPEAT)
        .address_mode_w(vk::SamplerAddressMode::REPEAT)
        .anisotropy_enable(true)
        .max_anisotropy(16.0)
        .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
        .unnormalized_coordinates(false)
        .compare_enable(false)
        .compare_op(vk::CompareOp::ALWAYS)
        .min_lod(0.0)
        .max_lod(data.mip_levels as f32)
        .mip_lod_bias(0.0);
    data.texture_sampler = unsafe { device.create_sampler(&info, None)? };

    Ok(())

}

//====================
// Model
//====================

pub fn load_model(data: &mut EngineData) -> Result<()> {
    // Model
    let mut reader = BufReader::new(File::open("../resources/viking_room.obj")?);

    let (models, _) = tobj::load_obj_buf(
        &mut reader,
        &tobj::LoadOptions {
            triangulate: true,
            ..Default::default()
        },
        |_| Ok(Default::default()),
    )?;

    // Vertices / Indices
    let mut unique_vertices = HashMap::new();
    for model in &models {
        for index in &model.mesh.indices {
            let pos_offset = (3 * index) as usize;
            let tex_coord_offset = (2 * index) as usize;

            let vertex = Vertex {
                pos: vec3(
                    model.mesh.positions[pos_offset],
                    model.mesh.positions[pos_offset + 1] ,
                    model.mesh.positions[pos_offset + 2],
                    ),
                color: vec3(1.0, 1.0, 1.0),
                tex_coord: vec2(
                    model.mesh.texcoords[tex_coord_offset],
                    1.0 - model.mesh.texcoords[tex_coord_offset + 1],
                ),
            };

            if let Some(index) = unique_vertices.get(&vertex) {
                data.indices.push(*index as u32);
            } else {
                let index = data.vertices.len();
                unique_vertices.insert(vertex, index);
                data.vertices.push(vertex);
                data.indices.push(index as u32);
            }
        }
    }

    Ok(())

}

//====================
// Buffers
//====================

pub fn create_vertex_buffer(instance: &Instance, device: &Device, data: &mut EngineData) -> Result<()> {}

pub fn create_index_buffer(instance: &Instance, device: &Device, data: &mut EngineData) -> Result<()> {}

pub fn create_uniform_buffers(instance: &Instance, device: &Device, data: &mut EngineData) -> Result<()> {}

//====================
// Descriptors
//====================

pub fn create_descriptor_pool(device: &Device, data: &mut EngineData) -> Result<()> {}

pub fn create_descriptor_sets(device: &Device, data: &mut EngineData) -> Result<()> {}

//==================== 
// Command Buffers
//====================

pub fn create_command_buffers(device: &Device, data: &mut EngineData) -> Result<()> {}

//====================
// Sync Objects
//====================

pub fn create_sync_objects(device: &Device, data: &mut EngineData) -> Result<()> {}

//====================
// Shared Buffers
//====================

pub fn create_buffer(instance: &Instance, device: &Device, data: &EngineData, size: vk::DeviceSize, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags) -> Result<(vk::Buffer, vk::DeviceMemory)> {}

pub fn copy_buffer(device: &Device, data: &EngineData, source: vk::Buffer, destination: vk::Buffer, size: vk::DeviceSize) -> Result<()> {}

//====================
// Shared Images
//====================

pub fn create_image(instance: &Instance, device: &Device, data: &EngineData, width: u32, height: u32, mip_levels: u32, samples: vk::SampleCountFlags, format: vk::Format, tiling: vk::ImageTiling, usage: vk::ImageUsageFlags, properties: vk::MemoryPropertyFlags) -> Result<(vk::Image, vk::DeviceMemory)> {
    // Image
    let info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        })
        .mip_levels(mip_levels)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(samples);
    let image = unsafe { device.create_image(&info, None).expect("Failed to create image.")};

    // Memory 
    let requirements = unsafe { device.get_image_memory_requirements(image) };
    let info = vk::MemoryAllocateInfo::default()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(instance, data, properties, requirements).expect("Failed to create info for memory allocation."));
    let image_memory = unsafe { device.allocate_memory(&info, None).expect("Failed to allocate memory for image memory.")};
    (unsafe { device.bind_image_memory(image, image_memory, 0) }).expect("Failed to bind image memory.");
    Ok((image, image_memory))

}

pub fn create_image_view(device: &Device, image: vk::Image, format: vk::Format, aspects: vk::ImageAspectFlags, mip_levels: u32) -> Result<vk::ImageView> {}

pub fn transition_image_layout(device: &Device, data: &EngineData, image: vk::Image, format: vk::Format, old_layout: vk::ImageLayout, new_layout: vk::ImageLayout, mip_levels: u32) -> Result<()> {}

pub fn copy_buffer_to_image(device: &Device, data: &EngineData, buffer: vk::Buffer, image: vk::Image, width: u32, height: u32) -> Result<()> {}

//====================
// Other Shared
//====================

pub fn get_memory_type_index(instance: &Instance, data: &EngineData, properties: vk::MemoryPropertyFlags, requirements: vk::MemoryRequirements) -> Result<u32> {}

pub fn begin_single_time_commands(device: &Device, data: &EngineData) -> Result<vk::CommandBuffer> {}

pub fn end_single_time_commands(device: &Device, data: &EngineData, command_buffer: vk::CommandBuffer) -> Result<()> {}