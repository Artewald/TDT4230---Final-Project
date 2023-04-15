use std::io::{Write, Read};

use artewald_engine_lib::voxel::{Material, VoxelData};

const SCENE_FOLDER: &str = "scenes";

pub fn does_scene_exist(scene_name: &str) -> bool {
    std::path::Path::new(format!("{}/{}_voxel_data.bin", SCENE_FOLDER, scene_name).as_str()).exists()
}

pub fn save_voxel_data_to_file(voxel_data: Vec<VoxelData>, materials: Vec<Material>, scene_name: &str) {
    let voxel_bytes = voxels_to_bytes(&voxel_data);
    let mut file = std::fs::File::create(format!("{}/{}_voxel_data.bin", SCENE_FOLDER, scene_name)).unwrap();
    file.write_all(&voxel_bytes).unwrap();

    let material_bytes = materials_to_bytes(&materials);
    let mut file = std::fs::File::create(format!("{}/{}_material_data.bin", SCENE_FOLDER, scene_name)).unwrap();
    file.write_all(&material_bytes).unwrap();
}

pub fn load_voxel_data_from_file(scene_name: &str) -> (Vec<VoxelData>, Vec<Material>) {
    let mut file = std::fs::File::open(format!("{}/{}_voxel_data.bin", SCENE_FOLDER, scene_name)).unwrap();
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).unwrap();
    let voxel_data = bytes_to_voxels(&bytes);

    let mut file = std::fs::File::open(format!("{}/{}_material_data.bin", SCENE_FOLDER, scene_name)).unwrap();
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).unwrap();
    let materials = bytes_to_materials(&bytes);

    (voxel_data, materials)
}

fn materials_to_bytes(materials: &[Material]) -> Vec<u8> {
    let size = materials.len() * std::mem::size_of::<Material>();
    let mut bytes = Vec::with_capacity(size);
    unsafe {
        let ptr = bytes.as_mut_ptr() as *mut Material;
        ptr.copy_from_nonoverlapping(materials.as_ptr(), materials.len());
        bytes.set_len(size);
    }
    bytes
}

fn bytes_to_materials(bytes: &[u8]) -> Vec<Material> {
    let material_size = std::mem::size_of::<Material>();
    let len = bytes.len() / material_size;
    let mut materials = Vec::with_capacity(len);
    unsafe {
        let ptr = bytes.as_ptr() as *const Material;
        for i in 0..len {
            materials.push(*ptr.add(i));
        }
    }
    materials
}

fn voxels_to_bytes(voxels: &[VoxelData]) -> Vec<u8> {
    let size = voxels.len() * std::mem::size_of::<VoxelData>();
    let mut bytes = Vec::with_capacity(size);
    unsafe {
        let ptr = bytes.as_mut_ptr() as *mut VoxelData;
        ptr.copy_from_nonoverlapping(voxels.as_ptr(), voxels.len());
        bytes.set_len(size);
    }
    bytes
}

fn bytes_to_voxels(bytes: &[u8]) -> Vec<VoxelData> {
    let voxel_size = std::mem::size_of::<VoxelData>();
    let len = bytes.len() / voxel_size;
    let mut voxels = Vec::with_capacity(len);
    unsafe {
        let ptr = bytes.as_ptr() as *const VoxelData;
        for i in 0..len {
            voxels.push(*ptr.add(i));
        }
    }
    voxels
}