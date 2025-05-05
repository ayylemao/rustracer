use std::f64::INFINITY;

use super::{Shape, next_shape_id};
use crate::bounds::Bounds;
use crate::intersection::Intersection;
use crate::material::Material;
use crate::math::EPSILON;
use crate::matrix::{Matrix, SqMatrix};
use crate::ray::Ray;
use crate::vec4::Vec4;

#[derive(Debug)]
pub struct Cube {
    pub id: usize,
    pub transform: SqMatrix<4>,
    pub material: Material,
    pub inverse: SqMatrix<4>,
    pub bounds: Bounds,
}

impl Cube {
    pub fn new() -> Cube {
        let id = next_shape_id();

        Cube {
            id,
            transform: Matrix::eye(),
            material: Material::default(),
            inverse: Matrix::eye(),
            bounds: Bounds::new(Vec4::point(-1.0, -1.0, -1.0), Vec4::point(1.0, 1.0, 1.0)),
        }
    }
    pub fn with_transformation(mat: Matrix<4, 4>) -> Self {
        let id = next_shape_id();
        Cube {
            id,
            transform: mat.clone(),
            material: Material::default(),
            inverse: mat.inverse(),
            bounds: Bounds::new(Vec4::point(-1.0, -1.0, -1.0), Vec4::point(1.0, 1.0, 1.0)),
        }
    }

    pub fn check_axis(origin: f64, direction: f64) -> (f64, f64) {
        let tmin_numerator = -1.0 - origin;
        let tmax_numerator = 1.0 - origin;

        let (mut tmin, mut tmax) = if direction.abs() >= EPSILON {
            (tmin_numerator / direction, tmax_numerator / direction)
        } else {
            (tmin_numerator * INFINITY, tmax_numerator * INFINITY)
        };

        if tmin > tmax {
            std::mem::swap(&mut tmin, &mut tmax);
        }
        (tmin, tmax)
    }
}

impl Shape for Cube {
    fn local_intersect<'a>(&'a self, ray: &Ray) -> Vec<Intersection<'a>> {
        let (xtmin, xtmax) = Cube::check_axis(ray.origin.x, ray.direction.x);
        let (ytmin, ytmax) = Cube::check_axis(ray.origin.y, ray.direction.y);
        let (ztmin, ztmax) = Cube::check_axis(ray.origin.z, ray.direction.z);

        let tmin = xtmin.max(ytmin).max(ztmin);
        let tmax = xtmax.min(ytmax).min(ztmax);

        if tmin > tmax {
            return vec![];
        }

        vec![
            Intersection::new(tmin, self, None, None),
            Intersection::new(tmax, self, None, None),
        ]
    }

    fn local_normal_at(&self, local_point: Vec4, _i: &Intersection) -> Vec4 {
        let abs_x = local_point.x.abs();
        let abs_y = local_point.y.abs();
        let abs_z = local_point.z.abs();
        let maxc = abs_x.max(abs_y).max(abs_z);

        if maxc == abs_x {
            Vec4::vector(local_point.x.signum(), 0.0, 0.0)
        } else if maxc == abs_y {
            Vec4::vector(0.0, local_point.y.signum(), 0.0)
        } else {
            Vec4::vector(0.0, 0.0, local_point.z.signum())
        }
    }

    fn transform(&self) -> &SqMatrix<4> {
        &self.transform
    }

    fn material(&self) -> &Material {
        &self.material
    }

    fn set_transformation(&mut self, mat: Matrix<4, 4>) {
        self.transform = mat.clone();
        self.inverse = mat.inverse();
    }

    fn set_material(&mut self, material: Material) {
        self.material = material;
    }

    fn id(&self) -> usize {
        self.id
    }

    fn inverse(&self) -> &Matrix<4, 4> {
        &self.inverse
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn bounds(&self) -> crate::bounds::Bounds {
        self.bounds
    }

    fn as_any_mut(&mut self) ->  &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
pub mod tests {
    //use std::f64::consts::PI;

    use std::sync::Arc;

    use crate::Sphere;

    use super::*;

    #[test]
    fn test_ray_cube_intersections() {
        use crate::ray::Ray;
        use crate::vec4::Vec4;

        struct TestCase {
            origin: Vec4,
            direction: Vec4,
            t1: f64,
            t2: f64,
        }

        let cases = vec![
            TestCase {
                origin: Vec4::point(5.0, 0.5, 0.0),
                direction: Vec4::vector(-1.0, 0.0, 0.0),
                t1: 4.0,
                t2: 6.0,
            },
            TestCase {
                origin: Vec4::point(-5.0, 0.5, 0.0),
                direction: Vec4::vector(1.0, 0.0, 0.0),
                t1: 4.0,
                t2: 6.0,
            },
            TestCase {
                origin: Vec4::point(0.5, 5.0, 0.0),
                direction: Vec4::vector(0.0, -1.0, 0.0),
                t1: 4.0,
                t2: 6.0,
            },
            TestCase {
                origin: Vec4::point(0.5, -5.0, 0.0),
                direction: Vec4::vector(0.0, 1.0, 0.0),
                t1: 4.0,
                t2: 6.0,
            },
            TestCase {
                origin: Vec4::point(0.5, 0.0, 5.0),
                direction: Vec4::vector(0.0, 0.0, -1.0),
                t1: 4.0,
                t2: 6.0,
            },
            TestCase {
                origin: Vec4::point(0.5, 0.0, -5.0),
                direction: Vec4::vector(0.0, 0.0, 1.0),
                t1: 4.0,
                t2: 6.0,
            },
            TestCase {
                origin: Vec4::point(0.0, 0.5, 0.0),
                direction: Vec4::vector(0.0, 0.0, 1.0),
                t1: -1.0,
                t2: 1.0,
            },
        ];

        for (i, case) in cases.into_iter().enumerate() {
            let cube = Cube::new();
            let ray = Ray::from_vec4(case.origin, case.direction);
            let xs = cube.local_intersect(&ray);
            assert_eq!(xs.len(), 2, "Case {} failed: expected 2 intersections", i);
            assert!(
                (xs[0].t - case.t1).abs() < 1e-5,
                "Case {} failed: xs[0].t = {}, expected {}",
                i,
                xs[0].t,
                case.t1
            );
            assert!(
                (xs[1].t - case.t2).abs() < 1e-5,
                "Case {} failed: xs[1].t = {}, expected {}",
                i,
                xs[1].t,
                case.t2
            );
        }
    }
    #[test]
    fn test_ray_misses_cube() {
        use crate::ray::Ray;
        use crate::vec4::Vec4;

        struct TestCase {
            origin: Vec4,
            direction: Vec4,
        }

        let cases = vec![
            TestCase {
                origin: Vec4::point(-2.0, 0.0, 0.0),
                direction: Vec4::vector(0.2673, 0.5345, 0.8018),
            },
            TestCase {
                origin: Vec4::point(0.0, -2.0, 0.0),
                direction: Vec4::vector(0.8018, 0.2673, 0.5345),
            },
            TestCase {
                origin: Vec4::point(0.0, 0.0, -2.0),
                direction: Vec4::vector(0.5345, 0.8018, 0.2673),
            },
            TestCase {
                origin: Vec4::point(2.0, 0.0, 2.0),
                direction: Vec4::vector(0.0, 0.0, -1.0),
            },
            TestCase {
                origin: Vec4::point(0.0, 2.0, 2.0),
                direction: Vec4::vector(0.0, -1.0, 0.0),
            },
            TestCase {
                origin: Vec4::point(2.0, 2.0, 0.0),
                direction: Vec4::vector(-1.0, 0.0, 0.0),
            },
        ];

        for (i, case) in cases.into_iter().enumerate() {
            let cube = Cube::new();
            let ray = Ray::from_vec4(case.origin, case.direction);
            let xs = cube.local_intersect(&ray);
            assert_eq!(
                xs.len(),
                0,
                "Ray miss test case {} failed: expected 0 intersections, got {}",
                i,
                xs.len()
            );
        }
    }
    #[test]
    fn test_cube_normals() {
        use crate::vec4::Vec4;

        struct TestCase {
            point: Vec4,
            expected_normal: Vec4,
        }

        let cases = vec![
            TestCase {
                point: Vec4::point(1.0, 0.5, -0.8),
                expected_normal: Vec4::vector(1.0, 0.0, 0.0),
            },
            TestCase {
                point: Vec4::point(-1.0, -0.2, 0.9),
                expected_normal: Vec4::vector(-1.0, 0.0, 0.0),
            },
            TestCase {
                point: Vec4::point(-0.4, 1.0, -0.1),
                expected_normal: Vec4::vector(0.0, 1.0, 0.0),
            },
            TestCase {
                point: Vec4::point(0.3, -1.0, -0.7),
                expected_normal: Vec4::vector(0.0, -1.0, 0.0),
            },
            TestCase {
                point: Vec4::point(-0.6, 0.3, 1.0),
                expected_normal: Vec4::vector(0.0, 0.0, 1.0),
            },
            TestCase {
                point: Vec4::point(0.4, 0.4, -1.0),
                expected_normal: Vec4::vector(0.0, 0.0, -1.0),
            },
            TestCase {
                point: Vec4::point(1.0, 1.0, 1.0),
                expected_normal: Vec4::vector(1.0, 0.0, 0.0),
            },
            TestCase {
                point: Vec4::point(-1.0, -1.0, -1.0),
                expected_normal: Vec4::vector(-1.0, 0.0, 0.0),
            },
        ];

        let cube = Cube::new();
        let dummy: Arc<dyn Shape + Send + Sync> = Arc::new(Sphere::new());

        for (i, case) in cases.into_iter().enumerate() {
            let normal = cube.local_normal_at(
                case.point,
                &Intersection::new(0.0, dummy.as_ref(), None, None),
            );
            assert_eq!(
                normal, case.expected_normal,
                "Failed case {}: expected {:?}, got {:?}",
                i, case.expected_normal, normal
            );
        }
    }
}
