use std::sync::Arc;

use super::{Shape, next_shape_id};
use crate::intersection::Intersection;
use crate::material::Material;
use crate::matrix::{Matrix, SqMatrix};
use crate::ray::Ray;
use crate::vec4::Vec4;

#[derive(Debug)]
pub struct Group {
    pub id: usize,
    pub children: Vec<Arc<dyn Shape + Send + Sync>>,
    pub parent: Option<Arc<dyn Shape + Send + Sync>>,
    pub transfom: SqMatrix<4>,
}

impl Group {
    pub fn new() -> Self {
        Self {
            id: next_shape_id(),
            children: Vec::new(),
            transfom: Matrix::eye(),
            parent: None
        }
    }
    pub fn add_shape(&mut self, shape: Arc<dyn Shape>) {
        shape.set_parent(Arc::new(self));
        self.children.push(shape);
    }
}

impl Shape for Group {
    fn id(&self) -> usize {
        self.id
    }

    fn intersect<'a>(&'a self, ray: &crate::ray::Ray) -> Vec<crate::intersection::Intersection<'a>> {
        vec![Intersection::new(0.0, self.children[0].as_ref())]
    }

    fn local_intersect<'a>(&'a self, ray: &crate::ray::Ray) -> Vec<crate::intersection::Intersection<'a>> {
        vec![Intersection::new(0.0, self.children[0].as_ref())]
    }

    fn local_normal_at(&self, local_point: crate::vec4::Vec4) -> crate::vec4::Vec4 {
        Vec4::vector(0.0, 0.0, 0.0)
    }

    fn material(&self) -> &crate::material::Material {
        self.children[0].material()
    }

    fn normal_at(&self, world_point: crate::vec4::Vec4) -> crate::vec4::Vec4 {
        Vec4::vector(0.0, 0.0, 0.0)
    }

    fn set_material(&mut self, material: crate::material::Material) {

    }

    fn set_transformation(&mut self, mat: SqMatrix<4>) {
        self.transfom = mat;
    }

    fn transform(&self) -> &SqMatrix<4> {
        &self.transfom
    }

    fn set_parent(&mut self, parent: Arc<dyn Shape>) {
        
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;

    use crate::Sphere;

    use super::Group;


    #[test]
    fn add_to_group() {
        let mut g = Group::new();
        let s = Sphere::new();
        g.add_shape(Arc::new(s));
    }
}
