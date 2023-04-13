use artewald_engine_lib::voxel;
use artewald_engine_lib::{threadpool::ThreadPoolHelper, voxel::Material};
use nalgebra::{Vector2, Vector3, Vector4};
use renderer::setup_renderer_and_run;
use voxel::Chunk;

mod renderer;

// Optimization can be done by using flamegraph and cargo-asm
fn main() {
    //let time = Instant::now();
    let thread_pool = ThreadPoolHelper::new(Some(0));
    let mut chunk = Chunk::new(Vector2::new(0, 0), 16);

    let mut green_material = Material::new_default();
    green_material.color = Vector4::new(0.0, 1.0, 0.0, 1.0);
    green_material.smoothness = 1.0;
    green_material.specular_color = Vector4::new(1.0, 1.0, 1.0, 1.0);
    green_material.specular_probability = 0.15;
    let mut red_material = Material::new_default();
    red_material.color = Vector4::new(1.0, 0.0, 0.0, 1.0);
    let mut light_material = Material::new_default();
    light_material.emissive_color = Vector3::new(1.0, 1.0, 1.0);
    light_material.emissive_strength = 1.0;

    chunk.fill_voxels(
        thread_pool.clone(),
        Vector3::new(Vector2::new(5, 10), Vector2::new(0, 5), Vector2::new(3, 15)),
        red_material,
    );
    chunk.fill_voxels(
        thread_pool.clone(),
        Vector3::new(Vector2::new(0, 3), Vector2::new(2, 5), Vector2::new(5, 10)),
        green_material,
    );
    chunk.fill_voxels(
        thread_pool.clone(),
        Vector3::new(
            Vector2::new(0, 15),
            Vector2::new(10, 11),
            Vector2::new(0, 20),
        ),
        light_material,
    );
    let (voxel_data, materials) = chunk.get_oct_tree(
        Vector3::new(0.0, 0.0, 0.0),
        (90.0_f32 / 1080.0_f32).to_radians(),
    );
    if voxel_data.len() > (u32::MAX - 2) as usize {
        panic!("There are more than u32-2 indices in the voxel array for the gpu, that's too much for the GPU");
    }
    // println!("{:?}", voxel_data);
    setup_renderer_and_run(voxel_data, materials);
}
