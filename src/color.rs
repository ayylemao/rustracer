use super::math::ApproxEq;
use num_traits::ToPrimitive;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Color {
    pub fn new(r: f64, g: f64, b: f64) -> Color {
        Self { r, g, b }
    }
    pub fn default() -> Color {
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        }
    }
    pub fn to_rgb_u8(self) -> (u8, u8, u8) {
        (
            (self.r.clamp(0.0, 1.0) * 255.0f64).round() as u8,
            (self.g.clamp(0.0, 1.0) * 255.0f64).round() as u8,
            (self.b.clamp(0.0, 1.0) * 255.0f64).round() as u8,
        )
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        self.r.approx_eq(&other.r) && self.g.approx_eq(&other.g) && self.b.approx_eq(&other.b)
    }
}

impl Add for Color {
    type Output = Color;
    fn add(self, rhs: Self) -> Self::Output {
        Color::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b)
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Self) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
    }
}

impl Sub for Color {
    type Output = Color;
    fn sub(self, rhs: Self) -> Self::Output {
        Color::new(self.r - rhs.r, self.g - rhs.g, self.b - rhs.b)
    }
}

impl SubAssign for Color {
    fn sub_assign(&mut self, rhs: Self) {
        self.r -= rhs.r;
        self.g -= rhs.g;
        self.b -= rhs.b;
    }
}

impl Mul for Color {
    type Output = Color;
    fn mul(self, rhs: Self) -> Self::Output {
        Color::new(self.r * rhs.r, self.g * rhs.g, self.b * rhs.b)
    }
}

impl MulAssign for Color {
    fn mul_assign(&mut self, rhs: Self) {
        self.r *= rhs.r;
        self.g *= rhs.g;
        self.b *= rhs.b;
    }
}

impl<I> Mul<I> for Color
where
    I: ToPrimitive + Copy,
{
    type Output = Color;
    fn mul(self, rhs: I) -> Self::Output {
        let scalar = rhs.to_f64().expect("Failed to convert to f64");
        let result = Color {
            r: self.r * scalar,
            g: self.g * scalar,
            b: self.b * scalar,
        };
        result
    }
}
impl<I> MulAssign<I> for Color
where
    I: ToPrimitive + Copy,
{
    fn mul_assign(&mut self, rhs: I) {
        let scalar = rhs.to_f64().expect("Failed to convert to f64");
        self.r *= scalar;
        self.g *= scalar;
        self.b *= scalar;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_colors() {
        let c1 = Color {
            r: 0.9,
            g: 0.6,
            b: 0.75,
        };
        let c2 = Color {
            r: 0.7,
            g: 0.1,
            b: 0.25,
        };
        let expected = Color {
            r: 1.6,
            g: 0.7,
            b: 1.0,
        };

        assert_eq!(c1 + c2, expected);
    }

    #[test]
    fn subtract_colors() {
        let c1 = Color {
            r: 0.9,
            g: 0.6,
            b: 0.75,
        };
        let c2 = Color {
            r: 0.7,
            g: 0.1,
            b: 0.25,
        };
        let expected = Color {
            r: 0.2,
            g: 0.5,
            b: 0.5,
        };

        assert_eq!(c1 - c2, expected);
    }

    #[test]
    fn multiply_color_by_scalar() {
        let c = Color {
            r: 0.2,
            g: 0.3,
            b: 0.4,
        };
        let expected = Color {
            r: 0.4,
            g: 0.6,
            b: 0.8,
        };

        assert_eq!(c * 2.0, expected);
    }

    #[test]
    fn multiply_colors_component_wise() {
        let c1 = Color {
            r: 1.0,
            g: 0.2,
            b: 0.4,
        };
        let c2 = Color {
            r: 0.9,
            g: 1.0,
            b: 0.1,
        };
        let expected = Color {
            r: 0.9,
            g: 0.2,
            b: 0.04,
        };

        assert_eq!(c1 * c2, expected);
    }
}
