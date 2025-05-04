use raytracer::{
    camera::Camera,
    color::Color,
    light::PointLight,
    material::Material,
    matrix::Matrix,
    obj_parser::Parser,
    patterns::{checker::Checker, Pattern},
    shapes::{plane::Plane, Shape},
    vec4::Vec4,
    world::World, Sphere,
};
use std::{f64::consts::PI, sync::Arc};

const WIDTH: usize = 1600;
const HEIGHT: usize = 800;
fn main() {
    let mut p = Parser::new();
    let mut teapot = p.parse_file("objects/teapot.obj");
    let mut tmat = Material::default();
    tmat.set_color(Color::orange());
    teapot.set_material(tmat);
    teapot.set_transformation(Matrix::translation(-4.0, 0.0, 3.0) * Matrix::rotation_x(-PI/2.0) * Matrix::scaling(0.4, 0.4, 0.4));


    let mut floor = Plane::new();
    let mut mat = Material::default();
    mat.reflective = 0.8;
    let mut pat = Checker::new(Color::light_gray(), Color::dark_gray());
    pat.set_transformation(Matrix::scaling(5.0, 5.0, 5.0));
    mat.set_pattern(pat);
    floor.set_material(mat);
    
    let mut world: World = World::new(PointLight {
        position: Vec4::point(-10.0, 20.0, -10.0),
        intensity: Color::white(),
    });

    let mut s = Sphere::new();
    s.set_transformation(Matrix::translation(-3.0, 1.0, -3.0));
    s.material.reflective = 0.9;
    s.material.color = Color::red();

    let mut sb = Sphere::new();
    sb.material.set_color(Color::light_gray());
    sb.material.reflective = 0.9;
    sb.material.transparency = 0.9;
    sb.material.diffuse = 0.1;
    sb.material.ambient = 0.1;
    sb.material.refractive_index = 1.5;

    sb.set_transformation(Matrix::translation(1.0, 2.5, -1.0) * Matrix::scaling(2.5, 2.5, 2.5));

    world.add_shape(Arc::new(floor));
    world.add_shape(Arc::new(teapot));
    world.add_shape(Arc::new(s));
    world.add_shape(Arc::new(sb));
    // === Camera ===
    let mut camera = Camera::new(WIDTH, HEIGHT, PI / 3.0, 5, 16);
    camera.set_view(
        Vec4::point(0.0, 10.0, -20.0),
        Vec4::point(0.0, 1.0, 0.0),
        Vec4::vector(0.0, 1.0, 0.0),
    );

    let image = camera.render(&world);
    image.save("teapot-post-change.png");
}