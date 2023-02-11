use std::sync::{Arc, RwLock};

use artewald_engine_lib::voxel::VoxelData;
use vulkano::{memory::allocator::{FreeListAllocator, GenericMemoryAllocator}, buffer::{CpuAccessibleBuffer, BufferUsage}, image::{StorageImage, ImageDimensions, ImageAccess, view::ImageView}, format::Format};

use crate::renderer::camera::Camera;

use super::vulkan_data::VulkanData;


pub struct RenderBufferData {
    pub image: Arc<StorageImage>,
    pub buffer: Arc<CpuAccessibleBuffer<[u8]>>,
    pub view: Arc<ImageView<StorageImage>>,
}

pub fn camera_data_buffer(data: Camera, allocator: Arc<GenericMemoryAllocator<Arc<FreeListAllocator>>>) -> Arc<CpuAccessibleBuffer<Camera>> {
    CpuAccessibleBuffer::from_data(allocator.as_ref(), BufferUsage {storage_buffer: true, ..BufferUsage::empty()}, false, data).unwrap()
}

pub fn voxel_buffer(data: Vec<VoxelData>, allocator: Arc<GenericMemoryAllocator<Arc<FreeListAllocator>>>) -> Arc<CpuAccessibleBuffer<[VoxelData]>> {
    CpuAccessibleBuffer::from_iter(allocator.as_ref(), BufferUsage {storage_buffer: true, ..BufferUsage::empty()}, false, data.into_iter()).unwrap()
}

pub fn render_buffer(vulkan_data_rw: Arc<RwLock<Option<VulkanData>>>) -> RenderBufferData {
    let binding = vulkan_data_rw.read().unwrap();
    let vulkan_data = binding.as_ref().unwrap();

    let image = StorageImage::new(
        vulkan_data.memory_allocator.clone().as_ref(),
        ImageDimensions::Dim2d {
            width: vulkan_data.images[0].dimensions().width(),
            height: vulkan_data.images[0].dimensions().height(),
            array_layers: 1,
        }, 
        Format::R8G8B8A8_UNORM,
        Some(vulkan_data.queue.clone().queue_family_index()),
    ).unwrap();

    let buffer = CpuAccessibleBuffer::from_iter(
        vulkan_data.memory_allocator.clone().as_ref(),
        BufferUsage {
            transfer_src: true,
            transfer_dst: true,
            storage_buffer: true,
            ..Default::default()
        },
        false,
        (0..vulkan_data.images[0].dimensions().width() * vulkan_data.images[0].dimensions().height() * 4).map(|_| 0u8)
    ).unwrap();

    let view = ImageView::new_default(image.clone()).unwrap();

    RenderBufferData { image, buffer, view }
}