pub const EPSILON: f64 = 0.0001;

pub trait ApproxEq {
    fn approx_eq(&self, other: &Self) -> bool;
}

impl ApproxEq for f64 {
    fn approx_eq(&self, other: &Self) -> bool {
        (self - other).abs() < EPSILON
    }
}
