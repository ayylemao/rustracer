use crate::bounds::Bounds;
use crate::intersection::Intersection;
use crate::material::Material;
use crate::matrix::{Matrix, SqMatrix};
use crate::ray::Ray;
use crate::vec4::Vec4;
use std::any::Any;
use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};

pub mod cube;
pub mod group;
pub mod plane;
pub mod smooth_triangle;
pub mod sphere;
pub mod triangle;

static SHAPE_ID: AtomicUsize = AtomicUsize::new(1);

pub fn next_shape_id() -> usize {
    SHAPE_ID.fetch_add(1, Ordering::Relaxed)
}

pub trait Shape: Debug + Sync + Send + Any {
    fn intersect<'a>(&'a self, ray: &Ray) -> Vec<Intersection<'a>> {
        let local_ray = ray.transform(self.inverse());
        self.local_intersect(&local_ray)
    }
    fn local_intersect<'a>(&'a self, ray: &Ray) -> Vec<Intersection<'a>>;
    fn normal_at(&self, world_point: Vec4, i: &Intersection) -> Vec4 {
        let local_point = self.inverse() * &world_point;
        let local_normal = self.local_normal_at(local_point, i);
        let mut world_normal = self.inverse().transpose() * local_normal;
        world_normal.w = 0.0;
        world_normal.norm()
    }
    fn local_normal_at(&self, local_point: Vec4, i: &Intersection) -> Vec4;
    fn transform(&self) -> &SqMatrix<4>;
    fn material(&self) -> &Material;
    fn set_transformation(&mut self, mat: SqMatrix<4>);
    fn set_material(&mut self, material: Material);
    fn id(&self) -> usize;
    fn inverse(&self) -> &Matrix<4, 4>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn bounds(&self) -> Bounds;
}
