use crate::math::{ApproxEq, EPSILON};
use crate::ray::Ray;
use crate::shapes::Shape;
use crate::vec4::Vec4;
use std::cmp::Ordering;

pub struct Computations<'a> {
    pub object: &'a dyn Shape,
    pub point: Vec4,
    pub eyev: Vec4,
    pub normalv: Vec4,
    pub inside: bool,
    pub over_point: Vec4,
    pub reflectv: Vec4,
    pub n1: f64,
    pub n2: f64,
    pub under_point: Vec4,
}
impl<'a> Computations<'a> {
    pub fn new(
        object: &'a dyn Shape,
        point: Vec4,
        eyev: Vec4,
        normalv: Vec4,
        raydir: Vec4,
        n1: f64,
        n2: f64,
    ) -> Self {
        let (inside, normalv) = if normalv.dot(&eyev) < 0.0 {
            (true, -normalv)
        } else {
            (false, normalv)
        };
        let reflectv = raydir.reflect(&normalv);
        let over_point = point + normalv * EPSILON;
        let under_point = point - normalv * EPSILON;
        Self {
            object,
            point,
            eyev,
            normalv,
            inside,
            over_point,
            reflectv,
            n1: n1,
            n2: n2,
            under_point,
        }
    }
    pub fn object(&self) -> &'a dyn Shape {
        self.object
    }

    pub fn schlick(&self) -> f64 {
        let mut cos = self.eyev.dot(&self.normalv);
        if self.n1 > self.n2 {
            let n = self.n1 / self.n2;
            let sin2_t = n.powi(2) * (1.0 - cos.powi(2));
            if sin2_t > 1.0 {
                return 1.0;
            }

            let cos_t = (1.0 - sin2_t).sqrt();
            cos = cos_t;
        }
        let r0 = ((self.n1 - self.n2) / (self.n1 + self.n2)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cos).powi(5)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Intersection<'a> {
    pub t: f64,
    pub object: &'a dyn Shape,
    pub u: Option<f64>,
    pub v: Option<f64>,
}

impl Intersection<'_> {
    pub fn new(t: f64, object: &dyn Shape, u: Option<f64>, v: Option<f64>) -> Intersection {
        Intersection { t, object, u, v }
    }

    pub fn hit<'a>(int_list: &'a [Intersection<'a>]) -> Option<&'a Intersection<'a>> {
        int_list
            .iter()
            .filter(|i| i.t >= 0.0)
            .min_by(|a, b| a.t.partial_cmp(&b.t).unwrap())
    }
    pub fn prepare_computations<'a>(
        &self,
        ray: &Ray,
        int_list: &'a [Intersection<'a>],
    ) -> Computations {
        let point = ray.position(self.t);
        let mut comps = Computations::new(
            self.object,
            point,
            -ray.direction,
            self.object.normal_at(point, self),
            ray.direction,
            0.0,
            0.0,
        );
        let mut container: Vec<&dyn Shape> = Vec::new();

        for i in int_list {
            if std::ptr::eq(i, self) {
                comps.n1 = container
                    .last()
                    .map_or(1.0, |s| s.material().refractive_index);
            }
            match container.iter().position(|x| x.id() == i.object.id()) {
                Some(index) => {
                    container.remove(index);
                }
                None => {
                    container.push(i.object);
                }
            }
            if std::ptr::eq(i, self) {
                comps.n2 = container
                    .last()
                    .map_or(1.0, |s| s.material().refractive_index);
                break;
            }
        }
        comps
    }
}
impl PartialEq for Intersection<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.t.approx_eq(&other.t)
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
    use std::f64::consts::SQRT_2;

    use crate::{Sphere, shapes::plane::Plane};

    use super::*;

    #[test]
    fn get_hit() {
        let sphere = Sphere::new();
        let i1 = Intersection::new(1.0, &sphere, None, None);
        let i2 = Intersection::new(2.0, &sphere, None, None);
        let xs = vec![i1, i2];
        let i = Intersection::hit(&xs);
        assert_eq!(i1, *i.unwrap());

        let i1 = Intersection::new(-1.0, &sphere, None, None);
        let i2 = Intersection::new(1.0, &sphere, None, None);
        let xs = vec![i1, i2];
        let i = Intersection::hit(&xs);
        assert_eq!(i2, *i.unwrap());
        let i1 = Intersection::new(-1.0, &sphere, None, None);
        let i2 = Intersection::new(-1.0, &sphere, None, None);
        let xs = vec![i1, i2];
        let i = Intersection::hit(&xs);
        assert_eq!(None, i);

        let i1 = Intersection::new(5.0, &sphere, None, None);
        let i2 = Intersection::new(7.0, &sphere, None, None);
        let i3 = Intersection::new(-3.0, &sphere, None, None);
        let i4 = Intersection::new(2.0, &sphere, None, None);
        let xs = vec![i1, i2, i3, i4];
        let i = Intersection::hit(&xs);
        assert_eq!(*i.unwrap(), i4);
    }

    #[test]
    pub fn hit_when_intersection_outside_inside() {
        let r = Ray::new(0.0, 0.0, -5.0, 0.0, 0.0, 1.0);
        let s = Sphere::new();
        let i = Intersection::new(4.0, &s, None, None);
        let comps = i.prepare_computations(&r, &Vec::<Intersection>::new());
        assert_eq!(comps.inside, false);

        let r = Ray::new(0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        let s = Sphere::new();
        let i = Intersection::new(1.0, &s, None, None);
        let comps = i.prepare_computations(&r, &Vec::<Intersection>::new());
        assert_eq!(comps.point, Vec4::point(0.0, 0.0, 1.0));
        assert_eq!(comps.eyev, Vec4::vector(0.0, 0.0, -1.0));
        assert_eq!(comps.inside, true);
        assert_eq!(comps.normalv, Vec4::vector(0.0, 0.0, -1.0));
    }

    #[test]
    fn precompute_reflection_vec() {
        let s = Plane::new();
        let r = Ray::from_vec4(
            Vec4::point(0.0, 1.0, -1.0),
            Vec4::vector(0.0, -SQRT_2 / 2.0, SQRT_2 / 2.0),
        );

        let i = Intersection::new(SQRT_2, &s, None, None);
        let comps = i.prepare_computations(&r, &Vec::<Intersection>::new());
        assert_eq!(
            comps.reflectv,
            Vec4::vector(0.0, SQRT_2 / 2.0, SQRT_2 / 2.0)
        );
    }
}
