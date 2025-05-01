use crate::matrix::Matrix;
use crate::shapes::Shape;
use crate::{color::Color, vec4::Vec4};
use std::clone::Clone;
use std::fmt::Debug;

pub mod checker;
pub mod gradient;
pub mod ring;
pub mod stripe_pattern;

pub trait Pattern: Debug + Sync + Send {
    fn color_at(&self, point: &Vec4) -> Color;
    fn transform(&self) -> &Matrix<4, 4>;
    fn set_transformation(&mut self, matrix: Matrix<4, 4>);
    fn pattern_at(&self, object: &dyn Shape, world_point: &Vec4) -> Color {
        let object_point = object.transform().inverse() * world_point;
        let pattern_point = self.transform().inverse() * object_point;
        self.color_at(&pattern_point)
    }
}

#[derive(Debug, Clone)]
pub struct TestPattern {
    pub transform: Matrix<4, 4>,
}

impl TestPattern {
    pub fn new() -> Self {
        TestPattern {
            transform: Matrix::eye(),
        }
    }
}

impl Pattern for TestPattern {
    fn color_at(&self, point: &Vec4) -> Color {
        Color::new(point.x, point.y, point.z)
    }
    fn pattern_at(&self, _object: &dyn Shape, world_point: &Vec4) -> Color {
        self.color_at(world_point)
    }
    fn set_transformation(&mut self, matrix: Matrix<4, 4>) {
        self.transform = matrix;
    }
    fn transform(&self) -> &Matrix<4, 4> {
        return &self.transform;
    }
}
