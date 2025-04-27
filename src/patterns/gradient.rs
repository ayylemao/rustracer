use crate::{color::Color, matrix::Matrix};

use super::Pattern;

#[derive(Debug, Clone)]
pub struct Gradient {
    pub colors: [Color; 2],
    pub transform: Matrix<4, 4>,
}

impl Gradient {
    pub fn new(a: Color, b: Color) -> Self {
        Gradient {
            colors: [a, b],
            transform: Matrix::eye(),
        }
    }
}

impl Pattern for Gradient {
    fn color_at(&self, point: &crate::vec4::Vec4) -> Color {
        let dist = self.colors[1] - self.colors[0];
        let fraction = point.x - point.x.floor();
        self.colors[0] + dist * fraction
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

    use super::Gradient;

    #[test]
    fn grad() {
        let p = Gradient::new(Color::white(), Color::black());
        assert_eq!(p.color_at(&Vec4::point(0.0, 0.0, 0.0)), Color::white());
        assert_eq!(
            p.color_at(&Vec4::point(0.25, 0.0, 0.0)),
            Color::new(0.75, 0.75, 0.75)
        );
    }
}
