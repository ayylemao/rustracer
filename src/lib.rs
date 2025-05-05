pub const SAMPLES_PER_PIXEL: usize = 32;

pub mod camera;
pub mod canvas;
pub mod color;
pub mod intersection;
pub mod light;
pub mod material;
pub mod math;
pub mod matrix;
pub mod ray;
pub mod shapes;
pub use shapes::sphere::Sphere;
pub mod bounds;
pub mod obj_parser;
pub mod patterns;
pub mod transform;
pub mod vec4;
pub mod world;
