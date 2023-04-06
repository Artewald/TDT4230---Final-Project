use std::fs::{read_to_string, write};

fn main() {
    let mut shader = read_to_string("./resources/shaders/shaders.pre").unwrap();

    let raytracer = read_to_string("./resources/shaders/ray_tracer.comp").unwrap();

    let recursion = read_to_string("./resources/shaders/recursive.rec").unwrap();

    let ray_bounce = read_to_string("./resources/shaders/ray_bounce.comp").unwrap();

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

    let mut times_to_bounce_rays = 0;
    for line in raytracer.clone().split('\n').into_iter() {
        if line.contains("const uint AMOUNT_OF_RAY_BOUNCES = ") {
            let mut words = line.split(" = ");
            words.next().unwrap();
            times_to_bounce_rays = words
                .next()
                .unwrap()
                .split(';')
                .next()
                .unwrap()
                .trim()
                .parse()
                .unwrap();
        }
    }

    let ray_bounces_raw = ray_bounce
        .repeat(times_to_bounce_rays)
        .replace("\n", &format!("\n{}", "    ".repeat(2)));

    let mut counter = 0;
    let ray_bounces_iter = ray_bounces_raw.split("\n");
    let mut ray_bounces = String::new();
    ray_bounces += "\n";
    for line in ray_bounces_iter {
        if line.contains("NN") {
            ray_bounces += &line.replace("NN", &counter.to_string());
            counter += 1;
        } else {
            ray_bounces += &line;
        }
        if !line.contains("\n") {
            ray_bounces += "\n";
        }
    }

    shader += &raytracer
        .replace("//RECURSION_MARKER", &recursed)
        .replace("//RAY_BOUNCES_MARKER", &ray_bounces);
    shader += &read_to_string("./resources/shaders/shaders.post").unwrap();

    write(
        "src/renderer/window_handler/winit_vulkan_handler/shader.rs",
        shader,
    )
    .unwrap();
}
