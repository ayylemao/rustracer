use raytracer::{
    camera::Camera,
    color::Color,
    light::PointLight,
    material::Material,
    matrix::Matrix,
    obj_parser::Parser,
    patterns::checker::Checker,
    shapes::{Shape, plane::Plane},
    vec4::Vec4,
    world::World,
};
use std::{f64::consts::PI, sync::Arc};

const WIDTH: usize = 2000;
const HEIGHT: usize = 1000;
fn main() {
    let mut p = Parser::new();
    let mut teapot = p.parse_file("objects/teapot.obj");
    let mut tmat = Material::default();
    tmat.set_color(Color::orange());
    tmat.reflective = 0.0;
    teapot.set_material(tmat);
    teapot.set_transformation(Matrix::translation(0.0, 0.0, 0.0));

    let mut floor = Plane::new();
    let mut mat = Material::default();
    mat.reflective = 0.8;
    mat.set_pattern(Checker::new(Color::light_gray(), Color::dark_gray()));
    floor.set_material(mat);
    let mut world: World = World::new(PointLight {
        position: Vec4::point(-10.0, 20.0, -10.0),
        intensity: Color::white(),
    });
    world.add_shape(Arc::new(floor));
    world.add_shape(Arc::new(teapot));
    // === Camera ===
    let mut camera = Camera::new(WIDTH, HEIGHT, PI / 3.0, 5, 16);
    camera.set_view(
        Vec4::point(0.0, 5.0, -8.0),
        Vec4::point(0.0, 1.0, 0.0),
        Vec4::vector(0.0, 1.0, 0.0),
    );

    let image = camera.render(&world);
    image.save("image_test_group.png");
}
