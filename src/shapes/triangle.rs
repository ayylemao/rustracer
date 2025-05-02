use super::{Shape, next_shape_id};
use crate::{
    intersection::Intersection,
    material::Material,
    math::EPSILON,
    matrix::{Matrix, SqMatrix},
    vec4::Vec4,
};

#[derive(Debug)]
pub struct Triangle {
    pub id: usize,
    pub transform: SqMatrix<4>,
    pub material: Material,
    pub p1: Vec4,
    pub p2: Vec4,
    pub p3: Vec4,
    pub e1: Vec4,
    pub e2: Vec4,
    pub normal: Vec4,
    pub inverse: SqMatrix<4>,
}

impl Triangle {
    pub fn new(p1: Vec4, p2: Vec4, p3: Vec4) -> Self {
        let e1 = p2 - p1;
        let e2 = p3 - p1;
        let normal = e2.cross(&e1).norm();
        Self {
            id: next_shape_id(),
            transform: Matrix::eye(),
            material: Material::default(),
            p1,
            p2,
            p3,
            e1,
            e2,
            normal,
            inverse: Matrix::eye(),
        }
    }
}

impl Shape for Triangle {
    fn id(&self) -> usize {
        self.id
    }

    fn local_intersect<'a>(
        &'a self,
        ray: &crate::ray::Ray,
    ) -> Vec<crate::intersection::Intersection<'a>> {
        let dir_cross_e2 = ray.direction.cross(&self.e2);
        let det = self.e1.dot(&dir_cross_e2);
        if det.abs() < EPSILON {
            return vec![];
        }

        let f = 1.0 / det;
        let p1_to_origin = ray.origin - self.p1;
        let u = f * p1_to_origin.dot(&dir_cross_e2);
        if u < 0.0 || u > 1.0 {
            return vec![];
        }

        let origin_cross_e1 = p1_to_origin.cross(&self.e1);
        let v = f * ray.direction.dot(&origin_cross_e1);
        if v < 0.0 || (u + v) > 1.0 {
            return vec![];
        }

        let t = f * self.e2.dot(&origin_cross_e1);
        vec![Intersection::new(t, self)]
    }

    fn local_normal_at(&self, _local_point: Vec4) -> Vec4 {
        self.normal
    }

    fn material(&self) -> &Material {
        &self.material
    }

    fn normal_at(&self, _world_point: Vec4) -> Vec4 {
        self.normal
    }

    fn set_material(&mut self, material: Material) {
        self.material = material;
    }

    fn set_transformation(&mut self, mat: SqMatrix<4>) {
        self.transform = mat.clone();
        self.inverse = mat.inverse();
    }

    fn transform(&self) -> &SqMatrix<4> {
        &self.transform
    }

    fn inverse(&self) -> &Matrix<4, 4> {
        &self.inverse
    }
}

#[cfg(test)]
pub mod tests {

    use crate::{ray::Ray, shapes::Shape, vec4::Vec4};

    use super::Triangle;

    #[test]
    pub fn cons_triangle() {
        let p1 = Vec4::point(0.0, 1.0, 0.0);
        let p2 = Vec4::point(-1.0, 0.0, 0.0);
        let p3 = Vec4::point(1.0, 0.0, 0.0);
        let t = Triangle::new(p1, p2, p3);
        assert_eq!(t.e1, Vec4::vector(-1.0, -1.0, 0.0));
        assert_eq!(t.e2, Vec4::vector(1.0, -1.0, 0.0));
        assert_eq!(t.normal, Vec4::vector(0.0, 0.0, -1.0));

        let n1 = t.local_normal_at(Vec4::point(-0.5, 0.75, 0.0));
        assert_eq!(n1, t.normal);
        let n2 = t.local_normal_at(Vec4::point(0.5, 0.25, 0.0));
        assert_eq!(n2, t.normal);
    }

    #[test]
    fn intersect_paralell_ray() {
        let p1 = Vec4::point(0.0, 1.0, 0.0);
        let p2 = Vec4::point(-1.0, 0.0, 0.0);
        let p3 = Vec4::point(1.0, 0.0, 0.0);
        let t = Triangle::new(p1, p2, p3);

        let r = Ray::new(0.0, -1.0, -2.0, 0.0, 1.0, 0.0);
        let xs = t.local_intersect(&r);
        assert_eq!(xs.is_empty(), true);

        let r = Ray::new(1.0, 1.0, -2.0, 0.0, 0.0, 1.0);
        let xs = t.local_intersect(&r);
        assert_eq!(xs.is_empty(), true);

        let r = Ray::new(-1.0, 1.0, -2.0, 0.0, 0.0, 1.0);
        let xs = t.local_intersect(&r);
        assert_eq!(xs.is_empty(), true);

        let r = Ray::new(0.0, -1.0, -2.0, 0.0, 0.0, 1.0);
        let xs = t.local_intersect(&r);
        assert_eq!(xs.is_empty(), true);

        let r = Ray::new(0.0, 0.5, -2.0, 0.0, 0.0, 1.0);
        let xs = t.local_intersect(&r);
        assert_eq!(xs.len(), 1);
        assert_eq!(xs[0].t, 2.0);
    }
}
