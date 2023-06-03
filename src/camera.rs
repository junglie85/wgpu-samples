use bytemuck_derive::{Pod, Zeroable};
use glam::{Mat4, Vec3};

pub struct CameraDescriptor {
    pub aspect_ratio: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
    pub position: Vec3,
    pub direction: Vec3,
    pub up: Vec3,
    pub speed: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub mouse_sensitivity: f32,
}

impl Default for CameraDescriptor {
    fn default() -> Self {
        Self {
            aspect_ratio: 16.0 / 9.0,
            fov_y: 45.0,
            z_near: 0.1,
            z_far: 100.0,
            position: Vec3::new(0.0, 0.0, 3.0),
            direction: Vec3::new(0.0, 0.0, -1.0),
            up: Vec3::Y,
            speed: 10.0,
            yaw: -90.0,
            pitch: 0.0,
            mouse_sensitivity: 0.1,
        }
    }
}

pub struct Camera {
    aspect_ratio: f32,
    fov_y: f32,
    z_near: f32,
    z_far: f32,
    position: Vec3,
    direction: Vec3,
    up: Vec3,
    speed: f32,
    yaw: f32,
    pitch: f32,
    mouse_sensitivity: f32,
}

impl Camera {
    pub fn new(desc: &CameraDescriptor) -> Self {
        Self {
            aspect_ratio: desc.aspect_ratio,
            fov_y: desc.fov_y,
            z_near: desc.z_near,
            z_far: desc.z_far,
            position: desc.position,
            direction: desc.direction,
            up: desc.up,
            speed: desc.speed,
            yaw: desc.yaw,
            pitch: desc.pitch,
            mouse_sensitivity: desc.mouse_sensitivity,
        }
    }

    pub fn get_position(&self) -> Vec3 {
        self.position
    }

    pub fn get_direction(&self) -> Vec3 {
        self.direction
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::look_to_rh(self.position, self.direction, self.up)
    }

    pub fn get_projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(
            self.fov_y.to_radians(),
            self.aspect_ratio,
            self.z_near,
            self.z_far,
        )
    }

    pub fn get_gpu_camera(&self) -> GpuCamera {
        GpuCamera {
            projection: self.get_projection_matrix().to_cols_array(),
            view: self.get_view_matrix().to_cols_array(),
            position: self.position.to_array(),
            _pad: 0.0,
        }
    }

    pub fn move_forward(&mut self, dt: f32) {
        self.position += self.speed * self.direction * dt;
    }

    pub fn move_backward(&mut self, dt: f32) {
        self.position -= self.speed * self.direction * dt;
    }

    pub fn skew_left(&mut self, dt: f32) {
        self.position -= self.direction.cross(self.up).normalize() * self.speed * dt;
    }

    pub fn skew_right(&mut self, dt: f32) {
        self.position += self.direction.cross(self.up).normalize() * self.speed * dt;
    }

    pub fn yaw_pitch(&mut self, yaw: f32, pitch: f32) {
        self.yaw += yaw * self.mouse_sensitivity;
        self.pitch += pitch * self.mouse_sensitivity;

        if self.pitch > 89.0 {
            self.pitch = 89.0;
        }
        if self.pitch < -89.0 {
            self.pitch = -89.0;
        }

        let direction = Vec3::new(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        );
        self.direction = direction.normalize();
    }

    pub fn zoom(&mut self, delta: f32) {
        self.fov_y -= delta;

        if self.fov_y < 1.0 {
            self.fov_y = 1.0;
        }
        if self.fov_y > 45.0 {
            self.fov_y = 45.0;
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct GpuCamera {
    projection: [f32; 16],
    view: [f32; 16],
    position: [f32; 3],
    _pad: f32,
}
