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
use std::{f64::consts::PI, sync::Arc};

const WIDTH: usize = 1600;
const HEIGHT: usize = 800;
fn main() {
    // === Floor ===
    let mut floor = Plane::new();
    let floor_pat = Checker::new(Color::dark_gray(), Color::light_gray());
    //floor_pat.set_transformation(Matrix::scaling(0.25, 0.25, 0.25));
    floor.material.set_pattern(floor_pat);
    floor.material.specular = 0.5;
    floor.material.reflective = 0.8;

    // === Back Wall ===
    let mut wall = Plane::new();
    let mut wall_pat = Checker::new(Color::black(), Color::white());
    wall_pat.set_transformation(Matrix::scaling(0.5, 0.5, 0.5));
    wall.material.set_pattern(wall_pat);
    wall.set_transformation(Matrix::translation(0.0, 0.0, 10.0) * Matrix::rotation_x(PI / 2.0));
    wall.material.specular = 0.0;

    // === Center Sphere ===
    let mut center = Sphere::new();
    center.set_transformation(Matrix::translation(0.0, 1.0, 0.0));
    center.material.diffuse = 0.6;
    center.material.specular = 0.5;
    center.material.reflective = 0.8;
    center.material.set_color(Color::yellow());
    //let mut stripe_pat = StripePattern::new(Color::magenta(), Color::white());
    //stripe_pat.set_transformation(Matrix::rotation_y(PI / 4.0) * Matrix::scaling(0.2, 0.2, 0.2));
    //center.material.set_pattern(stripe_pat);

    // === Left Gradient Sphere ===
    let mut left = Sphere::new();
    left.set_transformation(
        Matrix::translation(-0.8, 0.33, -1.5) * Matrix::scaling(0.33, 0.33, 0.33),
    );
    let grad_left = Checker::new(Color::cyan(), Color::blue());
    //grad_left.set_transformation(Matrix::scaling(3.0, 2.0, 2.0));
    left.material.set_pattern(grad_left);
    left.material.specular = 0.2;
    left.material.reflective = 0.8;

    // === Right Gradient Sphere ===
    let mut right = Sphere::new();
    right.set_transformation(Matrix::translation(1.5, 0.5, -1.0) * Matrix::scaling(0.5, 0.5, 0.5));
    let mut grad_right = Gradient::new(Color::orange(), Color::red());
    grad_right.set_transformation(Matrix::scaling(2.0, 1.0, 1.0));
    right.material.set_pattern(grad_right);
    right.material.specular = 0.2;
    right.material.reflective = 0.8;

    // === World ===
    let mut world = World::new(PointLight::new(
        Vec4::point(-10.0, 10.0, -10.0),
        Color::white(),
    ));

    world.add_shape(Arc::new(floor));
    //world.add_shape(Arc::new(wall));
    world.add_shape(Arc::new(center));
    world.add_shape(Arc::new(left));
    world.add_shape(Arc::new(right));

    // === Camera ===
    let mut camera = Camera::new(WIDTH, HEIGHT, PI / 3.0, 5, 16);
    camera.set_view(
        Vec4::point(0.0, 2.0, -6.0),
        Vec4::point(0.0, 1.0, 0.0),
        Vec4::vector(0.0, 1.0, 0.0),
    );

    let image = camera.render(&world);
    image.save("image_test.png");
}
