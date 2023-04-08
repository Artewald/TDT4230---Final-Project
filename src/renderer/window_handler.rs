use artewald_engine_lib::voxel::{Material, VoxelData};

pub mod winit_vulkan_handler;

pub trait WindowHandler {
    /// Starts and runs the window and rendering
    /// # NOTE
    /// This function must run on the main thread and will most likely hijack it!
    fn run(&mut self, inital_voxels_to_present: Vec<VoxelData>, voxel_materials: Vec<Material>);
}
