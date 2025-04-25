use crate::math::ApproxEq;
use std::cmp::Ordering;
use crate::shape::Shape;

#[derive(Debug, Clone, Copy)]
pub struct Intersection<'a> {
    pub id: usize,
    pub t: f64,
    pub object: &'a dyn Shape
}

impl Intersection<'_> {
    pub fn new(t: f64, id: usize, object: &dyn Shape) -> Intersection {
        Intersection {id, t, object }
    }

    pub fn hit<'a>(int_list: &'a [Intersection<'a>]) -> Option<&'a Intersection<'a>> {
        int_list
            .iter()
            .filter(|i| i.t >= 0.0)
            .min_by(|a, b| a.t.partial_cmp(&b.t).unwrap())
    }
}
impl PartialEq for Intersection<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.t.approx_eq(&other.t) && self.id == other.id
    }
}

impl PartialOrd for Intersection<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.t.partial_cmp(&other.t)
    }
}

impl Eq for Intersection<'_> {}

impl Ord for Intersection<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap() // be sure `t` isn't NaN
    }
}

#[cfg(test)]
pub mod tests {
    use crate::shape::Sphere;

    use super::*;

    #[test]
    fn get_hit() {
        let sphere = Sphere::new();
        let i1 = Intersection::new(1.0, 1, &sphere);
        let i2 = Intersection::new(2.0, 1, &sphere);
        let xs = vec![i1, i2];
        let i = Intersection::hit(&xs);
        assert_eq!(i1, *i.unwrap());

        let i1 = Intersection::new(-1.0, 1, &sphere);
        let i2 = Intersection::new(1.0, 1, &sphere);
        let xs = vec![i1, i2];
        let i = Intersection::hit(&xs);
        assert_eq!(i2, *i.unwrap());
        let i1 = Intersection::new(-1.0, 1, &sphere);
        let i2 = Intersection::new(-1.0, 1, &sphere);
        let xs = vec![i1, i2];
        let i = Intersection::hit(&xs);
        assert_eq!(None, i);

        let i1 = Intersection::new(5.0, 1, &sphere);
        let i2 = Intersection::new(7.0, 1, &sphere);
        let i3 = Intersection::new(-3.0, 1, &sphere);
        let i4 = Intersection::new(2.0, 1, &sphere);
        let xs = vec![i1, i2, i3, i4];
        let i = Intersection::hit(&xs);
        assert_eq!(*i.unwrap(), i4);
    }
}
