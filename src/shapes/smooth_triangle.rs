use super::{Shape, next_shape_id};
use crate::bounds::Bounds;
use crate::intersection::Intersection;
use crate::material::Material;
use crate::math::EPSILON;
use crate::matrix::{Matrix, SqMatrix};
use crate::vec4::Vec4;

#[derive(Debug)]
pub struct SmoothTriangle {
    pub id: usize,
    pub transform: SqMatrix<4>,
    pub material: Material,
    pub inverse: SqMatrix<4>,
    pub p1: Vec4,
    pub p2: Vec4,
    pub p3: Vec4,
    pub n1: Vec4,
    pub n2: Vec4,
    pub n3: Vec4,
    pub e1: Vec4,
    pub e2: Vec4,
    pub normal: Vec4,
    pub bounds: Bounds,
}

impl SmoothTriangle {
    pub fn new(p1: Vec4, p2: Vec4, p3: Vec4, n1: Vec4, n2: Vec4, n3: Vec4) -> SmoothTriangle {
        let min_x = p1.x.min(p2.x).min(p3.x);
        let min_y = p1.y.min(p2.y).min(p3.y);
        let min_z = p1.z.min(p2.z).min(p3.z);

        let max_x = p1.x.max(p2.x).max(p3.x);
        let max_y = p1.y.max(p2.y).max(p3.y);
        let max_z = p1.z.max(p2.z).max(p3.z);

        let bounds_min = Vec4::point(min_x, min_y, min_z);
        let bounds_max = Vec4::point(max_x, max_y, max_z);
        let bounds = Bounds::new(bounds_min, bounds_max);
        let id = next_shape_id();
        let e1 = p2 - p1;
        let e2 = p3 - p1;
        let normal = e2.cross(&e1).norm();
        Self {
            id,
            transform: Matrix::eye(),
            material: Material::default(),
            inverse: Matrix::eye(),
            p1,
            p2,
            p3,
            n1,
            n2,
            n3,
            e1,
            e2,
            normal,
            bounds,
        }
    }
}

impl Shape for SmoothTriangle {
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
        vec![Intersection::new(t, self, Some(u), Some(v))]
    }

    fn local_normal_at(&self, _local_point: Vec4, i: &Intersection) -> Vec4 {
        (self.n2 * i.u.unwrap()
            + self.n3 * i.v.unwrap()
            + self.n1 * (1.0 - i.u.unwrap() - i.v.unwrap()))
        .norm()
    }

    fn material(&self) -> &Material {
        &self.material
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn bounds(&self) -> crate::bounds::Bounds {
        self.bounds
    }

    fn as_any_mut(&mut self) ->  &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;

    use super::SmoothTriangle;
    use crate::math::ApproxEq;
    use crate::{
        color::Color, intersection::Intersection, light::PointLight, ray::Ray, shapes::Shape,
        vec4::Vec4, world::World,
    };

    #[test]
    fn smooth_tri1() {
        let p1 = Vec4::point(0.0, 1.0, 0.0);
        let p2 = Vec4::point(-1.0, 0.0, 0.0);
        let p3 = Vec4::point(1.0, 0.0, 0.0);

        let n1 = Vec4::vector(0.0, 1.0, 0.0);
        let n2 = Vec4::vector(-1.0, 1.0, 0.0);
        let n3 = Vec4::vector(1.0, 0.0, 0.0);

        let tri = SmoothTriangle::new(p1, p2, p3, n1, n2, n3);
        let mut w = World::default();
        w.add_shape(Arc::new(tri));
        let tri = w.shapes[2].as_ref();
        let i = Intersection::new(3.5, tri, Some(0.2), Some(0.4));
        assert_eq!(i.u.unwrap(), 0.2);
        assert_eq!(i.v.unwrap(), 0.4);
    }
    #[test]
    fn smooth_tri2() {
        let r = Ray::new(-0.2, 0.3, -2.0, 0.0, 0.0, 1.0);
        let p1 = Vec4::point(0.0, 1.0, 0.0);
        let p2 = Vec4::point(-1.0, 0.0, 0.0);
        let p3 = Vec4::point(1.0, 0.0, 0.0);

        let n1 = Vec4::vector(0.0, 1.0, 0.0);
        let n2 = Vec4::vector(-1.0, 0.0, 0.0);
        let n3 = Vec4::vector(1.0, 0.0, 0.0);

        let tri = SmoothTriangle::new(p1, p2, p3, n1, n2, n3);

        let xs = tri.local_intersect(&r);
        assert!(xs[0].u.unwrap().approx_eq(&0.45));
        assert!(xs[0].v.unwrap().approx_eq(&0.25));
    }
    #[test]
    fn smooth_tri3() {
        let p1 = Vec4::point(0.0, 1.0, 0.0);
        let p2 = Vec4::point(-1.0, 0.0, 0.0);
        let p3 = Vec4::point(1.0, 0.0, 0.0);

        let n1 = Vec4::vector(0.0, 1.0, 0.0);
        let n2 = Vec4::vector(-1.0, 0.0, 0.0);
        let n3 = Vec4::vector(1.0, 0.0, 0.0);
        let u = 0.45;
        let v = 0.25;

        let tri = SmoothTriangle::new(p1, p2, p3, n1, n2, n3);

        let mut w = World::new(PointLight::new(Vec4::point(5.0, 5.0, 5.0), Color::white()));

        w.add_shape(Arc::new(tri));

        let tri = w.shapes[0].as_ref();

        let i = Intersection::new(1.0, tri, Some(u), Some(v));
        let n = tri.normal_at(Vec4::point(0.0, 0.0, 0.0), &i);
        assert_eq!(n, Vec4::vector(-0.5547, 0.83205, 0.0));
    }
    #[test]
    fn smooth_tri4() {
        let p1 = Vec4::point(0.0, 1.0, 0.0);
        let p2 = Vec4::point(-1.0, 0.0, 0.0);
        let p3 = Vec4::point(1.0, 0.0, 0.0);

        let n1 = Vec4::vector(0.0, 1.0, 0.0);
        let n2 = Vec4::vector(-1.0, 0.0, 0.0);
        let n3 = Vec4::vector(1.0, 0.0, 0.0);
        let u = 0.45;
        let v = 0.25;

        let tri = SmoothTriangle::new(p1, p2, p3, n1, n2, n3);

        let mut w = World::new(PointLight::new(Vec4::point(5.0, 5.0, 5.0), Color::white()));

        w.add_shape(Arc::new(tri));

        let tri = w.shapes[0].as_ref();

        let i = Intersection::new(1.0, tri, Some(u), Some(v));

        let r = Ray::new(-0.2, 0.3, -2.0, 0.0, 0.0, 1.0);
        let xs = vec![i];

        let comps = i.prepare_computations(&r, &xs);

        assert_eq!(comps.normalv, Vec4::vector(-0.5547, 0.83205, 0.0));
    }
}
