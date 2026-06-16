use std::{borrow::Cow, error::Error, ffi::{self, CStr}};
use core::{ffi::c_char};
use ash::{
    vk, Device, Entry, Instance, 
    ext::debug_utils, 
};
use log::{warn, info};
use winit::{
    event_loop::EventLoop,
    raw_window_handle::HasDisplayHandle,
    window::Window,
    };
use anyhow::{Result, anyhow};
use crate::EngineData;

const APP_NAME: &CStr = c"Testing";
const ENGINE_NAME: &CStr = c"M.A.V.";
const VALIDATION_ENABLED: bool = cfg!(debug_assertions);

/* #[derive(Debug, Clone, Copy)]
struct QueueFamilyIndices {
    graphics: u32,
    present: u32,
}

impl QueueFamilyIndices {
    unsafe fn get(instance: &Instance, data: &EngineData, physical_device: vk::PhysicalDevice) -> Result<Self> {
        let properties = instance.get_physical_device_queue_family_properties(physical_device);
        let graphics = properties
            .iter()
            .position(|p| p.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|i| i as u32);

        let mut present = None;
        for (index, properties) in properties.iter().enumerate() {
            if instan
        }
    }
} */

pub fn test() {
    println!("Testing importation of functions.")
}

//====================
// Instance
//====================

pub unsafe fn create_instance(entry: &Entry, data: &mut EngineData) -> Result<Instance, Box<dyn Error>> {
    let entry = unsafe{Entry::load().expect("Failed to load vulkan Entry.")};
    let event_loop = EventLoop::new()?;
    // Application Info
    let application_info = vk::ApplicationInfo::default()
        .application_name(APP_NAME)
        .application_version(0)
        .engine_name(ENGINE_NAME)
        .engine_version(0)
        .api_version(vk::make_api_version(0, 1, 0, 0));

    // Layers
    let layer_names = [c"VK_LAYER_KHRONOS_validation"];
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

pub unsafe fn pick_physical_device(instance: &Instance, data: &mut EngineData, physical_device: vk::PhysicalDevice) ->Result<()> {
    for physical_device in instance.enumerate_physical_devices()? {
        let properties = instance.get_physical_device_properties(physical_device);

        if let Err(error) = check_physical_device(instance, data, physical_device) {
            warn!("Skipping physical device {:?}: {}", properties.device_name, error);
        } else {
            info!("Selected phyiscal device {:?}.", properties.device_name);
            data.physical_device = physical_device;
            data.msaa_samples = get_msaa_samples(instance, data);
            return Ok(());
        }
    }
    Err(anyhow!("Failed to find suitable physical device."))
}

/* pub fn check_physical_device(instance: &Instance, data: &EngineData, physical_device: vk::PhysicalDevice) -> Result<()> {
    QueueFamilyIndices::get(instance, data, physical_device)
} */

pub fn check_physical_device_extensions(instance: &Instance, physical_device: vk::PhysicalDevice) -> Result<()> {}

pub fn get_msaa_samples(instance: &Instance, data: &EngineData) -> vk::SampleCountFlags() {}

//====================
// Logical Device
//====================

pub fn create_logical_device(entry: &Entry, instance: &Instance, data: &mut EngineData) -> Result<Device> {}

//====================
// Swapchain
//====================

pub fn create_swapchain(window: &Window, instance: &Instance, device: &Device, data: &mut EngineData) -> Result<()> {}

pub fn get_swapchain_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {}

pub fn get_swapchain_present_mode(present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {}

pub fn get_swapchain_extent(window: &Window, capabilities: vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {}

pub fn create_swapchain_image_views(device: &Device, data: &mut EngineData) -> Result<()> {}

//====================
// Pipeline
//====================

pub fn create_render_pass(instance: &Instance, device: &Device, data: &mut EngineData) -> Result<()> {}

pub fn create_descriptor_set_layout(device: &Device, data: &mut EngineData) -> Result<()> {}

pub fn create_pipeline(device: &Device, data: &mut EngineData) -> Result<()> {}

pub fn create_shader_module(device: &Device, bytecode: &[u8]) -> Result<vk::ShaderModule> {}

//====================
// Framebuffers
//====================

pub fn create_framebuffers(device: &Device, data: &mut EngineData) -> Result<()> {}

//====================
// Command Pool
//====================

pub fn create_command_pools(instance: &Instance, device: &Device, data: &mut EngineData) -> Result<()> {}

pub fn create_command_pool(instance: &Instance, device: &Device, data: &mut EngineData) -> Result<vk::CommandPool> {}

//====================
// Color Objects
//====================

pub fn create_color_objects(instance: &Instance, device: &Device, data: &mut EngineData) -> Result<()> {}

//====================
// Depth Objects
//====================

pub fn create_depth_objects(instance: &Instance, device: &Device, data: &mut EngineData) -> Result<()> {}

pub fn get_depth_format(instance: &Instance, data: &EngineData) -> Result<vk::Format> {}

pub fn get_supported_format(instance: &Instance, data: &EngineData, candidates: &[vk::Format], tiling: vk::ImageTiling, features: vk::FormatFeatureFlags) -> Result<vk::Format> {}

//====================
// Texture
//====================

pub fn create_texture_image(instance: &Instance, device: &Device, data: &mut EngineData) -> Result<()> {}

pub fn generate_mipmaps(instance: &Instance, device: &Device, data: &EngineData, image: vk::Image, format: vk::Format, width: u32, height: u32, mip_levels: u32) -> Result<()> {}

pub fn create_texture_image_view(device: &Device, data: &mut EngineData) -> Result<()> {}

pub fn create_texture_sampler(device: &Device, data: &mut EngineData) -> Result<()> {}

//====================
// Model
//====================

pub fn load_model(data: &mut EngineData) -> Result<()> {}

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

pub fn create_buffer(instance: &Instance, device: &Device, data: &EngineData, size: vk::DeviceSize, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags) -> Result<vk::Buffer, vk::DeviceMemory> {}

pub fn copy_buffer(device: &Device, data: &EngineData, source: vk::Buffer, destination: vk::Buffer, size: vk::DeviceSize) -> Result<()> {}

//====================
// Shared Images
//====================

pub fn create_image(instance: &Instance, device: &Device, data: &EngineData, width: u32, height: u32, mip_levels: u32, samples: vk::SampleCountFlags, format: vk::Format, tiling: vk::ImageTiling, usage: vk::ImageUsageFlags, properties: vk::MemoryPropertyFlags) -> Result<vk::Image, vk::DeviceMemory> {}

pub fn create_image_view(device: &Device, image: vk::Image, format: vk::Format, aspects: vk::ImageAspectFlags, mip_levels: u32) -> Result<vk::Image> {}

pub fn transition_image_layout(device: &Device, data: &EngineData, image: vk::Image, format: vk::Format, old_layout: vk::ImageLayout, new_layout: vk::ImageLayout, mip_levels: u32) -> Result<()> {}

pub fn copy_buffer_to_image(device: &Device, data: &EngineData, buffer: vk::Buffer, image: vk::Image, width: u32, height: u32) -> Result<()> {}

//====================
// Other Shared
//====================

pub fn get_memory_type_index(instance: &Instance, data: &EngineData, properties: vk::MemoryPropertyFlags, requirements: vk::MemoryRequirements) -> Result<u32> {}

pub fn begin_single_time_commands(device: &Device, data: &EngineData) -> Result<vk::CommandBuffer> {}

pub fn end_single_time_commands(device: &Device, data: &EngineData, command_buffer: vk::CommandBuffer) -> Result<()> {}