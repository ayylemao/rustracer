use crate::{color::Color, vec4::Vec4};
use std::fmt::Debug;
use std::clone::Clone;



pub trait Pattern : Debug + Clone {
    fn color_at(&self, point: &Vec4) -> Color;
}

#[derive(Debug, Clone)]
pub struct StripePattern {
    pub colors: Vec<Color>
}

impl StripePattern {
    pub fn new(a: Color, b: Color) -> Self {
        let mut cs: Vec<Color> = Vec::new();
        cs.push(a);
        cs.push(b);
        StripePattern { colors: cs }
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