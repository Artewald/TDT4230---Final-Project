use std::fs::{read_to_string, write};

fn main() {
    let mut shader = read_to_string("./resources/shaders/shaders.pre").unwrap();

    let raytracer = read_to_string("./resources/shaders/ray_tracer.comp").unwrap();

    let recursion = read_to_string("./resources/shaders/recursive.rec").unwrap();

    let mut recursed = String::new();
    for i in 0..17 {
        recursed += &recursion
            .replace("NN", &i.to_string())
            .replace("MMM", &(i + 1).to_string())
            .replace("\n", &format!("\n{}", "    ".repeat(i + 1)));
    }
    recursed += "\n";
    for i in 0..17 {
        recursed += &format!("{}}}\n", "    ".repeat(17 - i));
    }

    shader += &raytracer.replace("//RECURSION_MARKER", &recursed);
    shader += &read_to_string("./resources/shaders/shaders.post").unwrap();

    write(
        "src/renderer/window_handler/winit_vulkan_handler/shader.rs",
        shader,
    )
    .unwrap();
}
