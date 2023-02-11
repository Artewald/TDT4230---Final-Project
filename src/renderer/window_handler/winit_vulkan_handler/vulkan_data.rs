use std::sync::Arc;

use vulkano::{swapchain::{Surface, Swapchain, SwapchainCreateInfo, PresentMode}, instance::{Instance, InstanceCreateInfo}, device::{Device, Queue, DeviceCreateInfo, QueueCreateInfo, DeviceExtensions, physical::{PhysicalDeviceType, PhysicalDevice}}, memory::allocator::{GenericMemoryAllocator, FreeListAllocator, GenericMemoryAllocatorCreateInfo, Threshold, BlockSize, AllocationType}, descriptor_set::allocator::StandardDescriptorSetAllocator, command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo}, image::{SwapchainImage, ImageUsage}, VulkanLibrary};
use vulkano_win::create_surface_from_winit;
use winit::{window::{Window, WindowBuilder}, event_loop::EventLoop};



pub struct VulkanData {
    pub surface: Arc<Surface>,
    pub window: Arc<Window>,
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub memory_allocator: Arc<GenericMemoryAllocator<Arc<FreeListAllocator>>>,
    pub desc_allocator: Arc<StandardDescriptorSetAllocator>,
    pub cmd_allocator: Arc<StandardCommandBufferAllocator>,
    pub queue: Arc<Queue>,
    pub swapchain: Arc<Swapchain>,
    pub images: Vec<Arc<SwapchainImage>>,
}

impl VulkanData {
    pub fn new_default(event_loop: &EventLoop<()>) -> Self {
        let instance = Self::create_vulkan_instance();
    
        let window = Arc::new(WindowBuilder::new().with_title("Octree voxel raytracer").build(&event_loop).unwrap());
        let surface = create_surface_from_winit(window.clone(), instance.clone()).unwrap();
    
        let (device, queue) = Self::device_and_queue(instance.clone(), surface.clone());
    
        let (swapchain, images) = Self::swapchain_and_swapchain_images(device.clone(), surface.clone(), window.clone());
    
        VulkanData {surface: surface, window: window, instance: instance.clone(), device: device.clone(), memory_allocator: Self::memory_allocator(device.clone()), desc_allocator: Arc::new(StandardDescriptorSetAllocator::new(device.clone())), cmd_allocator: Arc::new(StandardCommandBufferAllocator::new(device.clone(), StandardCommandBufferAllocatorCreateInfo {..Default::default()})), queue: queue.clone(), swapchain: swapchain, images: images }
    }

    fn create_vulkan_instance() -> Arc<Instance> {
        let library = VulkanLibrary::new().unwrap();
        let required_extension = vulkano_win::required_extensions(&library);
        
        Instance::new(library, InstanceCreateInfo {
            enabled_extensions: required_extension, 
            enumerate_portability: true,
            ..Default::default()
        }).expect("failed to create instance")
    }

    fn device_and_queue(instance: Arc<Instance>, surface: Arc<Surface>) -> (Arc<Device>, Arc<Queue>) {
        let device_extensions = Self::device_extensions();
        let (physical_device, queue_family_index) = Self::physical_device_and_queue_family_index(instance.clone(), &device_extensions, &surface);
    
        println!("VULKANDATA(device_and_queues): Renderer using {}", physical_device.clone().properties().device_name);

        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {queue_family_index: queue_family_index as u32, ..Default::default()}],
                ..Default::default()
            },
        ).expect("failed to create device");

        let queue = queues.next().unwrap();

        (device, queue)
    }

    fn device_extensions() -> DeviceExtensions {
        DeviceExtensions {
            khr_swapchain: true,
            khr_storage_buffer_storage_class: true,
            khr_swapchain_mutable_format: true,
            ..DeviceExtensions::empty()
        }
    }

    fn physical_device_and_queue_family_index(instance: Arc<Instance>, device_extensions: &DeviceExtensions, surface: &Surface) -> (Arc<PhysicalDevice>, u32) {
        instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|p| {
                p.supported_extensions().contains(&device_extensions)
            }).filter_map(|p| {
                p.queue_family_properties().iter().enumerate().position(|(i, q)| {
                        q.queue_flags.compute && p.surface_support(i as u32, &surface).unwrap_or(false)
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| {
                match p.properties().device_type {
                    PhysicalDeviceType::DiscreteGpu => 0,
                    PhysicalDeviceType::IntegratedGpu => 1,
                    PhysicalDeviceType::VirtualGpu => 2,
                    PhysicalDeviceType::Cpu => 3,
                    PhysicalDeviceType::Other => 4,
                    _ => 5,
                }
            })
            .expect("No suitable physical device found")
    }

    fn swapchain_and_swapchain_images(device: Arc<Device>, surface: Arc<Surface>, window: Arc<Window>) -> (Arc<Swapchain>, Vec<Arc<SwapchainImage>>) {
        let surface_capabilities = device.physical_device().surface_capabilities(&surface, Default::default()).unwrap();
        let image_format = Some(device.physical_device().surface_formats(&surface, Default::default()).unwrap()[0].0);
    
        Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: surface_capabilities.min_image_count,
                image_format,
                image_extent: window.inner_size().into(),
                image_usage: ImageUsage {
                    storage: true,
                    color_attachment: true,
                    transfer_dst: true,
                    ..Default::default()
                },
                composite_alpha: surface_capabilities.supported_composite_alpha.iter().next().unwrap(),
                present_mode: PresentMode::Fifo,
                ..Default::default()
            }
        ).unwrap()
    }

    fn memory_allocator(device: Arc<Device>) -> Arc<GenericMemoryAllocator<Arc<FreeListAllocator>>> {
        Arc::new(GenericMemoryAllocator::new(
            device.clone(), 
            GenericMemoryAllocatorCreateInfo {block_sizes: &[(0 as Threshold, 199_999_999 as BlockSize)], 
            allocation_type: AllocationType::Unknown, ..Default::default()}
        ).unwrap())
    }

}