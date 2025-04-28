use crate::{color::Color, matrix::Matrix};

use super::Pattern;

#[derive(Debug, Clone)]
pub struct Checker {
    pub colors: [Color; 2],
    pub transform: Matrix<4, 4>,
}

impl Checker {
    pub fn new(a: Color, b: Color) -> Self {
        Checker {
            colors: [a, b],
            transform: Matrix::eye(),
        }
    }
}

impl Pattern for Checker {
    fn color_at(&self, point: &crate::vec4::Vec4) -> Color {
        let sum = point.x.round() as i32 + point.y.round() as i32 + point.z.round() as i32;
        if sum % 2 == 0 {
            return self.colors[0];
        }
        self.colors[1]
    }

    fn set_transformation(&mut self, matrix: Matrix<4, 4>) {
        self.transform = matrix;
    }

    fn transform(&self) -> &Matrix<4, 4> {
        &self.transform
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{color::Color, patterns::Pattern, vec4::Vec4};

    use super::Checker;

    #[test]
    fn checker() {
        let p = Checker::new(Color::white(), Color::black());
        assert_eq!(p.color_at(&Vec4::point(0.0, 0.0, 0.0)), Color::white());
        assert_eq!(p.color_at(&Vec4::point(0.99, 0.0, 0.0)), Color::white());
        assert_eq!(p.color_at(&Vec4::point(1.01, 0.0, 0.0)), Color::black());
    }
}
