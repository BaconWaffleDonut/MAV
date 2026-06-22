use std::{borrow::Cow, collections::{HashMap, HashSet}, ffi::{self, CStr}, ops::Index};
use core::{ffi::c_char};
use ash::{
    Device, Entry, Instance, ext::debug_utils, khr::{surface, swapchain}, vk::{self, PhysicalDevice, SurfaceKHR} 
};
use winit::{
    event_loop::EventLoop,
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
    };
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

/* fn utils(entry: &Entry, instance: &Instance, window: &dyn Window) -> Result<()> {
    let event_loop = EventLoop::new()?;
    let surface_loader = surface::Instance::new(&entry, &instance);
    let surface = unsafe{ash_window::create_surface(&entry, &instance, event_loop.display_handle()?.as_raw(), window.window_handle()?.as_raw(), None)}.expect("Failed to create surface.");

    Ok(())
} */

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

pub unsafe fn create_instance(data: &mut EngineData) -> Result<Instance> {
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

pub fn create_logical_device(entry: &Entry, instance: &Instance, data: &mut EngineData, window: &dyn Window, physical_device: vk::PhysicalDevice) -> Result<Device> {
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