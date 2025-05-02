use std::sync::Arc;

use super::{Shape, next_shape_id};
use crate::intersection::Intersection;
use crate::matrix::{Matrix, SqMatrix};
use crate::ray::Ray;

#[derive(Debug)]
pub struct Group {
    pub id: usize,
    pub children: Vec<Arc<dyn Shape + Send + Sync>>,
    pub transfom: SqMatrix<4>,
}

impl Group {
    pub fn new() -> Self {
        Self {
            id: next_shape_id(),
            children: Vec::new(),
            transfom: Matrix::eye(),
        }
    }
    pub fn add_child(&mut self, mut shape: Arc<dyn Shape + Send + Sync>) {
        let combined = &self.transfom * shape.transform();
        let shape_mut =
            Arc::get_mut(&mut shape).expect("Shape Arc was already cloned; add it only once");
        shape_mut.set_transformation(combined);
        self.children.push(shape);
    }
}

impl Shape for Group {
    fn id(&self) -> usize {
        self.id
    }

    fn local_intersect<'a>(
        &'a self,
        _ray: &crate::ray::Ray,
    ) -> Vec<crate::intersection::Intersection<'a>> {
        panic!("local_intersect should never be called on a Group")
    }

    fn local_normal_at(&self, _local_point: crate::vec4::Vec4) -> crate::vec4::Vec4 {
        panic!("local_normal_at should never be called on a Group");
    }

    fn material(&self) -> &crate::material::Material {
        self.children[0].material()
    }

    fn normal_at(&self, _world_point: crate::vec4::Vec4) -> crate::vec4::Vec4 {
        panic!("normal_at should never be called on a Group");
    }

    fn set_material(&mut self, _material: crate::material::Material) {}

    fn set_transformation(&mut self, mat: SqMatrix<4>) {
        for child in &mut self.children {
            let child_original_trafo = &self.transfom.inverse() * child.transform();
            let child_new_trafo = mat.clone() * child_original_trafo;
            let child_mut = Arc::get_mut(child)
                .expect("Child Arc was cloned elsewhere; ensure unique ownership");
            child_mut.set_transformation(child_new_trafo);
        }
        self.transfom = mat;
    }

    fn transform(&self) -> &SqMatrix<4> {
        &self.transfom
    }

    fn intersect<'a>(&'a self, ray: &Ray) -> Vec<Intersection<'a>> {
        let mut xs = Vec::new();
        for child in &self.children {
            let inv = child.transform().inverse();
            let local_ray = ray.transform(&inv);
            xs.extend(child.local_intersect(&local_ray));
        }
        //xs.sort();
        xs
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;

    use crate::{Sphere, matrix::Matrix, ray::Ray, shapes::Shape};

    use super::Group;

    #[test]
    fn add_to_group() {
        let mut g = Group::new();
        let s = Sphere::new();
        let id = s.id;
        g.add_child(Arc::new(s));
        assert_eq!(g.children.is_empty(), false);
        let s = &g.children[0];
        assert_eq!(id, s.id());
    }
    #[test]
    fn intersect_empty() {
        let g = Group::new();
        let r = Ray::new(0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        let xs = g.intersect(&r);
        assert_eq!(xs.is_empty(), true);
    }
    #[test]
    fn intersect_non_empty() {
        let mut g = Group::new();
        let s1 = Sphere::new();
        let id1 = s1.id;
        let mut s2 = Sphere::new();
        let id2 = s2.id;
        s2.set_transformation(Matrix::translation(0.0, 0.0, -3.0));
        let mut s3 = Sphere::new();
        s3.set_transformation(Matrix::translation(5.0, 0.0, 0.0));

        g.add_child(Arc::new(s1));
        g.add_child(Arc::new(s2));
        g.add_child(Arc::new(s3));

        let r = Ray::new(0.0, 0.0, -5.0, 0.0, 0.0, 1.0);
        let mut xs = g.intersect(&r);
        xs.sort();
        assert_eq!(xs.len(), 4);
        assert_eq!(xs[0].object.id(), id2);
        assert_eq!(xs[1].object.id(), id2);
        assert_eq!(xs[2].object.id(), id1);
        assert_eq!(xs[3].object.id(), id1);
    }
    #[test]
    fn intersect_with_trans_group() {
        let mut g = Group::new();
        g.set_transformation(Matrix::scaling(2.0, 2.0, 2.0));
        let mut s = Sphere::new();
        s.set_transformation(Matrix::translation(5.0, 0.0, 0.0));
        g.add_child(Arc::new(s));
        let r = Ray::new(10.0, 0.0, -10.0, 0.0, 0.0, 1.0);
        let mut xs = g.intersect(&r);
        xs.sort();
        println!("{:?}", xs);
        assert_eq!(xs.len(), 2);
    }

    //#[test]
    //fn convert_from_wold_to_object_space() {
    //    let mut g1 = Group::new();
    //    g1.set_transformation(Matrix::rotation_y(PI/2.0));
    //    let mut g2 = Group::new();
    //    g2.set_transformation(Matrix::scaling(1.0, 2.0, 3.0));

    //    let mut s = Sphere::new();
    //    s.set_transformation(Matrix::translation(5.0, 0.0, 0.0));

    //    g2.add_child(Arc::new(s));
    //    g1.add_child(Arc::new(g2));

    //    let world_point = Vec4::point(1.7321, 1.1547, -5.5774);
    //
    //}
}
