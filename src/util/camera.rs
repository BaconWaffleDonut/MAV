use std::f32::consts::PI;

use cgmath::{Vector2, vec2};

#[derive(Clone, Copy)]
pub struct Camera {
    pub(crate) pos_x: f32,
    pub(crate) pos_y: f32,
    pub(crate) pos_z: f32,
    pub(crate) rot_x: f32,
    pub(crate) rot_y: f32,
    pub(crate) rot_z: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Camera { 
            pos_x: 0.0, 
            pos_y: 0.0, 
            pos_z: 0.0, 
            rot_x: PI / 2.0, 
            rot_y: 0.0, 
            rot_z: 0.0 }
    }
}