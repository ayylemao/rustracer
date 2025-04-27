use crate::intersection::Intersection;
use crate::material::Material;
use crate::matrix::SqMatrix;
use crate::ray::Ray;
use crate::vec4::Vec4;
use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};

pub mod plane;
pub mod sphere;

static SHAPE_ID: AtomicUsize = AtomicUsize::new(0);

pub fn next_shape_id() -> usize {
    SHAPE_ID.fetch_add(1, Ordering::Relaxed)
}

pub trait Shape: Debug + Sync + Send {
    fn intersect<'a>(&'a self, ray: &Ray) -> Vec<Intersection<'a>> {
        let local_ray = ray.transform(&self.transform().inverse());
        self.local_intersect(&local_ray)
    }
    fn local_intersect<'a>(&'a self, ray: &Ray) -> Vec<Intersection<'a>>;
    fn normal_at(&self, world_point: Vec4) -> Vec4 {
        let local_point = self.transform().inverse() * world_point;
        let local_normal = self.local_normal_at(local_point);
        let mut world_normal = self.transform().inverse().transpose() * local_normal;
        world_normal.w = 0.0;
        world_normal.norm()
    }
    fn local_normal_at(&self, local_point: Vec4) -> Vec4;
    fn transform(&self) -> &SqMatrix<4>;
    fn material(&self) -> &Material;
    fn set_transformation(&mut self, mat: SqMatrix<4>);
    fn set_material(&mut self, material: Material);
}
