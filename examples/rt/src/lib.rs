//! This is a simple packetized ray tracer example which demonstrates
//! interopability with structs in Rust and ISPC.

#[macro_use]
extern crate ispc;
extern crate image;
extern crate rand;
extern crate serde_json;

use std::time::Instant;

use rand::Rng;

pub use vec3f::Vec3f;
pub use camera::Camera;
pub use geom::{Sphere, Plane, Geometry, ISPCGeometry};
pub use lights::PointLight;
pub use material::Lambertian;
pub use scene::Scene;

pub mod vec3f;
pub mod camera;
pub mod geom;
pub mod lights;
pub mod material;
pub mod scene;

ispc_module!(rt);

pub fn render() {
    let scene = Scene::load("./scenes/sphere_on_plane.json");
    let mut framebuffer = vec![0.0; scene.width * scene.height * 3];
    let mut srgb_img_buf = vec![0u8; scene.width * scene.height * 3];
    let mut rng = rand::thread_rng();
    // We need a random seed for each scanline of the image
    let scanline_seeds: Vec<_> = rng.gen_iter::<i32>().take(scene.height).collect();
    unsafe {
        let geom: Vec<_> = scene.geometry.iter().map(|x| x.ispc_equiv()).collect();
        let start = Instant::now();
        rt::render(&scene.camera as *const Camera, geom.as_ptr(), geom.len() as i32, scene.light.ispc_equiv(),
                   scanline_seeds.as_ptr(), scene.width as i32, scene.height as i32, framebuffer.as_mut_ptr());
        let elapsed = start.elapsed();
        println!("Rendering took {}s", elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9);
        rt::framebuffer_to_srgb(framebuffer.as_ptr(), srgb_img_buf.as_mut_ptr(),
                                scene.width as i32, scene.height as i32);
    }
    match image::save_buffer("rt.png", &srgb_img_buf[..], scene.width as u32, scene.height as u32, image::RGB(8)) {
        Ok(_) => println!("Rendered image saved to rt.png"),
        Err(e) => panic!("Error saving image: {}", e),
    };
}

