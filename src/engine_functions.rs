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

pub fn test() {
    println!("Testing importation of functions.")
}