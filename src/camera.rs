use crate::SAMPLES_PER_PIXEL;
use crate::gpu::GPUAccel;
use crate::gpu::gpu_types::GpuRay;
use crate::matrix::SqMatrix;
use crate::{canvas::Canvas, color::Color, matrix::Matrix, ray::Ray, vec4::Vec4, world::World};
use array_init::array_init;
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;

pub struct Camera {
    pub hsize: usize,
    pub vsize: usize,
    pub fov: f32,
    pub transform: Matrix<4, 4>,
    pub pixel_size: f32,
    pub half_width: f32,
    pub half_height: f32,
    pub reflection_max: usize,
    pub max_threads: usize,
    pub inverse: SqMatrix<4>,
}

impl Camera {
    pub fn new(
        hsize: usize,
        vsize: usize,
        fov: f32,
        reflection_max: usize,
        max_threads: usize,
    ) -> Self {
        let half_view = (fov / 2.0).tan();
        let aspect = hsize as f32 / vsize as f32;
        let (half_width, half_height) = if aspect >= 1.0 {
            (half_view, half_view / aspect)
        } else {
            (half_view * aspect, half_view)
        };

        let pixel_size = (half_width * 2.0) / hsize as f32;

        Camera {
            hsize,
            vsize,
            fov,
            transform: Matrix::eye(),
            pixel_size,
            half_width,
            half_height,
            reflection_max,
            max_threads,
            inverse: Matrix::eye(),
        }
    }
    pub fn set_view(&mut self, from: Vec4, to: Vec4, up: Vec4) {
        self.transform = Camera::view_transform(from, to, up);
        self.inverse = self.transform.inverse();
    }
    pub fn set_view_from_matrix(&mut self, mat: Matrix<4, 4>) {
        self.transform = mat.clone();
        self.inverse = mat.inverse();
    }
    pub fn view_transform(from: Vec4, to: Vec4, up: Vec4) -> Matrix<4, 4> {
        let forward = (to - from).norm();
        let upn = up.norm();
        let left = forward.cross(&upn);
        let true_up = left.cross(&forward);
        let val = [
            [left.x, left.y, left.z, 0.0],
            [true_up.x, true_up.y, true_up.z, 0.0],
            [-forward.x, -forward.y, -forward.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        Matrix::from_array(val) * Matrix::translation(-from.x, -from.y, -from.z)
    }
    pub fn rays_for_pixels(&self, px: usize, py: usize) -> [Ray; SAMPLES_PER_PIXEL] {
        let mut rng = rand::rng();
        let n_sqrt = (SAMPLES_PER_PIXEL as f32).sqrt().round() as usize;
        debug_assert_eq!(
            n_sqrt * n_sqrt,
            SAMPLES_PER_PIXEL,
            "SAMPLES_PER_PIXEL must be a perfect square"
        );

        array_init(|i| {
            let xi = i % n_sqrt;
            let yi = i / n_sqrt;

            let dx: f32 = (xi as f32 + rng.random::<f32>()) / n_sqrt as f32;
            let dy: f32 = (yi as f32 + rng.random::<f32>()) / n_sqrt as f32;

            let xoffset = (px as f32 + dx) * self.pixel_size;
            let yoffset = (py as f32 + dy) * self.pixel_size;

            let world_x = self.half_width - xoffset;
            let world_y = self.half_height - yoffset;

            let pixel = &self.inverse * &Vec4::point(world_x, world_y, -1.0);
            let origin = &self.inverse * &Vec4::point(0.0, 0.0, 0.0);
            let direction = (pixel - origin).norm();

            Ray { origin, direction }
        })
    }

    pub fn ray_for_pixel(&self, px: usize, py: usize) -> Ray {
        let px = px as f32;
        let py = py as f32;
        let xoffset = (px + 0.5) * self.pixel_size;
        let yoffset = (py + 0.5) * self.pixel_size;

        let world_x = self.half_width - xoffset;
        let world_y = self.half_height - yoffset;
        let pixel = &self.inverse * &Vec4::point(world_x, world_y, -1.0);
        let origin = &self.inverse * &Vec4::point(0.0, 0.0, 0.0);
        let direction = (pixel - origin).norm();
        Ray { origin, direction }
    }

    pub fn render_gpu(&self, world: &World) -> Canvas {
        let mut image = Canvas::new(self.hsize, self.vsize);
        let total_pixels: u64 = (self.hsize * self.vsize) as u64;
    
        let bar = ProgressBar::new(total_pixels);
        bar.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] [{wide_bar}] {pos}/{len} ({percent}%)",
            )
            .unwrap()
            .progress_chars("=>-"),
        );
    
        let mut gpu = GPUAccel::new("src/gpu/shader.wgsl");
        gpu.populate_shapes(&world.shapes);
    
        let pixels: Vec<(usize, usize)> = (0..self.vsize)
            .flat_map(|y| (0..self.hsize).map(move |x| (x, y)))
            .collect();
    
        let mut rays: Vec<Ray> = Vec::new();
        let mut gpu_rays: Vec<GpuRay> = Vec::new();
        for (x, y) in &pixels {
            let ray = self.ray_for_pixel(*x, *y);
            gpu_rays.push(GpuRay::from_ray(&ray));
            rays.push(ray);
        }
    
        gpu.upload_rays_and_shapes(&gpu_rays);
        gpu.dispatch();
    
        let gpu_intersections = gpu.download_intersections();
    
        // Parallel section
        let results: Vec<((usize, usize), Color)> = pixels
            .into_par_iter()
            .enumerate()
            .map(|(idx, (x, y))| {
                let intersections_for_ray = gpu.get_hits_for_ray(&gpu_intersections, idx);
                let color = world.color_at_gpu(&intersections_for_ray, &rays[idx], self.reflection_max);
                bar.inc(1);
                ((x, y), color)
            })
            .collect();
    
        bar.finish();
    
        for ((x, y), color) in results {
            image.set_pixel(x, y, color);
        }
    
        image
    }
    pub fn render(&mut self, world: &World) -> Canvas {
        let mut image = Canvas::new(self.hsize, self.vsize);

        let total_pixels = (self.hsize * self.vsize) as u64;

        let bar = ProgressBar::new(total_pixels);
        bar.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] [{wide_bar}] {pos}/{len} ({percent}%)",
            )
            .unwrap()
            .progress_chars("=>-"),
        );

        let pool = ThreadPoolBuilder::new()
            .num_threads(self.max_threads)
            .build()
            .unwrap();

        let pixels: Vec<(usize, usize)> = (0..self.vsize)
            .flat_map(|y| (0..self.hsize).map(move |x| (x, y)))
            .collect();
        let results: Vec<((usize, usize), Color)> = pool
            .install(|| {
                pixels.into_par_iter().map(|(x, y)| {
                    let color: Color = if SAMPLES_PER_PIXEL == 1 {
                        let ray = self.ray_for_pixel(x, y);
                        world.color_at(&ray, self.reflection_max)
                    } else {
                        let rays = self.rays_for_pixels(x, y);
                        let mut color_avg = Color::black();
                        for ray in rays {
                            color_avg += world.color_at(&ray, self.reflection_max);
                        }
                        color_avg / SAMPLES_PER_PIXEL as f32
                    };
                    bar.inc(1);
                    ((x, y), color)
                })
            })
            .collect();

        bar.finish();

        for ((x, y), color) in results {
            image.set_pixel(x, y, color);
        }
        image
    }
}

#[cfg(test)]
pub mod tests {
    use std::{
        f32::consts::{PI, SQRT_2},
        sync::Arc,
    };

    use crate::{
        Sphere,
        color::Color,
        light::PointLight,
        material::Material,
        matrix::Matrix,
        obj_parser::Parser,
        patterns::{Pattern, checker::Checker},
        shapes::{Shape, plane::Plane},
        vec4::Vec4,
        world::World,
    };

    use super::Camera;

    #[test]
    fn default_orientation() {
        let from = Vec4::point(0.0, 0.0, 0.0);
        let to = Vec4::point(0.0, 0.0, -1.0);
        let up = Vec4::vector(0.0, 1.0, 0.0);
        let t = Camera::view_transform(from, to, up);
        assert_eq!(t, Matrix::eye());
    }
    #[test]
    fn arbitrary_view_transformation() {
        let from = Vec4::point(1.0, 3.0, 2.0);
        let to = Vec4::point(4.0, -2.0, 8.0);
        let up = Vec4::vector(1.0, 1.0, 0.0);

        let t = Camera::view_transform(from, to, up);

        let vals = [
            [-0.50709, 0.50709, 0.67612, -2.36643],
            [0.76772, 0.60609, 0.12122, -2.82843],
            [-0.35857, 0.59761, -0.71714, 0.00000],
            [0.00000, 0.00000, 0.00000, 1.00000],
        ];
        let expected = Matrix::from_array(vals);
        assert_eq!(t, expected);
    }
    #[test]
    fn center_canvas() {
        let c = Camera::new(201, 101, PI / 2.0, 0, 1);
        let r = c.ray_for_pixel(100, 50);
        assert_eq!(r.origin, Vec4::point(0.0, 0.0, 0.0));
        assert_eq!(r.direction, Vec4::vector(0.0, 0.0, -1.0));
    }
    #[test]
    fn corner_canvas() {
        let c = Camera::new(201, 101, PI / 2.0, 0, 1);
        let r = c.ray_for_pixel(0, 0);
        assert_eq!(r.origin, Vec4::point(0.0, 0.0, 0.0));
        assert_eq!(r.direction, Vec4::vector(0.66519, 0.33259, -0.66851));
    }
    #[test]
    fn corner_canvas_transform() {
        let mut c = Camera::new(201, 101, PI / 2.0, 0, 1);
        c.set_view_from_matrix(Matrix::rotation_y(PI / 4.0) * Matrix::translation(0.0, -2.0, 5.0));
        let r = c.ray_for_pixel(100, 50);

        assert_eq!(r.origin, Vec4::point(0.0, 2.0, -5.0));
        assert_eq!(r.direction, Vec4::vector(SQRT_2 / 2.0, 0.0, -SQRT_2 / 2.0));
    }
    #[test]
    fn render_func() {
        let w = World::default();
        let mut c = Camera::new(11, 11, PI / 2.0, 0, 1);
        let from = Vec4::point(0.0, 0.0, -5.0);
        let to = Vec4::point(0.0, 0.0, 0.0);
        let up = Vec4::vector(0.0, 1.0, 0.0);
        c.set_view(from, to, up);
        let image = c.render(&w);
        assert_eq!(image[(5, 5)], Color::new(0.38066, 0.47583, 0.2855));
    }

    #[test]
    pub fn gpu_render() {
        let mut p = Parser::new();
        let mut teapot = p.parse_file("objects/teapot.obj");
        let mut tmat = Material::default();
        tmat.reflective = 0.8;
        tmat.set_color(Color::orange());
        teapot.set_material(tmat);

        teapot.set_transformation(
            Matrix::translation(-4.0, 0.0, 3.0)
                * Matrix::rotation_x(-PI / 2.0)
                * Matrix::scaling(0.4, 0.4, 0.4),
        );

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

        let mut sb = Sphere::new();
        sb.material.set_color(Color::light_gray());
        sb.material.reflective = 0.9;
        sb.material.transparency = 0.9;
        sb.material.diffuse = 0.1;
        sb.material.ambient = 0.1;
        sb.material.refractive_index = 1.5;

        sb.set_transformation(Matrix::translation(1.0, 2.5, -1.0) * Matrix::scaling(2.5, 2.5, 2.5));

        //world.add_shape(Arc::new(floor));
        world.add_shape(Arc::new(teapot));
        //world.add_shape(Arc::new(sb));
        // === Camera ===
        let mut camera = Camera::new(400, 200, PI / 3.0, 1, 16);
        camera.set_view(
            Vec4::point(0.0, 10.0, -20.0),
            Vec4::point(0.0, 1.0, 0.0),
            Vec4::vector(0.0, 1.0, 0.0),
        );

        //let image = camera.render_gpu(&world);
        //image.save("test_gpu.png");
    }
}
