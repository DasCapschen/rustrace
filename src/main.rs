use crate::renderer::Renderer;

mod camera;
mod pathtracer;
mod ray;
mod renderer;

mod gfx {
    pub mod material;
    pub mod texture;
}

mod math {
    pub mod mat3;
    pub mod onb;
    pub mod pdf;
    pub mod quat;
    pub mod transform;
    pub mod vec3;
}

mod hit;
mod hittables {
    pub mod aabb;
    pub mod bvh;
    pub mod mesh;
    pub mod primitives;
    pub mod volume;
}

//this should probably go somewhere else to be available globally
enum ErrorCodes {
    Success = 0,
    Usage = 1,
}

fn usage_err(message: &str) -> ! {
    println!("Error: {}", message);
    println!();
    usage();
}

fn usage() -> ! {
    println!("Usage: ./raytrace [OPTIONS]");
    println!("Available Options:");
    println!("    -w, --width NUMBER        set window width");
    println!("    -h, --height NUMBER       set window height");
    println!("    -s, --samples NUMBER      number of samples per pixel");
    println!("    -i, --incremental         render the picture incrementally");
    println!("    --help                    show this help");
    std::process::exit(ErrorCodes::Usage as i32);
}

fn parse_u32(opt: &Option<String>) -> u32 {
    if let Some(string) = opt {
        if let Ok(int) = string.parse::<u32>() {
            int
        } else {
            usage_err("Argument was not a number!");
        }
    } else {
        usage_err("Argument missing!");
    }
}

fn main() {
    let mut args = std::env::args();

    let mut width = 800;
    let mut height = 600;
    let mut samples = 48;
    let mut incremental = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-w" | "--width" => {
                width = parse_u32(&args.next());
            }
            "-h" | "--height" => {
                height = parse_u32(&args.next());
            }
            "-s" | "--samples" => {
                samples = parse_u32(&args.next());
            }
            "-i" | "--incremental" => {
                incremental = true;
            }
            "--help" => {
                usage();
            }
            _ => {}
        }
    }

    Renderer::new(width, height, samples, incremental)
        .build_scene()
        .run();
}
