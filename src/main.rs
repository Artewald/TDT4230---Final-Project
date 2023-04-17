use artewald_engine_lib::voxel::{self};
use artewald_engine_lib::threadpool::ThreadPoolHelper;
use nalgebra::Vector2;
use renderer::setup_renderer_and_run;
use scene_creator::{create_simple_scene, create_complex_scene};
use voxel::Chunk;

use crate::scene_creator::{create_mirror_scene, create_neon_mirror_maze_scene};

mod renderer;
mod scene_creator;

// Optimization can be done by using flamegraph and cargo-asm
fn main() {
    let thread_pool = ThreadPoolHelper::new(Some(0));
    let mut chunk = Chunk::new(Vector2::new(0, 0), 16);

    let mut recreate = false;

    let args = std::env::args().collect::<Vec<String>>();
    if args.contains(&String::from("--recreate")) {
        recreate = true;
    }

    let mut voxel_data = Vec::new();
    let mut materials = Vec::new();

    // Just so that it's clear creating the scene does not necessarily mean that the chunk is filled with voxels, it's just a way to get the data.
    // If the scene is already stored in a file, it will be loaded from there. Unless the --recreate flag is set.
    for arg in args {
        match arg.as_str() {
            "--simple" => {
                (voxel_data, materials) = create_simple_scene(&mut chunk, thread_pool.clone(), recreate);
                break;
            },
            "--complex" => {
                (voxel_data, materials) = create_complex_scene(&mut chunk, thread_pool.clone(), recreate);
                break;
            },
            "--mirror" => {
                (voxel_data, materials) = create_mirror_scene(&mut chunk, thread_pool.clone(), recreate);
                break;
            },
            "--neon" => {
                (voxel_data, materials) = create_neon_mirror_maze_scene(&mut chunk, thread_pool.clone(), recreate);
                break;
            },
            _ => {}
        }
    }

    if voxel_data.is_empty() {
        (voxel_data, materials) = create_simple_scene(&mut chunk, thread_pool, recreate);
    }

    if voxel_data.len() > (u32::MAX - 2) as usize {
        panic!("There are more than u32-2 indices in the voxel array for the gpu, that's too much for the GPU");
    }

    setup_renderer_and_run(voxel_data, materials);
}
