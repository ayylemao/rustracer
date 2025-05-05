use std::sync::Arc;

use super::{Shape, next_shape_id};
use crate::bounds::Bounds;
use crate::intersection::Intersection;
use crate::matrix::{Matrix, SqMatrix};
use crate::ray::Ray;
use crate::vec4::Vec4;

#[derive(Debug)]
pub struct Group {
    pub id: usize,
    pub children: Vec<Arc<dyn Shape + Send + Sync>>,
    pub transfom: SqMatrix<4>,
    pub inverse: SqMatrix<4>,
}

impl Group {
    pub fn new() -> Self {
        Self {
            id: next_shape_id(),
            children: Vec::new(),
            transfom: Matrix::eye(),
            inverse: Matrix::eye(),
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

    fn local_normal_at(
        &self,
        _local_point: crate::vec4::Vec4,
        _i: &Intersection,
    ) -> crate::vec4::Vec4 {
        panic!("local_normal_at should never be called on a Group");
    }

    fn material(&self) -> &crate::material::Material {
        self.children[0].material()
    }

    fn normal_at(&self, _world_point: crate::vec4::Vec4, _i: &Intersection) -> crate::vec4::Vec4 {
        panic!("normal_at should never be called on a Group");
    }

    fn set_material(&mut self, material: crate::material::Material) {
        for child in &mut self.children {
            let child_mut = Arc::get_mut(child)
                .expect("Child Arc was cloned elsewhere; ensure unique ownership");
            child_mut.set_material(material.clone());
        }
    }

    fn set_transformation(&mut self, mat: SqMatrix<4>) {
        let old_group = self.transfom.clone();
        let new_group = mat.clone();
        let delta = new_group * old_group.inverse();

        for child in &mut self.children {
            let child_mut = Arc::get_mut(child)
                .expect("Child Arc was cloned elsewhere; ensure unique ownership");

            if let Some(group) = child_mut.as_any_mut().downcast_mut::<Group>() {
                group.set_transformation(delta.clone());
            } else {
                let child_transform = child_mut.transform();
                let new_child_transform = &delta * child_transform;
                child_mut.set_transformation(new_child_transform);
            }
        }
        self.transfom = mat.clone();
        self.inverse = mat.inverse();
    }

    fn transform(&self) -> &SqMatrix<4> {
        &self.transfom
    }

    fn intersect<'a>(&'a self, ray: &Ray) -> Vec<Intersection<'a>> {
        let mut xs = Vec::new();
        for child in &self.children {
            xs.extend(child.intersect(&ray));
        }
        xs
    }

    fn inverse(&self) -> &Matrix<4, 4> {
        &self.inverse
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) ->  &mut dyn std::any::Any {
        self
    }

    fn bounds(&self) -> crate::bounds::Bounds {
        let mut x_min = f64::INFINITY;
        let mut y_min = f64::INFINITY;
        let mut z_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        let mut y_max = f64::NEG_INFINITY;
        let mut z_max = f64::NEG_INFINITY;
    
        for child in &self.children {
            let b = if child.as_any().is::<Group>() {
                child.bounds()
            } else {
                child.bounds().transform(child.transform())
            };
    
            if !b.is_finite() {
                continue;
            }
    
            x_min = x_min.min(b.min.x);
            y_min = y_min.min(b.min.y);
            z_min = z_min.min(b.min.z);
            x_max = x_max.max(b.max.x);
            y_max = y_max.max(b.max.y);
            z_max = z_max.max(b.max.z);
        }
    
        Bounds::new(
            Vec4::point(x_min, y_min, z_min),
            Vec4::point(x_max, y_max, z_max),
        )
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
    //#[test]
    //fn intersect_with_trans_group() {
    //    let mut g = Group::new();
    //    let group_trans = Matrix::scaling(2.0, 2.0, 2.0);
    //    g.set_transformation(group_trans.clone());

    //    let mut s = Sphere::new();

    //    let sphere_trans = Matrix::translation(5.0, 0.0, 0.0);

    //    s.set_transformation(sphere_trans.clone());

    //    g.add_child(Arc::new(s));

    //    let post_child_transform = g.children[0].transform();
    //    let post_child_inverse = g.children[0].inverse();
    //    assert_eq!(post_child_transform.inverse(), post_child_inverse.clone());
    //    assert_eq!(group_trans * sphere_trans, post_child_transform.clone());

    //    let r = Ray::new(10.0, 0.0, -10.0, 0.0, 0.0, 1.0);
    //    let mut xs = g.intersect(&r);
    //    xs.sort();
    //    println!("{:?}", xs);
    //    assert_eq!(xs.len(), 2);
    //}

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
