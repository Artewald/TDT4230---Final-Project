# Octree-based voxel renderer
This project is my final project in the course 'TDT4230 - Graphics Visualisation'. The project is based on one of my spare-time projects called [ArtewaldEngine](https://github.com/Artewald/ArtewaldEngine) using [Vulkano](https://github.com/vulkano-rs/vulkano): "... a low-levelish API around Vulkan" in their own words. Vulkano is basically a wrapper around Vulkan to make it "safe" to use in Rust.

At the time of writing, ArtewaldEngine only has some basic 3D voxel intersection/raytracing (using a compute shader), so there is no lighting or shadows, and some basic operations to create the voxel octree and update it. It also has some basic 3D movement.

As the final project, I will continue my work on the rendering side of ArtewaldEngine by doing the following:
- [ ] Reorganising the project structure to make it clearer how the renderer works
- [ ] Add raytracing with multiple bounces, this will give the world shadows
- [ ] Add some physics-based rendering (PBR) to the raytracing, like roughness, metalness and albedo. How much I add will be based on how much work it is
- [ ] Further develop and optimize the voxel octree "class"

## Setup
To be able to compile and run the project on Ubuntu (or similar Ubuntu based distros) some packages have to be installed using this command: <br>
```sudo apt-get install build-essential git python cmake libvulkan-dev vulkan-utils``` <br>
For other operating systems or distros please follow the setups provided on [Vulkano's repository](https://github.com/vulkano-rs/vulkano#setup-and-troubleshooting).

## Run the program
To run the program use: ```cargo run``` in the root folder of the project.
