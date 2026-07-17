/* use cgmath::Point3;

#[derive(Clone, Copy)]
pub struct Camera {
    theta: f32,
    phi: f32,
    rho: f32,
    cam_x: f32,
    cam_y: f32,
    cam_z: f32,
}

fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    let value = if value > max { max } else { value };
    if value < min { min } else { value }
}

impl Camera {
    pub fn position(&self) -> Point3<f32> {
        Point3::new(
            self.rho * self.phi.sin() * self.theta.sin(),
            self.rho * self.phi.cos(),
            self.rho * self.phi.sin() * self.theta.cos(),
        )
    }

    pub fn rotate(&mut self, theta: f32, phi: f32) {
        self.theta += theta;
        let phi = self.phi + phi;
        self.phi = clamp(phi, 10.0_f32.to_radians(), 170.0_f32.to_radians());
    }

    pub fn foward(&mut self, rho: f32) {
        self.rho -= rho;
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            theta: 0.0_f32.to_radians(),
            phi: 45.0_f32.to_radians(),
            rho: 3.0,
            cam_x: 0.0,
            cam_y: 0.0,
            cam_z: 0.0,
        }
    }
} */

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
            rot_x: 0.0, 
            rot_y: 0.0, 
            rot_z: 0.0 }
    }
}