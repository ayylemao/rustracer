use crate::{matrix::Matrix, vec4::Vec4};

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
}

#[cfg(test)]

pub mod tests {
    use std::sync::Arc;

    use crate::{
        bounds::Bounds, matrix::Matrix, obj_parser::Parser, shapes::{group::Group, Shape}, vec4::Vec4, Sphere
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
