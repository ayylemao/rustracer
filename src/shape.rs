use std::sync::atomic::{AtomicUsize, Ordering};

use crate::intersection::Intersection;
use crate::matrix::{Matrix, SqMatrix};
use crate::ray::Ray;
use crate::vec4::Vec4;

static SHAPE_ID: AtomicUsize = AtomicUsize::new(0);

pub trait Shape {
    fn intersetct(&self, ray: &Ray) -> Vec<Intersection>;
}

#[derive(Debug, Clone)]
pub struct Sphere {
    pub id: usize,
    pub transform: SqMatrix<4>,
}

impl Sphere {
    pub fn new() -> Sphere {
        let id = SHAPE_ID.fetch_add(1, Ordering::Relaxed);
        Sphere {
            id,
            transform: Matrix::<4, 4>::eye(),
        }
    }
    pub fn set_transformation(&mut self, mat: Matrix<4, 4>) {
        self.transform = mat;
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
}

#[cfg(test)]
pub mod tests {
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
}
