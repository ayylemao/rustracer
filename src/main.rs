use raytracer::{
    camera::Camera, color::Color, light::PointLight, matrix::Matrix, shape::Sphere, vec4::Vec4,
    world::World,
};
use std::f64::consts::PI;

const WIDTH: usize = 1600;
const HEIGHT: usize = 800;
fn main() {
    // shapes
    let mut floor = Sphere::new();
    floor.set_transformation(Matrix::scaling(10.0, 0.01, 10.0));
    floor.material.set_color(Color::new(1.0, 0.9, 0.9));
    floor.material.specular = 0.0;

    let mut left_wall = Sphere::new();
    left_wall.set_transformation(
        Matrix::translation(0.0, 0.0, 5.0)
            * Matrix::rotation_y(-PI / 4.0)
            * Matrix::rotation_x(PI / 2.0)
            * Matrix::scaling(10.0, 0.01, 10.0),
    );
    left_wall.set_material(floor.material.clone());

    let mut right_wall = Sphere::new();
    right_wall.set_transformation(
        Matrix::translation(0.0, 0.0, 5.0)
            * Matrix::rotation_y(PI / 4.0)
            * Matrix::rotation_x(PI / 2.0)
            * Matrix::scaling(10.0, 0.01, 10.0),
    );
    right_wall.set_material(floor.material.clone());

    let mut middle = Sphere::new();
    middle.set_transformation(Matrix::translation(-0.5, 1.0, 0.5));
    middle.material.set_color(Color::new(0.1, 1.0, 0.5));
    middle.material.diffuse = 0.7;
    middle.material.specular = 0.3;

    let mut right = Sphere::new();
    right.set_transformation(Matrix::translation(1.5, 0.5, -0.5) * Matrix::scaling(0.5, 0.5, 0.5));
    right.material.set_color(Color::new(0.5, 1.0, 0.1));
    right.material.diffuse = 0.7;
    right.material.specular = 0.3;

    let mut left = Sphere::new();
    left.set_transformation(
        Matrix::translation(-1.5, 0.33, -0.75) * Matrix::scaling(0.33, 0.33, 0.33),
    );
    left.material.set_color(Color::new(1.0, 0.8, 0.1));
    left.material.diffuse = 0.7;
    left.material.specular = 0.3;

    // World Setup
    let mut world = World::new(PointLight::new(
        Vec4::point(-10.0, 100.0, -10.0),
        Color::white(),
    ));
    world.add_shape(Box::new(floor));
    world.add_shape(Box::new(left_wall));
    world.add_shape(Box::new(right_wall));
    world.add_shape(Box::new(middle));
    world.add_shape(Box::new(left));
    world.add_shape(Box::new(right));

    let mut camera = Camera::new(WIDTH, HEIGHT, PI / 3.0);
    camera.set_view(
        Vec4::point(0.0, 1.5, -5.0),
        Vec4::point(0.0, 1.0, 0.0),
        Vec4::vector(0.0, 1.0, 0.0),
    );

    let image = camera.render(&world);
    image.save("image.png");
}
