use std::sync::{Arc, RwLock};

use artewald_engine_lib::voxel;
use artewald_engine_lib::{threadpool::ThreadPoolHelper, voxel::Material};
use nalgebra::{Vector2, Vector3, Vector4};
use renderer::setup_renderer_and_run;
use scene_creator::{create_simple_scene, create_complex_scene};
use voxel::Chunk;

mod renderer;
mod scene_creator;

// Optimization can be done by using flamegraph and cargo-asm
fn main() {
    //let time = Instant::now();
    let thread_pool = ThreadPoolHelper::new(Some(0));
    let mut chunk = Chunk::new(Vector2::new(0, 0), 16);
    // create_simple_scene(&mut chunk, thread_pool);
    create_complex_scene(&mut chunk, thread_pool);

    let (voxel_data, materials) = chunk.get_oct_tree(
        Vector3::new(0.0, 0.0, 0.0),
        (90.0_f32 / 1080.0_f32).to_radians(),
    );
    if voxel_data.len() > (u32::MAX - 2) as usize {
        panic!("There are more than u32-2 indices in the voxel array for the gpu, that's too much for the GPU");
    }
    // println!("{:?}", voxel_data);
    // for voxel in voxel_data.iter() {
    //     if voxel.material_index == 4294967295 {
    //         continue;
    //     }
    //     println!("{:?}", voxel);
    // }
    setup_renderer_and_run(voxel_data, materials);
}