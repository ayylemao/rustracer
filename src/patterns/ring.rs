use crate::{color::Color, matrix::Matrix};

use super::Pattern;

#[derive(Debug, Clone)]
pub struct Ring {
    pub colors: [Color; 2],
    pub transform: Matrix<4, 4>,
}

impl Ring {
    pub fn new(a: Color, b: Color) -> Self {
        Ring {
            colors: [a, b],
            transform: Matrix::eye(),
        }
    }
}

impl Pattern for Ring {
    fn color_at(&self, point: &crate::vec4::Vec4) -> Color {
        if (point.x.powi(2) + point.z.powi(2)).sqrt().floor() % 2.0 == 0.0 {
            return self.colors[0];
        } else {
            return self.colors[1];
        };
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

    use super::Ring;

    #[test]
    fn grad() {
        let p = Ring::new(Color::white(), Color::black());
        assert_eq!(p.color_at(&Vec4::point(0.0, 0.0, 0.0)), Color::white());
        assert_eq!(
            p.color_at(&Vec4::point(1.0, 0.0, 0.0)),
            Color::black()
        );
        assert_eq!(
            p.color_at(&Vec4::point(0.708, 0.0, 0.708)),
            Color::black()
        );
    }
}
