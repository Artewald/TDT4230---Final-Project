use crate::voxel::VoxelData;

use self::window_handler::{winit_vulkan_handler::WinitVulkanHandler, WindowHandler};

mod window_handler;
mod camera;

/// #NOTE
/// This will most likely hijack the thread. Also, this should be ran from the main thread
pub fn setup_renderer_and_run(voxel_data: Vec<VoxelData>) {
    let window_handler: &mut dyn WindowHandler = &mut WinitVulkanHandler::init() as &mut dyn WindowHandler;
    window_handler.run(voxel_data);
}