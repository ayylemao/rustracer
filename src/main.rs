use raytracer::{
    Sphere,
    camera::Camera,
    color::Color,
    light::PointLight,
    matrix::Matrix,
    patterns::{Pattern, checker::Checker, gradient::Gradient},
    shapes::{Shape, plane::Plane},
    vec4::Vec4,
    world::World,
};
use std::f64::consts::PI;

const WIDTH: usize = 1600;
const HEIGHT: usize = 800;
fn main() {
    // shapes
    let mut floor = Plane::new();
    floor.material.set_color(Color::new(0.0, 0.9, 0.9));
    let mut pat = Gradient::new(Color::magenta(), Color::cyan());
    pat.set_transformation(Matrix::translation(10.0, 0.0, 0.0) * Matrix::scaling(15.0, 1.0, 1.0));
    floor.material.set_pattern(pat);
    floor.material.specular = 5.0;
    floor.material.reflective = 0.6;

    //let mut back = Plane::new();
    //back.material.set_color(Color::cyan());
    //back.set_transformation(Matrix::translation(0.0, 0.0, 10.0) * Matrix::rotation_x(PI / 2.0));
    //back.material.specular = 0.0;

    //let mut midwall = Plane::new();
    //midwall.material.set_color(Color::new(0.9, 0.9, 0.9));
    //midwall.set_transformation(
    //    Matrix::rotation_y(PI / 2.0)
    //        * Matrix::translation(0.0, 0.0, 10.0)
    //        * Matrix::rotation_x(PI / 2.0),
    //);
    //midwall.material.specular = 0.0;

    let mut middle = Sphere::new();
    middle.set_transformation(Matrix::translation(-0.5, 1.0, 0.5));
    middle.material.set_color(Color::new(0.1, 1.0, 0.5));
    middle.material.diffuse = 0.7;
    middle.material.specular = 0.3;
    middle.material.reflective = 0.5;
    let pat = Checker::new(Color::white(), Color::green());
    middle.material.set_pattern(pat);

    let mut right = Sphere::new();
    right.set_transformation(Matrix::translation(1.5, 0.5, -0.5) * Matrix::scaling(0.5, 0.5, 0.5));
    right.material.set_color(Color::new(0.5, 1.0, 0.1));
    right.material.diffuse = 0.7;
    right.material.specular = 0.3;
    right.material.reflective = 0.5;

    let mut left = Sphere::new();
    left.set_transformation(
        Matrix::translation(-1.5, 0.33, -0.75) * Matrix::scaling(0.33, 0.33, 0.33),
    );
    left.material.set_color(Color::new(1.0, 0.8, 0.1));
    left.material.diffuse = 0.7;
    left.material.specular = 0.3;
    left.material.reflective = 0.5;

    // World Setup
    let mut world = World::new(PointLight::new(
        Vec4::point(-10.0, 3.0, 0.0),
        Color::white()
    ));
    world.add_shape(Box::new(floor));
    world.add_shape(Box::new(middle));
    world.add_shape(Box::new(left));
    world.add_shape(Box::new(right));
    //world.add_shape(Box::new(back));
    //world.add_shape(Box::new(midwall));

    let mut camera = Camera::new(WIDTH, HEIGHT, PI / 3.0, 5);
    camera.set_view(
        Vec4::point(-5.0, 2.5, -5.0),
        Vec4::point(1.0, 1.0, 0.0),
        Vec4::vector(0.0, 1.0, 0.0),
    );

    let image = camera.render(&world);
    image.save("image.png");
}
