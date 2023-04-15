use std::fs::{File, self};
use std::io::{Write, Read};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use artewald_engine_lib::voxel::{self, VoxelData};
use artewald_engine_lib::{threadpool::ThreadPoolHelper, voxel::Material};
use nalgebra::{Vector2, Vector3, Vector4};
use renderer::setup_renderer_and_run;
use scene_creator::{create_simple_scene, create_complex_scene};
use voxel::Chunk;

mod renderer;
mod scene_creator;

// Optimization can be done by using flamegraph and cargo-asm
fn main() {
    let thread_pool = ThreadPoolHelper::new(Some(0));
    let mut chunk = Chunk::new(Vector2::new(0, 0), 16);
    let mut current_time = Instant::now();

    // Just so that it's clear creating the scene does not necessarily mean that the chunk is filled with voxels, it's just a way to get the data.
    // If the scene is already stored in a file, it will be loaded from there.
    // let (mut voxel_data, materials) = create_simple_scene(&mut chunk, thread_pool);
    let (mut voxel_data, materials) = create_complex_scene(&mut chunk, thread_pool);
    println!(
        "Time to create scene: {}ms",
        current_time.elapsed().as_millis()
    );

    // let (mut voxel_data, materials) = chunk.get_oct_tree(
    //     Vector3::new(0.0, 0.0, 0.0),
    //     (90.0_f32 / 1080.0_f32).to_radians(),
    // );
    if voxel_data.len() > (u32::MAX - 2) as usize {
        panic!("There are more than u32-2 indices in the voxel array for the gpu, that's too much for the GPU");
    }

    setup_renderer_and_run(voxel_data, materials);
}
