pub fn translate(x: f32, y: f32, z: f32) -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        x,   y,   z,   1.0
    ]
}

pub fn scale(x: f32, y: f32, z: f32) -> [f32; 16] {
    [
        x,  0.0, 0.0, 0.0,
        0.0, y,  0.0, 0.0,
        0.0, 0.0, z,  0.0,
        0.0, 0.0, 0.0, 1.0 
    ]
}

pub fn rotate_x(angle: f32) -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0,
        0.0, angle.cos(), angle.sin(), 0.0,
        0.0, -(angle.sin()), angle.cos(), 0.0,
        0.0, 0.0, 0.0, 1.0
    ]
}

pub fn rotate_y(angle: f32) -> [f32; 16] {
    [
        angle.cos(), 0.0, -(angle.sin()), 0.0,
        0.0, 1.0, 0.0, 0.0,
        angle.sin(), 0.0, angle.cos(), 0.0,
        0.0, 0.0, 0.0, 1.0
    ]
}

pub fn matrix_mult(a: [f32; 16], b: [f32; 16]) -> [f32; 16] {
    let mut result = [0.0; 16];

    for i in 0..4 {
        for j in 0..4 {
            result[i * 4 + j] = b[i*4 + 0]*a[j + 0] +
                                b[i*4 + 1]*a[j + 4] +
                                b[i*4 + 2]*a[j + 8] +
                                b[i*4 + 3]*a[j + 12];
        }
    }
    result
}

pub fn perspective(fov_angle: f32, aspect_ratio: f32, far: f32, near: f32) -> [f32; 16] {
    let tan_fov = (fov_angle / 2.0).tan();

    let m00 = 1.0 / (aspect_ratio * tan_fov);
    let m11 = 1.0 / tan_fov;
    let m22 = far / (far - near);
    let m23 = far * near / (near - far);

    [
        m00, 0.0, 0.0, 0.0,
        0.0, m11, 0.0, 0.0,
        0.0, 0.0, m22, 1.0,
        0.0, 0.0, m23, 0.0
    ]
}

