use bytemuck::{Pod, Zeroable};
use nalgebra::{Matrix4, Vector4, Vector3, Point3};



#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Camera {
    pub field_of_view: u32,
    pub render_distance: f32,
    pub aspect_ratio: f32,
    pub fov_tan: f32,
    pub camera_to_world_mat: Matrix4<f32>,
    pub clear_color: Vector4<f32>,
    pub position: Vector3<f32>,
}

impl Camera {
    fn create_camera_to_world_space(forward: Vector3<f32>, up: Vector3<f32>, position: Vector3<f32>) -> Matrix4<f32> {
        let mut camera_to_world = Matrix4::look_at_rh(&Point3::new(0.0, 0.0, 0.0), &Point3::new(forward.x, forward.y, forward.z), &up);
        camera_to_world[12] = position.x;
        camera_to_world[13] = position.y;
        camera_to_world[14] = position.z;
        camera_to_world
    }

    pub fn new(fov: u32, render_distance: f32, aspect_ratio: f32, target: Vector3<f32>, up_ref: Vector3<f32>, clear_color: Vector4<f32>, position: Vector3<f32>) -> Self {
        let forward: Vector3<f32> = target.normalize();
        let right: Vector3<f32> = forward.cross(&up_ref).normalize();
        let up: Vector3<f32> = right.cross(&forward).normalize();
        Camera { field_of_view: fov,
                        render_distance,
                        aspect_ratio, 
                        fov_tan: (fov as f32/2.0).to_radians().tan(),
                        camera_to_world_mat: Self::create_camera_to_world_space(forward, up, position),
                        clear_color,
                        position,
                    }
    }

    pub fn update_camera_dir(&mut self, target: Vector3<f32>, up_ref: Vector3<f32>) {
        let new_forward: Vector3<f32> = target.normalize();
        let new_right: Vector3<f32> = new_forward.cross(&up_ref).normalize();
        let new_up: Vector3<f32> = new_right.cross(&new_forward).normalize();
        self.camera_to_world_mat = Self::create_camera_to_world_space(new_forward, new_up, self.position);
    }

    pub fn camera_move(&mut self, input: Vector3<f32>, forward_vec: Vector3<f32>, delta_time: f32, movement_speed: f32) {
        let right_vec = forward_vec.cross(&Vector3::new(0.0, 1.0, 0.0)).normalize();
        if input.x > 0.0 {
            self.position.x -= forward_vec.x * delta_time * movement_speed;
            self.position.y += forward_vec.y * delta_time * movement_speed;
            self.position.z += forward_vec.z * delta_time * movement_speed;
        } else if input.x < 0.0 {
            self.position.x += forward_vec.x * delta_time * movement_speed;
            self.position.y -= forward_vec.y * delta_time * movement_speed;
            self.position.z -= forward_vec.z * delta_time * movement_speed;
        }
        if input.y > 0.0 {
            self.position.y += delta_time * movement_speed;
        } else if input.y < 0.0 {
            self.position.y -= delta_time * movement_speed;
        } 
        if input.z > 0.0 {
            self.position.x -= right_vec.x * delta_time * movement_speed;
            self.position.y += right_vec.y * delta_time * movement_speed;
            self.position.z += right_vec.z * delta_time * movement_speed;
        } else if input.z < 0.0 {
            self.position.x += right_vec.x * delta_time * movement_speed;
            self.position.y -= right_vec.y * delta_time * movement_speed;
            self.position.z -= right_vec.z * delta_time * movement_speed;
        }
        
    }
}