use std::ffi::CStr;

use ash::{Device, Entry, Instance, vk::{self, Window}};
use crate::EngineData;

const APP_NAME: &CStr = c"Testing";
const ENGINE_NAME: &CStr = c"M.A.V.";

pub fn test() {
    println!("Testing importation of functions.")
}

//====================
// Instance
//====================

pub fn create_instance(window: &Window, entry: &Entry, data: &mut EngineData) -> Result<Instance> {
    // Application Info
    let application_info = vk::ApplicationInfo::default()
        .application_name(APP_NAME)
        .application_version(0)
        .engine_name(ENGINE_NAME)
        .engine_version(0)
        .api_version(vk::make_api_version(0, 1, 0, 0));

    // Layers
    

    // Extensions


    // Create Instance


    // Debug Messenger
}

// Debug Callback

//====================
// Physical Device
//====================

pub fn pick_physical_device(instance: &Instance, data: &mut EngineData) -> Result<()> {}

pub fn check_physical_device(instance: &Instance, data: &EngineData, physical_device: vk::PhysicalDevice) -> Result<()> {}

pub fn check_physical_device_extensions(instance: &Instance, physical_device: vk::PhysicalDevice) -> Result<()> {}

pub fn get_msaa_samples(instance: &Instance, data: &EngineData) -> vk::SampleCountFlags {}

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