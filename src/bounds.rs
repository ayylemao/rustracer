use std::f32::INFINITY;

use crate::{math::EPSILON, matrix::Matrix, ray::Ray, vec4::Vec4};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub min: Vec4,
    pub max: Vec4,
}

impl Bounds {
    pub fn new(min: Vec4, max: Vec4) -> Self {
        Self { min, max }
    }
    pub fn transform(&self, transformation: &Matrix<4, 4>) -> Bounds {
        let new_min = transformation * &self.min;
        let new_max = transformation * &self.max;
        Bounds {
            min: new_min,
            max: new_max,
        }
    }
    pub fn is_finite(&self) -> bool {
        self.min.x.is_finite()
            && self.min.y.is_finite()
            && self.min.z.is_finite()
            && self.max.x.is_finite()
            && self.max.y.is_finite()
            && self.max.z.is_finite()
    }

    pub fn intersection<'a>(&'a self, ray: &Ray) -> bool {
        let (xtmin, xtmax) =
            Bounds::check_axis(self.min.x, self.max.x, ray.origin.x, ray.direction.x);
        let (ytmin, ytmax) =
            Bounds::check_axis(self.min.y, self.max.y, ray.origin.y, ray.direction.y);
        let (ztmin, ztmax) =
            Bounds::check_axis(self.min.z, self.max.z, ray.origin.z, ray.direction.z);

        let tmin = xtmin.max(ytmin).max(ztmin);
        let tmax = xtmax.min(ytmax).min(ztmax);

        if tmin > tmax {
            return false;
        }
        true
    }

    pub fn check_axis(min_axis: f32, max_axis: f32, origin: f32, direction: f32) -> (f32, f32) {
        let tmin_numerator = min_axis - origin;
        let tmax_numerator = max_axis - origin;

        let (mut tmin, mut tmax) = if direction.abs() >= EPSILON {
            (tmin_numerator / direction, tmax_numerator / direction)
        } else {
            (tmin_numerator * INFINITY, tmax_numerator * INFINITY)
        };

        if tmin > tmax {
            std::mem::swap(&mut tmin, &mut tmax);
        }
        (tmin, tmax)
    }
}

#[cfg(test)]

pub mod tests {
    use std::sync::Arc;

    use crate::{
        Sphere,
        bounds::Bounds,
        matrix::Matrix,
        obj_parser::Parser,
        shapes::{Shape, group::Group},
        vec4::Vec4,
    };

    #[test]
    fn test_bounding_box_simple() {
        let mut g = Group::new();
        let s2 = Sphere::with_transformation(Matrix::scaling(5.0, 1.0, 1.0));
        g.add_child(Arc::new(s2));

        assert_eq!(
            g.bounds(),
            Bounds::new(Vec4::point(-5.0, -1.0, -1.0), Vec4::point(5.0, 1.0, 1.0))
        );

        let mut g2 = Group::new();
        g2.add_child(Arc::new(g));
        g2.set_transformation(Matrix::scaling(5.0, 1.0, 1.0));
        assert_eq!(
            g2.bounds(),
            Bounds::new(Vec4::point(-25.0, -1.0, -1.0), Vec4::point(25.0, 1.0, 1.0))
        );
    }

    #[test]
    fn test_teapot() {
        let mut p = Parser::new();
        let mut g = p.parse_file("objects/teapot.obj");
        assert_eq!(
            g.bounds(),
            Bounds::new(
                Vec4::point(-15.0, -10.0, -0.0),
                Vec4::point(17.17, 10.0, 15.75)
            )
        );

        g.set_transformation(Matrix::scaling(5.0, 1.0, 1.0));

        assert_eq!(
            g.bounds(),
            Bounds::new(
                Vec4::point(-15.0 * 5.0, -10.0, -0.0),
                Vec4::point(17.17 * 5.0, 10.0, 15.75)
            )
        );
    }
}
