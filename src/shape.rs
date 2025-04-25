use std::sync::atomic::{AtomicUsize, Ordering};

use crate::intersection::Intersection;
use crate::material::Material;
use crate::matrix::{Matrix, SqMatrix};
use crate::ray::Ray;
use crate::vec4::Vec4;

static SHAPE_ID: AtomicUsize = AtomicUsize::new(0);

pub trait Shape {
    fn intersetct(&self, ray: &Ray) -> Vec<Intersection>;
    fn normal_at(&self, position: Vec4) -> Vec4;
}

#[derive(Debug, Clone)]
pub struct Sphere {
    pub id: usize,
    pub transform: SqMatrix<4>,
    pub material: Material,
}

impl Sphere {
    pub fn new() -> Sphere {
        let id = SHAPE_ID.fetch_add(1, Ordering::Relaxed);
        Sphere {
            id,
            transform: Matrix::<4, 4>::eye(),
            material: Material::default(),
        }
    }
    pub fn with_transformation(mat: Matrix<4, 4>) -> Self {
        let id = SHAPE_ID.fetch_add(1, Ordering::Relaxed);
        Sphere {
            id,
            transform: mat,
            material: Material::default(),
        }
    }
    pub fn set_transformation(&mut self, mat: Matrix<4, 4>) {
        self.transform = mat;
    }
    pub fn set_material(&mut self, material: Material) {
        self.material = material;
    }
}

impl Shape for Sphere {
    fn intersetct(&self, ray_in: &Ray) -> Vec<Intersection> {
        let mut intersection: Vec<Intersection> = Vec::new();
        let ray = ray_in.transform(&self.transform.inverse());
        let sphere_to_ray = ray.origin - Vec4::point(0.0, 0.0, 0.0);
        let a = ray.direction.dot(&ray.direction);
        let b = ray.direction.dot(&sphere_to_ray) * 2.0;
        let c = sphere_to_ray.dot(&sphere_to_ray) - 1.0;

        let discriminant = b.powf(2.0) - 4.0 * a * c;
        if discriminant < 0.0 {
            return intersection;
        }
        let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
        let t2 = (-b + discriminant.sqrt()) / (2.0 * a);
        intersection.push(Intersection::new(t1, self.id));
        intersection.push(Intersection::new(t2, self.id));
        intersection
    }

    fn normal_at(&self, world_point: Vec4) -> Vec4 {
        let object_point = self.transform.inverse() * world_point;
        let object_normal = object_point - Vec4::point(0.0, 0.0, 0.0);
        let mut world_normal = self.transform.inverse().transpose() * object_normal;
        world_normal.w = 0.0;
        world_normal.norm()
    }
}

#[cfg(test)]
pub mod tests {
    use std::f64::consts::PI;

    use super::*;

    #[test]
    fn sphere_init() {
        let s1 = Sphere::new();
        let s2 = Sphere::new();
        println!("{:?} {:?}", s1, s2)
    }

    #[test]
    fn intersetct() {
        let s1 = Sphere::new();
        let ray = Ray::new(0.0, 0.0, -5.0, 0.0, 0.0, 1.0);
        let xs = s1.intersetct(&ray);
        assert!(xs.len() == 2);
        assert!(xs[0].id == s1.id && xs[1].id == s1.id)
    }
    #[test]
    fn intersect_with_trans() {
        let ray = Ray::new(0.0, 0.0, -5.0, 0.0, 0.0, 1.0);
        let mut s1 = Sphere::new();
        let trans = Matrix::scaling(2.0, 2.0, 2.0);
        s1.set_transformation(trans);
        let xs = s1.intersetct(&ray);
        assert!(xs.len() == 2);
        assert!(xs[0].t == 3.0 && xs[1].t == 7.0);

        let ray = Ray::new(0.0, 0.0, -5.0, 0.0, 0.0, 1.0);
        let mut s1 = Sphere::new();
        let trans = Matrix::translation(5.0, 0.0, 0.0);
        s1.set_transformation(trans);
        let xs = s1.intersetct(&ray);
        assert!(xs.len() == 0);
    }
    #[test]
    fn normal_at() {
        let s = Sphere::new();
        let at = Vec4::point(
            3.0f64.sqrt() / 3.0,
            3.0f64.sqrt() / 3.0,
            3.0f64.sqrt() / 3.0,
        );
        let n = s.normal_at(at);
        assert_eq!(
            n,
            Vec4::vector(
                3.0f64.sqrt() / 3.0,
                3.0f64.sqrt() / 3.0,
                3.0f64.sqrt() / 3.0
            )
        );
    }
    #[test]
    fn normal_trans() {
        let mut s = Sphere::new();
        s.set_transformation(Matrix::translation(0.0, 1.0, 0.0));
        let n = s.normal_at(Vec4::point(0.0, 1.70711, -0.70711));
        assert_eq!(n, Vec4::vector(0.0, 0.70711, -0.70711));

        let mut s = Sphere::new();
        s.set_transformation(Matrix::scaling(1.0, 0.5, 1.0) * Matrix::rotation_z(PI / 5.0));
        let n = s.normal_at(Vec4::point(0.0, 2.0f64.sqrt() / 2.0, -2.0f64.sqrt() / 2.0));
        assert_eq!(n, Vec4::vector(0.0, 0.97014, -0.24254));
    }
}
