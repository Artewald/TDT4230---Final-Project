# Octree-based voxel renderer

This project is my final project in the course 'TDT4230 - Graphics Visualisation'. The project is based on one of my spare-time projects called [ArtewaldEngine](https://github.com/Artewald/ArtewaldEngine) using [Vulkano](https://github.com/vulkano-rs/vulkano): "... a low-levelish API around Vulkan" in their own words. Vulkano is basically a wrapper around Vulkan to make it "safe" to use in Rust.

## Run the program

To run the program use: `cargo run` in the root folder of the project. This can take a few minutes the first time you run the project.
You can also add commandline arguments to create up to four different scenes. Take a look at this youtube video for a showcase of the scenes and how to enter them: [https://youtu.be/SgFyXyY-RS0](https://youtu.be/SgFyXyY-RS0).
The different commands you can run are:

- `cargo run -- --simple` (this is the default scene created with `cargo run`)
- `cargo run -- --mirror`
- `cargo run -- --neon`
- `cargo run -- --complex`
- You can also add `--recreate` to recreate a scene from scratch. Or else the scene is loaded from a binary file for performance reasons. **Use this argument if the scene does not load properly! But it can take multiple minutes to recreate a scene**

**NOTE: The screen will be black until your mouse enters the window or the window is in focus! Some scenes are very dark at the starting location, so please look at the video mentioned previously for a reference on how to get to the part of the scene that shows something.**

### Problems?

If you are unable to run the program it might be because you are using an outdated version of Rust (this project was built with Rust 1.68) or are missing some libraries. The libraries can be downloaded on Ubuntu (or similar Ubuntu based distros) using this command: <br>
`sudo apt-get install build-essential git python cmake libvulkan-dev vulkan-utils` <br>
For other operating systems or distros please follow the setups provided on [Vulkano's repository](https://github.com/vulkano-rs/vulkano#setup-and-troubleshooting).
