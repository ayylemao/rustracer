use super::{Shape, next_shape_id};
use crate::bounds::Bounds;
use crate::intersection::Intersection;
use crate::material::Material;
use crate::matrix::{Matrix, SqMatrix};
use crate::ray::Ray;
use crate::vec4::Vec4;

#[derive(Debug)]
pub struct Sphere {
    pub id: usize,
    pub transform: SqMatrix<4>,
    pub material: Material,
    pub inverse: SqMatrix<4>,
    pub bounds: Bounds,
}

impl Sphere {
    pub fn get_bounds() -> Bounds {
        let bounds_min = Vec4::point(-1.0, -1.0, -1.0);
        let bounds_max = Vec4::point(1.0, 1.0, 1.0);
        Bounds::new(bounds_min, bounds_max)
    }

    pub fn new() -> Sphere {
        let id = next_shape_id();
        Sphere {
            id,
            transform: Matrix::eye(),
            material: Material::default(),
            inverse: Matrix::eye(),
            bounds: Sphere::get_bounds(),
        }
    }
    pub fn with_transformation(mat: Matrix<4, 4>) -> Self {
        let id = next_shape_id();
        Sphere {
            id,
            transform: mat.clone(),
            material: Material::default(),
            inverse: mat.inverse(),
            bounds: Sphere::get_bounds(),
        }
    }
    pub fn glas(refractive_index: f32) -> Sphere {
        let id = next_shape_id();
        let mut m1 = Material::glas();
        m1.refractive_index = refractive_index;
        Sphere {
            id,
            transform: Matrix::eye(),
            material: m1,
            inverse: Matrix::eye(),
            bounds: Sphere::get_bounds(),
        }
    }
}

impl Shape for Sphere {
    fn local_intersect<'a>(&'a self, ray: &Ray) -> Vec<Intersection<'a>> {
        let mut intersection: Vec<Intersection> = Vec::new();
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
        intersection.push(Intersection::new(t1, self, None, None));
        intersection.push(Intersection::new(t2, self, None, None));
        intersection
    }

    fn local_normal_at(&self, local_point: Vec4, _i: &Intersection) -> Vec4 {
        local_point - Vec4::point(0.0, 0.0, 0.0)
    }

    fn transform(&self) -> &SqMatrix<4> {
        &self.transform
    }

    fn material(&self) -> &Material {
        &self.material
    }

    fn set_transformation(&mut self, mat: Matrix<4, 4>) {
        self.transform = mat.clone();
        self.inverse = mat.inverse();
    }

    fn set_material(&mut self, material: Material) {
        self.material = material;
    }

    fn id(&self) -> usize {
        self.id
    }

    fn inverse(&self) -> &Matrix<4, 4> {
        &self.inverse
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn bounds(&self) -> crate::bounds::Bounds {
        self.bounds
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
pub mod tests {
    //use std::f32::consts::PI;

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
        let xs = s1.intersect(&ray);
        assert!(xs.len() == 2);
    }
    #[test]
    fn intersect_with_trans() {
        let ray = Ray::new(0.0, 0.0, -5.0, 0.0, 0.0, 1.0);
        let mut s1 = Sphere::new();
        let trans = Matrix::scaling(2.0, 2.0, 2.0);
        s1.set_transformation(trans);
        let xs = s1.intersect(&ray);
        assert!(xs.len() == 2);
        assert!(xs[0].t == 3.0 && xs[1].t == 7.0);

        let ray = Ray::new(0.0, 0.0, -5.0, 0.0, 0.0, 1.0);
        let mut s1 = Sphere::new();
        let trans = Matrix::translation(5.0, 0.0, 0.0);
        s1.set_transformation(trans);
        let xs = s1.intersect(&ray);
        assert!(xs.len() == 0);
    }
    //#[test]
    //fn normal_at() {
    //    let s = Sphere::new();
    //    let at = Vec4::point(
    //        3.0f32.sqrt() / 3.0,
    //        3.0f32.sqrt() / 3.0,
    //        3.0f32.sqrt() / 3.0,
    //    );
    //    let n = s.normal_at(at);
    //    assert_eq!(
    //        n,
    //        Vec4::vector(
    //            3.0f32.sqrt() / 3.0,
    //            3.0f32.sqrt() / 3.0,
    //            3.0f32.sqrt() / 3.0
    //        )
    //    );
    //}
    //#[test]
    //fn normal_trans() {
    //    let mut s = Sphere::new();
    //    s.set_transformation(Matrix::translation(0.0, 1.0, 0.0));
    //    let n = s.normal_at(Vec4::point(0.0, 1.70711, -0.70711));
    //    assert_eq!(n, Vec4::vector(0.0, 0.70711, -0.70711));

    //    let mut s = Sphere::new();
    //    s.set_transformation(Matrix::scaling(1.0, 0.5, 1.0) * Matrix::rotation_z(PI / 5.0));
    //    let n = s.normal_at(Vec4::point(0.0, 2.0f32.sqrt() / 2.0, -2.0f32.sqrt() / 2.0));
    //    assert_eq!(n, Vec4::vector(0.0, 0.97014, -0.24254));
    //}
}
