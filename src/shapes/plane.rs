use super::{Shape, next_shape_id};
use crate::{
    intersection::Intersection,
    material::Material,
    math::EPSILON,
    matrix::{Matrix, SqMatrix},
    vec4::Vec4,
};

#[derive(Debug)]
pub struct Plane {
    pub id: usize,
    pub transform: SqMatrix<4>,
    pub material: Material,
    pub inverse: SqMatrix<4>,
}

impl Plane {
    pub fn new() -> Plane {
        let id = next_shape_id();
        Plane {
            id,
            transform: Matrix::eye(),
            material: Material::default(),
            inverse: Matrix::eye(),
        }
    }
    pub fn with_transformation(mat: Matrix<4, 4>) -> Self {
        let id = next_shape_id();
        Plane {
            id,
            transform: mat.clone(),
            material: Material::default(),
            inverse: mat.inverse(),
        }
    }
}

impl Shape for Plane {
    fn local_intersect<'a>(
        &'a self,
        ray: &crate::ray::Ray,
    ) -> Vec<crate::intersection::Intersection<'a>> {
        if ray.direction.y.abs() < EPSILON {
            return vec![];
        } else {
            let t = -ray.origin.y / ray.direction.y;
            let i = Intersection::new(t, self);
            return vec![i];
        }
    }

    fn local_normal_at(&self, _local_point: crate::vec4::Vec4) -> crate::vec4::Vec4 {
        Vec4::vector(0.0, 1.0, 0.0)
    }

    fn transform(&self) -> &SqMatrix<4> {
        &self.transform
    }

    fn material(&self) -> &Material {
        &self.material
    }

    fn set_transformation(&mut self, mat: SqMatrix<4>) {
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
}

#[cfg(test)]
pub mod tests {
    use crate::{ray::Ray, shapes::Shape, vec4::Vec4};

    use super::Plane;

    #[test]
    fn intersect_with_plane() {
        let p = Plane::new();
        let r = Ray::from_vec4(Vec4::point(0.0, 10.0, 0.0), Vec4::vector(0.0, 0.0, 1.0));
        let xs = p.local_intersect(&r);
        assert_eq!(xs.len(), 0);
        let r = Ray::from_vec4(Vec4::point(0.0, 0.0, 0.0), Vec4::vector(0.0, 0.0, 1.0));
        let xs = p.local_intersect(&r);
        assert_eq!(xs.len(), 0);

        let p = Plane::new();
        let r = Ray::from_vec4(Vec4::point(0.0, 1.0, 0.0), Vec4::vector(0.0, -1.0, 0.0));
        let xs = p.local_intersect(&r);
        assert_eq!(xs.len(), 1);
        assert_eq!(xs[0].t, 1.0);

        let p = Plane::new();
        let r = Ray::from_vec4(Vec4::point(0.0, -1.0, 0.0), Vec4::vector(0.0, 1.0, 0.0));
        let xs = p.local_intersect(&r);
        assert_eq!(xs.len(), 1);
        assert_eq!(xs[0].t, 1.0);
    }
}
