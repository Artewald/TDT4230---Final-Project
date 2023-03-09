use std::{
    f32::consts::PI,
    sync::{Arc, RwLock},
    time::Instant,
};

use artewald_engine_lib::voxel::VoxelData;
use nalgebra::{Vector3, Vector4};
use vulkano::{
    buffer::CpuAccessibleBuffer,
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferToImageInfo, CopyImageInfo,
        CopyImageToBufferInfo,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator,
        layout::{DescriptorSetLayout, DescriptorType},
        PersistentDescriptorSet, WriteDescriptorSet,
    },
    image::{ImageAccess, ImageViewAbstract},
    pipeline::{ComputePipeline, Pipeline, PipelineBindPoint},
    swapchain::{
        acquire_next_image, AcquireError, SwapchainCreateInfo, SwapchainCreationError,
        SwapchainPresentInfo,
    },
    sync::{self, FlushError, GpuFuture},
};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use crate::renderer::camera::Camera;

use self::{buffers::render_buffer, vulkan_data::VulkanData};

use buffers::{camera_data_buffer, voxel_buffer};
use shader::render_shader;

use super::WindowHandler;

mod buffers;
mod shader;
mod vulkan_data;

const PRINT_RENDER_INFO: bool = false;

pub struct WinitVulkanHandler {
    vulkan_data: Arc<RwLock<Option<VulkanData>>>,
}

impl WindowHandler for WinitVulkanHandler {
    fn run(&mut self, inital_voxels_to_present: Vec<VoxelData>) {
        let vulkan_data_reference = self.vulkan_data.clone();
        let event_loop = EventLoop::new();
        vulkan_data_reference
            .write()
            .unwrap()
            .replace(VulkanData::new_default(&event_loop));

        let main_shader = render_shader(
            vulkan_data_reference
                .read()
                .unwrap()
                .as_ref()
                .unwrap()
                .device
                .clone(),
        );

        let compute_pipline = ComputePipeline::new(
            vulkan_data_reference
                .read()
                .unwrap()
                .as_ref()
                .unwrap()
                .device
                .clone(),
            main_shader.entry_point("main").unwrap(),
            &(),
            None,
            |_| {},
        )
        .unwrap();

        let camera_data_buffer = camera_data_buffer(
            Camera::new(
                90,
                1000.0,
                (vulkan_data_reference
                    .read()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .window
                    .inner_size()
                    .width as f32)
                    / (vulkan_data_reference
                        .read()
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .window
                        .inner_size()
                        .height as f32),
                Vector3::new(0.0, 0.0, 1.0),
                Vector3::new(0.0, 1.0, 0.0),
                Vector4::new(0.0, 0.0, 0.1884, 1.0),
                Vector3::new(-2.0, 0.0, 0.0),
            ),
            vulkan_data_reference
                .read()
                .unwrap()
                .as_ref()
                .unwrap()
                .memory_allocator
                .clone(),
        );
        let voxel_buffer = voxel_buffer(
            inital_voxels_to_present,
            vulkan_data_reference
                .read()
                .unwrap()
                .as_ref()
                .unwrap()
                .memory_allocator
                .clone(),
        );
        let mut render_buffer_data = render_buffer(vulkan_data_reference.clone());

        let compute_pipeline_clone = compute_pipline.clone();
        let set_layouts = compute_pipeline_clone.layout().set_layouts();

        let mut sets = Self::descriptor_sets(
            vulkan_data_reference
                .read()
                .unwrap()
                .as_ref()
                .unwrap()
                .desc_allocator
                .clone(),
            set_layouts,
            voxel_buffer.clone(),
            camera_data_buffer.clone(),
            render_buffer_data.view.clone(),
        );

        // Main render loop
        let mut recreate_swapchain = false;
        let mut prev_frame_end = Some(
            sync::now(
                vulkan_data_reference
                    .read()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .device
                    .clone(),
            )
            .boxed(),
        );

        let mut time = Instant::now();
        let mut delta_time = Instant::now();

        let mut window_focused = true;
        let mut yaw: f32 = 0.0;
        let mut pitch: f32 = 0.2;
        let mut look_target: Vector3<f32> = Vector3::new(0.0, 0.0, -1.0);
        let mut movement_input: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);

        let mouse_sensitivity = 1.0 / (50.0 * 50.0);
        let movement_speed = 0.01;

        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::Focused(true),
                    ..
                } => {
                    window_focused = true;
                    vulkan_data_reference
                        .read()
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .window
                        .set_cursor_visible(false);
                }

                Event::WindowEvent {
                    event: WindowEvent::Focused(false),
                    ..
                } => {
                    window_focused = false;
                    vulkan_data_reference
                        .read()
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .window
                        .set_cursor_visible(true);
                }

                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { position, .. },
                    ..
                } => {
                    if window_focused {
                        let dim = vulkan_data_reference
                            .read()
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .window
                            .inner_size();
                        let center = PhysicalPosition::new(dim.width / 2, dim.height / 2);
                        let mouse_movement = PhysicalPosition::new(
                            position.x - center.x as f64,
                            center.y as f64 - position.y,
                        );

                        yaw -= (mouse_movement.x * mouse_sensitivity) as f32;
                        if yaw >= 2.0 * PI {
                            yaw = 0.0;
                        } else if yaw <= 0.0 {
                            yaw = 2.0 * PI;
                        }

                        pitch -= (mouse_movement.y * mouse_sensitivity) as f32;
                        if pitch >= 1.5 * PI {
                            pitch = 1.499999 * PI;
                        }
                        if pitch <= -PI / 2.0 {
                            pitch = -PI / 2.00001;
                        }
                        look_target = Vector3::new(
                            pitch.cos() * yaw.sin(),
                            -pitch.sin(),
                            pitch.cos() * yaw.cos(),
                        )
                        .normalize();

                        vulkan_data_reference
                            .read()
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .window
                            .set_cursor_position(center)
                            .unwrap();
                    }
                }

                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { input, .. },
                    ..
                } => {
                    if window_focused {
                        handle_inputs(input, control_flow, &mut movement_input);
                    }
                }

                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,

                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => recreate_swapchain = true,

                Event::RedrawEventsCleared => {
                    // Get the window dimentions
                    let dim = vulkan_data_reference
                        .read()
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .window
                        .inner_size();
                    if dim.width == 0 || dim.height == 0 {
                        return;
                    }

                    // This frees up some resources from time to time based on what the GPU has managed to do and not
                    prev_frame_end.as_mut().unwrap().cleanup_finished();

                    // Recreates the swapchain, decriptor-sets and the image that is rendered to.
                    if recreate_swapchain {
                        Self::recreate_swapchain(vulkan_data_reference.clone(), dim);
                        render_buffer_data = render_buffer(vulkan_data_reference.clone());
                        let compute_pipeline_cpy = compute_pipline.clone();
                        let new_set_layouts = compute_pipeline_cpy.layout().set_layouts();
                        camera_data_buffer.clone().write().unwrap().aspect_ratio =
                            (dim.width as f32) / (dim.height as f32);
                        sets = Self::descriptor_sets(
                            vulkan_data_reference
                                .read()
                                .unwrap()
                                .as_ref()
                                .unwrap()
                                .desc_allocator
                                .clone(),
                            new_set_layouts,
                            voxel_buffer.clone(),
                            camera_data_buffer.clone(),
                            render_buffer_data.view.clone(),
                        );
                        recreate_swapchain = false;
                    }

                    // Gets the current available swapchain image that the rendered image can be copied to.
                    // If the swapchain is suboptimal then it will be recreated later.
                    let (img_index, suboptimal, acquire_future) = match acquire_next_image(
                        vulkan_data_reference
                            .read()
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .swapchain
                            .clone(),
                        None,
                    ) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("Failed to get the next image: {}", e),
                    };

                    if suboptimal {
                        recreate_swapchain = true;
                    }

                    // Building the command buffer and executing it.
                    camera_data_buffer.clone().write().unwrap().camera_move(
                        movement_input,
                        look_target,
                        delta_time.elapsed().as_millis() as f32,
                        movement_speed,
                    );
                    delta_time = Instant::now();
                    camera_data_buffer
                        .clone()
                        .write()
                        .unwrap()
                        .update_camera_dir(look_target, Vector3::new(0.0, 1.0, 0.0));

                    let mut builder = AutoCommandBufferBuilder::primary(
                        vulkan_data_reference
                            .read()
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .cmd_allocator
                            .clone()
                            .as_ref(),
                        vulkan_data_reference
                            .read()
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .queue
                            .queue_family_index(),
                        CommandBufferUsage::OneTimeSubmit,
                    )
                    .unwrap();
                    builder
                        .bind_pipeline_compute(compute_pipline.clone())
                        .bind_descriptor_sets(
                            PipelineBindPoint::Compute,
                            compute_pipline.clone().layout().clone(),
                            0,
                            sets.clone(),
                        )
                        .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
                            render_buffer_data.buffer.clone(),
                            render_buffer_data.image.clone(),
                        ))
                        .unwrap()
                        .dispatch([
                            vulkan_data_reference
                                .read()
                                .unwrap()
                                .as_ref()
                                .unwrap()
                                .images[0]
                                .dimensions()
                                .width()
                                / 8,
                            vulkan_data_reference
                                .read()
                                .unwrap()
                                .as_ref()
                                .unwrap()
                                .images[0]
                                .dimensions()
                                .height()
                                / 8,
                            1,
                        ])
                        .unwrap()
                        .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
                            render_buffer_data.image.clone(),
                            render_buffer_data.buffer.clone(),
                        ))
                        .unwrap()
                        .copy_image(CopyImageInfo::images(
                            render_buffer_data.image.clone(),
                            vulkan_data_reference
                                .read()
                                .unwrap()
                                .as_ref()
                                .unwrap()
                                .images[img_index as usize]
                                .clone(),
                        ))
                        .unwrap();

                    let command_buffer = builder.build().unwrap();
                    let future = sync::now(
                        vulkan_data_reference
                            .read()
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .device
                            .clone(),
                    )
                    .join(acquire_future)
                    .then_execute(
                        vulkan_data_reference
                            .read()
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .queue
                            .clone(),
                        command_buffer,
                    )
                    .unwrap()
                    .then_swapchain_present(
                        vulkan_data_reference
                            .read()
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .queue
                            .clone(),
                        SwapchainPresentInfo::swapchain_image_index(
                            vulkan_data_reference
                                .read()
                                .unwrap()
                                .as_ref()
                                .unwrap()
                                .swapchain
                                .clone(),
                            img_index,
                        ),
                    )
                    .then_signal_fence_and_flush();

                    match future {
                        Ok(future) => {
                            // Wait for the GPU to finish and then proceed
                            future.wait(None).unwrap();
                            prev_frame_end = Some(future.boxed())
                        }
                        Err(FlushError::OutOfDate) => {
                            recreate_swapchain = true;
                            prev_frame_end = Some(
                                sync::now(
                                    vulkan_data_reference
                                        .read()
                                        .unwrap()
                                        .as_ref()
                                        .unwrap()
                                        .device
                                        .clone(),
                                )
                                .boxed(),
                            );
                        }
                        Err(e) => panic!("Failed to flush future: {}", e),
                    }

                    if PRINT_RENDER_INFO {
                        println!("WINIT_VULKAN_HANDLER: PRINT_RENDER_INFO->Find a better way!");
                        println!("Render time: {}", time.elapsed().as_millis());
                        println!("Window in focus: {}", window_focused);
                        print!("\x1B[2J\x1B[1;1H");
                        time = Instant::now();
                    }

                    // Cleanup
                    // movement_input.x = 0.0;
                    // movement_input.y = 0.0;
                    // movement_input.z = 0.0;
                }

                _ => (),
            }
        });
    }
}

fn handle_inputs(
    input: winit::event::KeyboardInput,
    control_flow: &mut ControlFlow,
    movement_input: &mut nalgebra::Matrix<
        f32,
        nalgebra::Const<3>,
        nalgebra::Const<1>,
        nalgebra::ArrayStorage<f32, 3, 1>,
    >,
) {
    if let Some(btn) = input.virtual_keycode {
        let state = input.state;

        if btn == VirtualKeyCode::Escape {
            *control_flow = ControlFlow::Exit;
        }

        if state == ElementState::Pressed {
            if btn == VirtualKeyCode::W {
                movement_input.x = 1.0
            } else if btn == VirtualKeyCode::S {
                movement_input.x = -1.0
            } else if btn == VirtualKeyCode::Space {
                movement_input.y = 1.0
            } else if btn == VirtualKeyCode::LControl {
                movement_input.y = -1.0
            } else if btn == VirtualKeyCode::D {
                movement_input.z = 1.0
            } else if btn == VirtualKeyCode::A {
                movement_input.z = -1.0
            }
        } else if btn == VirtualKeyCode::W || btn == VirtualKeyCode::S {
            movement_input.x = 0.0
        } else if btn == VirtualKeyCode::Space || btn == VirtualKeyCode::LControl {
            movement_input.y = 0.0
        } else if btn == VirtualKeyCode::D || btn == VirtualKeyCode::A {
            movement_input.z = 0.0
        }
    }
}

impl WinitVulkanHandler {
    pub fn init() -> Self {
        WinitVulkanHandler {
            vulkan_data: Arc::new(RwLock::new(None)),
        }
    }

    fn recreate_swapchain(vulkan_data_rw: Arc<RwLock<Option<VulkanData>>>, dim: PhysicalSize<u32>) {
        let mut binding = vulkan_data_rw.write().unwrap();
        let mut vulkan_data = binding.as_mut().unwrap();

        let (new_sc, new_imgs) = match vulkan_data.swapchain.recreate(SwapchainCreateInfo {
            image_extent: dim.into(),
            ..vulkan_data.swapchain.create_info()
        }) {
            Ok(r) => r,
            Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
            Err(e) => panic!("Failed to recreate the swapchain, error: {}", e),
        };

        vulkan_data.swapchain = new_sc;
        vulkan_data.images = new_imgs;
    }

    fn descriptor_sets(
        desc_allocator: Arc<StandardDescriptorSetAllocator>,
        set_layouts: &[Arc<DescriptorSetLayout>],
        voxel_buffer: Arc<CpuAccessibleBuffer<[VoxelData]>>,
        misc_buffer: Arc<CpuAccessibleBuffer<Camera>>,
        img_view: Arc<dyn ImageViewAbstract>,
    ) -> Vec<Arc<PersistentDescriptorSet>> {
        let mut sets = vec![];

        for set_layout in set_layouts {
            let mut visited: Vec<DescriptorType> = vec![];
            for x in set_layout.bindings() {
                //println!("{:?}", x.1.descriptor_type);
                if visited.contains(&x.1.descriptor_type) {
                    continue;
                }

                if x.1.descriptor_type == DescriptorType::StorageBuffer {
                    sets.push(
                        PersistentDescriptorSet::new(
                            desc_allocator.clone().as_ref(),
                            set_layout.clone(),
                            [
                                WriteDescriptorSet::buffer(0, voxel_buffer.clone()),
                                WriteDescriptorSet::buffer(1, misc_buffer.clone()),
                            ],
                        )
                        .unwrap(),
                    );
                } else if x.1.descriptor_type == DescriptorType::StorageImage {
                    sets.push(
                        PersistentDescriptorSet::new(
                            desc_allocator.clone().as_ref(),
                            set_layout.clone(),
                            [WriteDescriptorSet::image_view(0, img_view.clone())],
                        )
                        .unwrap(),
                    )
                } else {
                    panic!("There exists an unused descriptorset, it should be implemented!");
                }

                visited.push(x.1.descriptor_type);
            }
        }

        sets
    }
}
