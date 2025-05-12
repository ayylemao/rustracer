pub const EPSILON: f32 = 0.0001;

pub trait ApproxEq {
    fn approx_eq(&self, other: &Self) -> bool;
}

impl ApproxEq for f32 {
    fn approx_eq(&self, other: &Self) -> bool {
        (self - other).abs() < EPSILON
    }
}
