use crate::math::ApproxEq;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy)]
pub struct Intersection {
    pub t: f64,
    pub id: usize,
}

impl Intersection {
    pub fn new(t: f64, id: usize) -> Intersection {
        Intersection { t, id }
    }

    pub fn hit(int_list: &Vec<Intersection>) -> Option<Intersection> {
        int_list
            .iter()
            .filter(|i| i.t >= 0.0)
            .min_by(|a, b| a.t.partial_cmp(&b.t).unwrap())
            .copied()
    }
}
impl PartialEq for Intersection {
    fn eq(&self, other: &Self) -> bool {
        self.t.approx_eq(&other.t)
    }
}

impl PartialOrd for Intersection {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.t.partial_cmp(&other.t)
    }
}

impl Eq for Intersection {}

impl Ord for Intersection {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap() // be sure `t` isn't NaN
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn get_hit() {
        let i1 = Intersection::new(1.0, 1);
        let i2 = Intersection::new(2.0, 1);
        let xs = vec![i1, i2];
        let i = Intersection::hit(&xs);
        assert_eq!(i1, i.unwrap());

        let i1 = Intersection::new(-1.0, 1);
        let i2 = Intersection::new(1.0, 1);
        let xs = vec![i1, i2];
        let i = Intersection::hit(&xs);
        assert_eq!(i2, i.unwrap());
        let i1 = Intersection::new(-1.0, 1);
        let i2 = Intersection::new(-1.0, 1);
        let xs = vec![i1, i2];
        let i = Intersection::hit(&xs);
        assert_eq!(None, i);

        let i1 = Intersection::new(5.0, 1);
        let i2 = Intersection::new(7.0, 1);
        let i3 = Intersection::new(-3.0, 1);
        let i4 = Intersection::new(2.0, 1);
        let xs = vec![i1, i2, i3, i4];
        let i = Intersection::hit(&xs);
        assert_eq!(i.unwrap(), i4);
    }
}
