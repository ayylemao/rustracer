use std::ops::Neg;

use crate::{matrix::SqMatrix, vec4::Vec4};

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vec4,
    pub direction: Vec4,
}

impl PartialEq for Ray {
    fn eq(&self, other: &Self) -> bool {
        self.origin == other.origin && self.direction == other.direction
    }
}

impl Ray {
    pub fn new(x: f32, y: f32, z: f32, dx: f32, dy: f32, dz: f32) -> Self {
        Ray {
            origin: Vec4::point(x, y, z),
            direction: Vec4::vector(dx, dy, dz),
        }
    }
    pub fn from_vec4(origin: Vec4, direction: Vec4) -> Self {
        Ray { origin, direction }
    }
    pub fn position(&self, t: f32) -> Vec4 {
        self.origin + self.direction * t
    }
    pub fn transform(&self, mat: &SqMatrix<4>) -> Ray {
        let new_origin = mat * &self.origin;
        let new_dir = mat * &self.direction;
        Ray {
            origin: new_origin,
            direction: new_dir,
        }
    }
    pub fn neg(&self) -> Ray {
        Self {
            origin: self.origin.clone(),
            direction: self.direction.neg(),
        }
    }
    pub fn neg_mut(&mut self) {
        self.direction = self.direction.neg();
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::matrix::Matrix;

    #[test]
    fn compute_point_distance() {
        let r = Ray::new(2.0, 3.0, 4.0, 1.0, 0.0, 0.0);
        assert_eq!(r.position(0.0), Vec4::point(2.0, 3.0, 4.0));
        assert_eq!(r.position(1.0), Vec4::point(3.0, 3.0, 4.0));
    }
    #[test]
    fn translate_ray() {
        let r = Ray::new(1.0, 2.0, 3.0, 0.0, 1.0, 0.0);
        let m = Matrix::translation(3.0, 4.0, 5.0);
        let r2 = r.transform(&m);
        assert!(
            r2.origin == Vec4::point(4.0, 6.0, 8.0) && r2.direction == Vec4::vector(0.0, 1.0, 0.0)
        );
    }
    #[test]
    fn scale_ray() {
        let r = Ray::new(1.0, 2.0, 3.0, 0.0, 1.0, 0.0);
        let m = Matrix::scaling(2.0, 3.0, 4.0);
        let r2 = r.transform(&m);
        assert_eq!(r2.origin, Vec4::point(2.0, 6.0, 12.0));
        assert_eq!(r2.direction, Vec4::vector(0.0, 3.0, 0.0));
    }
}
