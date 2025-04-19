use raytracer::{canvas::{self, Canvas}, color::Color, vec4::Vec4};

#[derive(Debug)]
struct Projectile {
    pos: Vec4,
    vel: Vec4
}

fn tick(grav: &Vec4, wind: &Vec4, proj: &mut Projectile) {
    proj.pos = proj.pos + proj.vel;
    proj.vel = proj.vel + *grav + *wind;
}

fn main() {
    let mut p = Projectile {
        pos: Vec4::point(0.0, 1.0, 0.0),
        vel: Vec4::vector(5.0, 5.0, 0.0)
    };
    let grav = Vec4::vector(0.0, -0.1, 0.0);
    let wind = Vec4::vector(-0.01, 0.0, 0.0);

    let mut positions: Vec<Vec4> = Vec::new();
    loop {
        positions.push(p.pos.clone());
        tick(&grav, &wind, &mut p);
        if p.pos.y <= 0.0 {
            break;
        }
    }
    let mut canvas = Canvas::new(100, 100);

    for each in positions.iter() {
        canvas[(each.x.round() as usize, each.y.round() as usize)] = Color::new(1.0, 0.0, 0.0);
        canvas[(each.x.round() as usize + 1, each.y.round() as usize + 1)] = Color::new(1.0, 0.0, 0.0);
        canvas[(each.x.round() as usize, each.y.round() as usize + 1)] = Color::new(1.0, 0.0, 0.0);
        canvas[(each.x.round() as usize + 1, each.y.round() as usize)] = Color::new(1.0, 0.0, 0.0);
    }
    println!("{}", canvas.to_ppm());
}
