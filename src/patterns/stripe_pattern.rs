use super::Pattern;
use crate::matrix::Matrix;
use crate::{color::Color, vec4::Vec4};
use std::clone::Clone;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct StripePattern {
    pub colors: [Color; 2],
    pub transform: Matrix<4, 4>,
}

impl StripePattern {
    pub fn new(a: Color, b: Color) -> Self {
        StripePattern {
            colors: [a, b],
            transform: Matrix::eye(),
        }
    }
}

impl Pattern for StripePattern {
    fn color_at(&self, point: &Vec4) -> Color {
        if point.x.floor() % 2.0 == 0.0 {
            return self.colors[0];
        } else {
            return self.colors[1];
        }
    }
    fn set_transformation(&mut self, matrix: Matrix<4, 4>) {
        self.transform = matrix;
    }
    fn transform(&self) -> &Matrix<4, 4> {
        return &self.transform;
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{color::Color, patterns::Pattern, vec4::Vec4};

    use super::StripePattern;

    #[test]
    fn stripe_pattern() {
        let pat = StripePattern::new(Color::white(), Color::black());
        assert_eq!(pat.color_at(&Vec4::point(0.0, 0.0, 0.0)), Color::white());
        assert_eq!(pat.color_at(&Vec4::point(0.0, 0.0, 1.0)), Color::white());
        assert_eq!(pat.color_at(&Vec4::point(0.0, 0.0, 2.0)), Color::white());
        assert_eq!(pat.color_at(&Vec4::point(1.0, 0.0, 0.0)), Color::black());
        assert_eq!(pat.color_at(&Vec4::point(-0.1, 0.0, 0.0)), Color::black());
        assert_eq!(pat.color_at(&Vec4::point(-1.0, 0.0, 0.0)), Color::black());
    }
}
