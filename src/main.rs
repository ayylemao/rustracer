use std::f64::consts::PI;

use raytracer::{canvas::Canvas, color::Color, vec4::Vec4, matrix::Matrix};


fn main() {
    let mut canvas = Canvas::new(100, 100);
    let center = Vec4::point(50.0, 50.0, 0.0);
    let radius = 40.0;
    let base = Vec4::point(0.0, -radius, 0.0); // top of the circle

    for i in 0..12 {
        let angle = i as f64 * std::f64::consts::TAU / 12.0; // 2π / 12
        let rot = Matrix::rotation_z(angle);
        let rotated = &rot * &base;
        let translated = &Matrix::translation(center.x, center.y, 0.0) * &rotated;

        let x = translated.x.round() as usize;
        let y = translated.y.round() as usize;

        if x < canvas.width && y < canvas.height {
            canvas[(x, y)] = Color::new(1.0, 0.0, 0.0);
        }
    }

    println!("{}", canvas.to_ppm());
}
