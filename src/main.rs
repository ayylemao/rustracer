use raytracer::{
    canvas::Canvas,
    color::Color,
    intersection::Intersection,
    light::PointLight,
    material::Material,
    matrix::Matrix,
    ray::Ray,
    shape::{Shape, Sphere},
    vec4::Vec4,
};
use std::io::{BufWriter, Write};

const WIDTH: usize = 1600;
const HEIGHT: usize = 1600;
fn main() {
    let mut canvas = Canvas::new(WIDTH, HEIGHT);

    let mut sphere = Sphere::new();

    let ray_origin = Vec4::point(0.0, 0.0, -5.0);
    let wall_z = 10.0;
    let wall_size = 7.0;
    let pixel_size = wall_size / WIDTH as f64;
    let half = wall_size / 2.0;

    let transform = Matrix::translation(0.0, 0.0, 2.0);
    //* Matrix::rotation_y(std::f64::consts::PI / 2.0)
    //* Matrix::rotation_z(std::f64::consts::PI / 8.0)
    //* Matrix::shearing(1.23, 0.05, 0.1, 0.6, 0.01, 0.2)
    //* Matrix::scaling(0.6, 1.2, 0.6);
    sphere.set_transformation(transform);

    let mut material = Material::default();
    material.set_color(Color::new(1.0, 0.2, 1.0));
    sphere.set_material(material);

    let light_position = Vec4::point(-30.0, 10.0, -10.0);
    let light_color = Color::white();
    let light = PointLight::new(light_position, light_color);

    for y in 0..HEIGHT {
        let world_y = half - pixel_size * y as f64;
        for x in 0..WIDTH {
            let world_x = -half + pixel_size * x as f64;
            let position = Vec4::point(world_x, world_y, wall_z);
            let ray = Ray::from_vec4(ray_origin, (position - ray_origin).norm());
            let xs = sphere.intersetct(&ray);
            if let Some(hit) = Intersection::hit(&xs) {
                let point = ray.position(hit.t);
                let normal = hit.object.normal_at(point);
                let eye = -ray.direction;
                let color = hit
                    .object
                    .material()
                    .lighting(&light, &point, &eye, &normal);
                canvas.set_pixel(x, y, color);
            }
        }
    }

    let file = std::fs::File::create("test.ppm").unwrap();
    let mut buff = BufWriter::new(file);
    buff.write(canvas.to_ppm().as_bytes()).unwrap();
}
