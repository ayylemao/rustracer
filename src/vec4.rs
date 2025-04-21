use num_traits::ToPrimitive;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::math::ApproxEq;

#[derive(Debug, Clone, Copy)]
pub struct Vec4 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl PartialEq for Vec4 {
    fn eq(&self, other: &Self) -> bool {
        self.x.approx_eq(&other.x)
            && self.y.approx_eq(&other.y)
            && self.z.approx_eq(&other.z)
            && self.w.approx_eq(&other.w)
    }
}

impl Add for Vec4 {
    type Output = Vec4;
    fn add(self, rhs: Self) -> Self::Output {
        let result = Vec4 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        };
        debug_assert!(result.w < 2.0, "Cannot add Point to Point!");
        result
    }
}

impl AddAssign for Vec4 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self.w += rhs.w;
        debug_assert!(self.w < 2.0, "Cannot add Point to Point!");
    }
}

impl Sub for Vec4 {
    type Output = Vec4;
    fn sub(self, rhs: Self) -> Self::Output {
        debug_assert!(
            !(self.w.approx_eq(&0.0) && rhs.w.approx_eq(&1.0)),
            "Cannot subtract a Point from a Vector!"
        );
        let result = Vec4 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: self.w - rhs.w,
        };
        result
    }
}

impl SubAssign for Vec4 {
    fn sub_assign(&mut self, rhs: Self) {
        debug_assert!(
            !(self.w.approx_eq(&0.0) && rhs.w.approx_eq(&1.0)),
            "Cannot subtract a Point from a Vector!"
        );
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
        self.w -= rhs.w;
    }
}

impl Neg for Vec4 {
    type Output = Vec4;
    fn neg(self) -> Self::Output {
        let result = Vec4 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        };
        result
    }
}

impl<I> Mul<I> for Vec4
where
    I: ToPrimitive + Copy,
{
    type Output = Vec4;
    fn mul(self, rhs: I) -> Self::Output {
        let scalar = rhs.to_f64().expect("Failed to convert to f64");
        let result = Vec4 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
            w: self.w * scalar,
        };
        result
    }
}
impl<I> MulAssign<I> for Vec4
where
    I: ToPrimitive + Copy,
{
    fn mul_assign(&mut self, rhs: I) {
        let scalar = rhs.to_f64().expect("Failed to convert to f64");
        self.x *= scalar;
        self.y *= scalar;
        self.z *= scalar;
        self.w *= scalar;
    }
}

impl<I> Div<I> for Vec4
where
    I: ToPrimitive + Copy,
{
    type Output = Vec4;
    fn div(self, rhs: I) -> Self::Output {
        let scalar = rhs.to_f64().expect("Failed to convert to f64");
        let result = Vec4 {
            x: self.x / scalar,
            y: self.y / scalar,
            z: self.z / scalar,
            w: self.w / scalar,
        };
        result
    }
}

impl<I> DivAssign<I> for Vec4
where
    I: ToPrimitive + Copy,
{
    fn div_assign(&mut self, rhs: I) {
        let scalar = rhs.to_f64().expect("Failed to convert to f64");
        self.x /= scalar;
        self.y /= scalar;
        self.z /= scalar;
        self.w /= scalar;
    }
}

impl Vec4 {
    pub fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w: w }
    }
    pub fn from_array(array: [f64; 4]) -> Self {
        Self {
            x: array[0],
            y: array[1],
            z: array[2],
            w: array[3],
        }
    }
    pub fn point(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z, w: 1.0 }
    }
    pub fn vector(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z, w: 0.0 }
    }
    pub fn magnitude(self) -> f64 {
        (self.x.powf(2.0) + self.y.powf(2.0) + self.z.powf(2.0) + self.w.powf(2.0)).sqrt()
    }
    pub fn norm(self) -> Vec4 {
        self / self.magnitude()
    }
    pub fn norm_mut(&mut self) {
        *self /= self.magnitude();
    }
    pub fn dot(self, rhs: &Vec4) -> f64 {
        debug_assert!(
            (self.w == 0.0 && rhs.w == 0.0),
            "Cannot take dot product of non-vectors!"
        );
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z + self.w * rhs.w
    }
    pub fn cross(self, rhs: &Vec4) -> Vec4 {
        debug_assert!(
            (self.w == 0.0 && rhs.w == 0.0),
            "Cannot take cross product of non-vectors!"
        );
        Vec4 {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
            w: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_is_point_vector_is_vector() {
        // point
        let t = Vec4::point(4.3, -4.2, 3.1);
        assert_eq!(t.w, 1.0);
        let t = Vec4::vector(4.3, -4.2, 3.1);
        assert_eq!(t.w, 0.0);
    }

    #[test]
    fn tuple_add() {
        let t1 = Vec4::new(3.0, -2.0, 5.0, 1.0);
        let t2 = Vec4::new(-2.0, 3.0, 1.0, 0.0);
        assert_eq!(t1 + t2, Vec4::new(1.0, 1.0, 6.0, 1.0));
    }
    #[test]
    #[should_panic(expected = "Cannot add Point to Point")]
    fn tuple_add_point_point() {
        let t1 = Vec4::new(3.0, -2.0, 5.0, 1.0);
        let t2 = Vec4::new(-2.0, 3.0, 1.0, 1.0);
        assert_eq!(t1 + t2, Vec4::new(1.0, 1.0, 6.0, 1.0));
    }
    #[test]
    fn point_point_sub() {
        let t1 = Vec4::point(3.0, 2.0, 1.0);
        let t2 = Vec4::point(5.0, 6.0, 7.0);
        assert_eq!(t1 - t2, Vec4::vector(-2.0, -4.0, -6.0));
    }
    #[test]
    fn vector_point_sub() {
        let t1 = Vec4::point(3.0, 2.0, 1.0);
        let t2 = Vec4::vector(5.0, 6.0, 7.0);
        assert_eq!(t1 - t2, Vec4::point(-2.0, -4.0, -6.0));
    }
    #[test]
    fn vector_vector_sub() {
        let t1 = Vec4::vector(3.0, 2.0, 1.0);
        let t2 = Vec4::vector(5.0, 6.0, 7.0);
        assert_eq!(t1 - t2, Vec4::vector(-2.0, -4.0, -6.0));
    }
    #[test]
    #[should_panic]
    fn point_vector_sub() {
        let t1 = Vec4::point(3.0, 2.0, 1.0);
        let t2 = Vec4::vector(5.0, 6.0, 7.0);
        assert_eq!(t1 - t2, Vec4::vector(-2.0, -4.0, -6.0));
    }
    #[test]
    fn negation() {
        let t1 = Vec4::new(3.0, 2.0, 1.0, -4.0);
        assert_eq!(-t1, Vec4::new(-3.0, -2.0, -1.0, 4.0));
    }
    #[test]
    fn scalar_mul() {
        let t1 = Vec4::new(3.0, 2.0, 1.0, -4.0);
        assert_eq!(t1 * 2, Vec4::new(6.0, 4.0, 2.0, -8.0));
        assert_eq!(t1 * 0.5, Vec4::new(1.5, 1.0, 0.5, -2.0));
        assert_eq!(t1 / 0.5, Vec4::new(6.0, 4.0, 2.0, -8.0));
    }
    #[test]
    fn magnitude() {
        let t1 = Vec4::vector(0.0, 1.0, 0.0);
        assert_eq!(t1.magnitude(), 1.0);
        let t1 = Vec4::vector(1.0, 2.0, 3.0);
        assert_eq!(t1.magnitude(), 14.0f64.sqrt());
        let t1 = Vec4::vector(-1.0, -2.0, -3.0);
        assert_eq!(t1.magnitude(), 14.0f64.sqrt());
    }
    #[test]
    fn norm() {
        let t1 = Vec4::vector(5.0, 0.0, 0.0);
        assert_eq!(t1.norm(), Vec4::vector(1.0, 0.0, 0.0));
        let mut t1 = Vec4::vector(1.0, 2.0, 3.0);
        t1.norm_mut();
        assert_eq!(t1, Vec4::vector(0.26726, 0.53452, 0.80178))
    }
    #[test]
    fn dot() {
        let t1 = Vec4::vector(1.0, 2.0, 3.0);
        let t2 = Vec4::vector(2.0, 3.0, 4.0);
        assert_eq!(t1.dot(&t2), 20.0);
    }
    #[test]
    #[should_panic]
    fn dot_panic() {
        let t1 = Vec4::point(1.0, 2.0, 3.0);
        let t2 = Vec4::vector(2.0, 3.0, 4.0);
        assert_eq!(t1.dot(&t2), 20.0);
    }
    #[test]
    fn cross() {
        let t1 = Vec4::vector(1.0, 2.0, 3.0);
        let t2 = Vec4::vector(2.0, 3.0, 4.0);
        assert_eq!(t1.cross(&t2), Vec4::vector(-1.0, 2.0, -1.0));
        assert_eq!(t2.cross(&t1), Vec4::vector(1.0, -2.0, 1.0));
    }
}
