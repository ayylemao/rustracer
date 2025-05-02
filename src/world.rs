use std::sync::Arc;

use crate::{
    Sphere,
    color::Color,
    intersection::{Computations, Intersection},
    light::PointLight,
    material::Material,
    matrix::Matrix,
    ray::Ray,
    shapes::Shape,
    vec4::Vec4,
};

#[derive(Debug)]
pub struct World {
    pub light: PointLight,
    pub shapes: Vec<Arc<dyn Shape + Send + Sync>>,
}

impl World {
    pub fn new(light: PointLight) -> Self {
        World {
            light,
            shapes: Vec::new(),
        }
    }
    pub fn default() -> Self {
        let mut world = Self {
            light: PointLight::new(Vec4::point(-10.0, 10.0, -10.0), Color::white()),
            shapes: Vec::new(),
        };

        let mut s1 = Sphere::new();
        let mut mat1 = Material::default();
        mat1.set_color(Color {
            r: 0.8,
            g: 1.0,
            b: 0.6,
        });
        mat1.diffuse = 0.7;
        mat1.specular = 0.2;
        s1.set_material(mat1);
        let s1 = Arc::new(s1);

        let mut s2 = Sphere::new();
        s2.set_material(Material::default());
        s2.set_transformation(Matrix::scaling(0.5, 0.5, 0.5));
        let s2 = Arc::new(s2);

        world.add_shape(s1);
        world.add_shape(s2);
        world
    }
    pub fn add_shape(&mut self, shape: Arc<dyn Shape>) {
        self.shapes.push(shape);
    }
    pub fn intersect(&self, ray: &Ray) -> Vec<Intersection> {
        let mut xs: Vec<Intersection> = Vec::new();
        for shape in &self.shapes {
            xs.extend(shape.intersect(&ray));
        }
        xs.sort();
        xs
    }
    pub fn shade_hit(&self, comps: Computations, remaining: usize) -> Color {
        let in_shadow = self.is_shadowed(&comps.over_point);
        let surface = Material::lighting(
            comps.object().material(),
            comps.object,
            &self.light,
            &comps.point,
            &comps.eyev,
            &comps.normalv,
            in_shadow,
        );
        let reflected = self.reflected_color(&comps, remaining);
        let refracted = self.refracted_color(&comps, remaining);

        let mat = comps.object().material();
        if mat.reflective > 0.0 && mat.transparency > 0.0 {
            let reflectance = comps.schlick();
            return surface + reflected * reflectance + refracted * (1.0 - reflectance);
        }

        surface + reflected + refracted
    }
    pub fn color_at(&self, ray: &Ray, remaining: usize) -> Color {
        let xs = self.intersect(ray);
        if let Some(hit) = Intersection::hit(&xs) {
            let comps = hit.prepare_computations(ray, &xs);
            self.shade_hit(comps, remaining)
        } else {
            Color::black()
        }
    }
    pub fn is_shadowed(&self, point: &Vec4) -> bool {
        let v = self.light.position - *point;
        let dist = v.magnitude();
        let dir = v.norm();

        let r = Ray::from_vec4(*point, dir);
        let intersections = self.intersect(&r);

        if let Some(h) = Intersection::hit(&intersections) {
            if h.t < dist {
                return true;
            }
        }
        false
    }
    pub fn reflected_color(&self, comps: &Computations, remaining: usize) -> Color {
        let c = if comps.object.material().reflective == 0.0 || remaining <= 0 {
            Color::black()
        } else {
            let reflect_ray = Ray::from_vec4(comps.over_point, comps.reflectv);
            self.color_at(&reflect_ray, remaining - 1)
        };
        c * comps.object.material().reflective
    }
    pub fn refracted_color(&self, comps: &Computations, remaining: usize) -> Color {
        if remaining == 0 {
            return Color::black();
        }

        let transparency = comps.object.material().transparency;
        if transparency == 0.0 {
            return Color::black();
        }
        let n_ratio = comps.n1 / comps.n2;
        let cos_i = comps.eyev.dot(&comps.normalv);
        let sin2_t = n_ratio.powi(2) * (1.0 - cos_i.powi(2));
        if sin2_t > 1.0 {
            return Color::black();
        }
        let cos_t = (1.0 - sin2_t).sqrt();
        let direction = comps.normalv * (n_ratio * cos_i - cos_t) - comps.eyev * n_ratio;
        let refract_ray = Ray::from_vec4(comps.under_point, direction);

        self.color_at(&refract_ray, remaining - 1) * transparency
    }
}

#[cfg(test)]
pub mod tests {
    use std::f64::consts::SQRT_2;
    use std::sync::Arc;

    use crate::Sphere;
    use crate::color::Color;
    use crate::intersection::Intersection;
    use crate::light::PointLight;
    use crate::material::Material;
    use crate::math::{ApproxEq, EPSILON};
    use crate::matrix::Matrix;
    use crate::patterns::TestPattern;
    use crate::ray::Ray;
    use crate::shapes::Shape;
    use crate::shapes::plane::Plane;
    use crate::vec4::Vec4;

    use super::World;

    #[test]
    fn cast_ray_on_world() {
        let w = World::default();
        let r = Ray::new(0.0, 0.0, -5.0, 0.0, 0.0, 1.0);
        let xs = w.intersect(&r);
        assert_eq!(xs.len(), 4);
        assert_eq!(xs[0].t, 4.0);
        assert_eq!(xs[1].t, 4.5);
        assert_eq!(xs[2].t, 5.5);
        assert_eq!(xs[3].t, 6.0);
    }

    #[test]
    fn shading_an_intersection() {
        let w = World::default();
        let r = Ray::new(0.0, 0.0, -5.0, 0.0, 0.0, 1.0);

        let shape = &*w.shapes[0]; // first object
        let i = Intersection::new(4.0, shape, None, None);
        let comps = i.prepare_computations(&r, &Vec::<Intersection>::new());
        let c = w.shade_hit(comps, 0);

        let expected = Color::new(0.38066, 0.47583, 0.2855);
        assert_eq!(c, expected);
    }

    #[test]
    fn shading_an_intersection_from_inside() {
        let mut w = World::default();
        w.light = PointLight::new(Vec4::point(0.0, 0.25, 0.0), Color::new(1.0, 1.0, 1.0));

        let r = Ray::new(0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        let shape = &*w.shapes[1];
        let i = Intersection::new(0.5, shape, None, None);
        let comps = i.prepare_computations(&r, &Vec::<Intersection>::new());
        let c = w.shade_hit(comps, 0);

        let expected = Color::new(0.90498, 0.90498, 0.90498);
        assert_eq!(c, expected);
    }
    #[test]
    fn ray_miss() {
        let w = World::default();
        let r = Ray::new(0.0, 0.0, -5.0, 0.0, 1.0, 0.0);
        let c = w.color_at(&r, 0);
        assert_eq!(c, Color::black());
    }
    #[test]
    fn ray_hit() {
        let w = World::default();
        let r = Ray::new(0.0, 0.0, -5.0, 0.0, 0.0, 1.0);
        let c = w.color_at(&r, 0);
        assert_eq!(c, Color::new(0.38066, 0.47583, 0.2855));
    }

    #[test]
    fn test_shading() {
        let w = World::default();
        let p = Vec4::point(-2.0, 2.0, -2.0);
        assert_eq!(w.is_shadowed(&p), false);
        let w = World::default();
        let p = Vec4::point(10.0, -10.0, 10.0);
        assert_eq!(w.is_shadowed(&p), true);
        let w = World::default();
        let p = Vec4::point(-20.0, 20.0, -20.0);
        assert_eq!(w.is_shadowed(&p), false);
    }

    #[test]
    fn reflect_on_non_reflect() {
        let mut world = World {
            light: PointLight::new(Vec4::point(-10.0, 10.0, -10.0), Color::white()),
            shapes: Vec::new(),
        };

        let mut mat1 = Material::default();
        mat1.set_color(Color {
            r: 0.8,
            g: 1.0,
            b: 0.6,
        });
        mat1.diffuse = 0.7;
        mat1.specular = 0.2;

        let s1 = Sphere {
            id: 0,
            transform: Matrix::eye(),
            material: mat1,
            inverse: Matrix::eye(),
        };
        let s1 = Arc::new(s1);
        let mut mat2 = Material::default();
        mat2.ambient = 1.0;
        let s2 = Sphere {
            id: 1,
            material: mat2,
            transform: Matrix::scaling(0.5, 0.5, 0.5),
            inverse: Matrix::scaling(0.5, 0.5, 0.5).inverse(),
        };
        let s2 = Arc::new(s2);

        world.add_shape(s1);
        world.add_shape(s2);

        let r = Ray::new(0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        let shape = &world.shapes[1];

        let i = Intersection::new(1.0, shape.as_ref(), None, None);
        let comps = i.prepare_computations(&r, &Vec::<Intersection>::new());
        let color = world.reflected_color(&comps, 3);
        assert_eq!(color, Color::black());
    }

    #[test]
    fn reflected_color_for_a_reflective_material() {
        let mut world = World {
            light: PointLight::new(Vec4::point(-10.0, 10.0, -10.0), Color::white()),
            shapes: Vec::new(),
        };

        let mut mat1 = Material::default();
        mat1.set_color(Color {
            r: 0.8,
            g: 1.0,
            b: 0.6,
        });
        mat1.diffuse = 0.7;
        mat1.specular = 0.2;

        let s1 = Sphere {
            id: 0,
            transform: Matrix::eye(),
            material: mat1,
            inverse: Matrix::eye(),
        };
        let s1 = Arc::new(s1);
        let mut mat2 = Material::default();
        mat2.ambient = 1.0;
        let s2 = Sphere {
            id: 1,
            material: mat2,
            transform: Matrix::scaling(0.5, 0.5, 0.5),
            inverse: Matrix::scaling(0.5, 0.5, 0.5).inverse(),
        };
        let s2 = Arc::new(s2);

        world.add_shape(s1);
        world.add_shape(s2);

        let mut mat3 = Material::default();
        mat3.reflective = 0.5;
        let plane = Plane {
            id: 2,
            material: mat3,
            transform: Matrix::translation(0.0, -1.0, 0.0),
            inverse: Matrix::translation(0.0, -1.0, 0.0).inverse(),
        };
        world.add_shape(Arc::new(plane));

        let r = Ray::new(0.0, 0.0, -3.0, 0.0, -SQRT_2 / 2.0, SQRT_2 / 2.0);
        let shape = &world.shapes[2];

        let i = Intersection::new(SQRT_2, shape.as_ref(), None, None);
        let comps = i.prepare_computations(&r, &Vec::<Intersection>::new());
        let color = world.reflected_color(&comps, 3);
        assert_eq!(color, Color::new(0.19032, 0.2379, 0.14274));
    }

    #[test]
    fn shade_hit_with_reflective_material() {
        let mut world = World {
            light: PointLight::new(Vec4::point(-10.0, 10.0, -10.0), Color::white()),
            shapes: Vec::new(),
        };

        let mut mat1 = Material::default();
        mat1.set_color(Color {
            r: 0.8,
            g: 1.0,
            b: 0.6,
        });
        mat1.diffuse = 0.7;
        mat1.specular = 0.2;

        let s1 = Sphere {
            id: 0,
            transform: Matrix::eye(),
            material: mat1,
            inverse: Matrix::eye(),
        };
        let s1 = Arc::new(s1);
        let mut mat2 = Material::default();
        mat2.ambient = 1.0;
        let s2 = Sphere {
            id: 1,
            material: mat2,
            transform: Matrix::scaling(0.5, 0.5, 0.5),
            inverse: Matrix::scaling(0.5, 0.5, 0.5).inverse(),
        };
        let s2 = Arc::new(s2);

        world.add_shape(s1);
        world.add_shape(s2);

        let mut mat3 = Material::default();
        mat3.reflective = 0.5;
        let plane = Plane {
            id: 2,
            material: mat3,
            transform: Matrix::translation(0.0, -1.0, 0.0),
            inverse: Matrix::translation(0.0, -1.0, 0.0).inverse(),
        };
        world.add_shape(Arc::new(plane));

        let r = Ray::new(0.0, 0.0, -3.0, 0.0, -SQRT_2 / 2.0, SQRT_2 / 2.0);
        let shape = &world.shapes[2];

        let i = Intersection::new(SQRT_2, shape.as_ref(), None, None);
        let comps = i.prepare_computations(&r, &Vec::<Intersection>::new());
        let color = world.shade_hit(comps, 1);
        assert_eq!(color, Color::new(0.87677, 0.92436, 0.82918));
    }

    #[test]
    fn color_at_with_mutually_reflective_surfaces() {
        let mut world = World {
            light: PointLight::new(Vec4::point(0.0, 0.0, 0.0), Color::white()),
            shapes: Vec::new(),
        };

        let mut mat1 = Material::default();
        mat1.reflective = 1.0;
        let lower = Plane {
            id: 0,
            transform: Matrix::translation(0.0, -1.0, 0.0),
            material: mat1,
            inverse: Matrix::translation(0.0, -1.0, 0.0).inverse(),
        };
        world.add_shape(Arc::new(lower));

        let mut mat2 = Material::default();
        mat2.reflective = 1.0;
        let higher = Plane {
            id: 1,
            transform: Matrix::translation(0.0, 1.0, 0.0),
            material: mat2,
            inverse: Matrix::translation(0.0, 1.0, 0.0).inverse(),
        };
        world.add_shape(Arc::new(higher));

        let r = Ray::new(0.0, 0.0, 0.0, 0.0, 1.0, 0.0);

        let _c = world.color_at(&r, 0);
    }
    #[test]
    fn color_at_max_recursion() {
        let mut world = World::default();

        let mut mat3 = Material::default();
        mat3.reflective = 0.5;
        let plane = Plane {
            id: 2,
            material: mat3,
            transform: Matrix::translation(0.0, -1.0, 0.0),
            inverse: Matrix::translation(0.0, -1.0, 0.0).inverse(),
        };
        world.add_shape(Arc::new(plane));

        let r = Ray::new(0.0, 0.0, -3.0, 0.0, -SQRT_2 / 2.0, SQRT_2 / 2.0);
        let shape = &world.shapes[2];

        let i = Intersection::new(SQRT_2, shape.as_ref(), None, None);
        let comps = i.prepare_computations(&r, &Vec::<Intersection>::new());
        let color = world.reflected_color(&comps, 0);
        assert_eq!(color, Color::black());
    }

    #[test]
    fn transluscence_1() {
        let mut a = Sphere::glas(1.5);
        a.set_transformation(Matrix::scaling(2.0, 2.0, 2.0));
        let mut b = Sphere::glas(2.0);
        b.set_transformation(Matrix::translation(0.0, 0.0, -0.25));
        let mut c = Sphere::glas(2.5);
        c.set_transformation(Matrix::translation(0.0, 0.0, 0.25));

        let mut world = World::new(PointLight::new(
            Vec4::point(0.0, 0.0, 0.0),
            Color::new(1.0, 1.0, 1.0),
        ));
        world.add_shape(Arc::new(a));
        world.add_shape(Arc::new(b));
        world.add_shape(Arc::new(c));

        let r = Ray::from_vec4(Vec4::point(0.0, 0.0, -4.0), Vec4::vector(0.0, 0.0, 1.0));

        let a = &world.shapes[0];
        let b = &world.shapes[1];
        let c = &world.shapes[2];
        let xs = vec![
            Intersection::new(2.0, a.as_ref(), None, None),
            Intersection::new(2.75, b.as_ref(), None, None),
            Intersection::new(3.25, c.as_ref(), None, None),
            Intersection::new(4.75, b.as_ref(), None, None),
            Intersection::new(5.25, c.as_ref(), None, None),
            Intersection::new(6.0, a.as_ref(), None, None),
        ];
        let expected = [
            [1.0, 1.5],
            [1.5, 2.0],
            [2.0, 2.5],
            [2.5, 2.5],
            [2.5, 1.5],
            [1.5, 1.0],
        ];
        let mut index = 0;
        for i in &xs {
            let comps = i.prepare_computations(&r, &xs);
            assert_eq!(comps.n1, expected[index][0]);
            assert_eq!(comps.n2, expected[index][1]);
            index += 1;
        }
    }
    #[test]
    fn transluscence_2() {
        let mut a = Sphere::glas(1.5);
        a.set_transformation(Matrix::translation(0.0, 0.0, 1.0));

        let mut world = World::new(PointLight::new(
            Vec4::point(0.0, 0.0, 0.0),
            Color::new(1.0, 1.0, 1.0),
        ));
        world.add_shape(Arc::new(a));

        let r = Ray::from_vec4(Vec4::point(0.0, 0.0, -5.0), Vec4::vector(0.0, 0.0, 1.0));

        let a = &world.shapes[0];

        let i = Intersection::new(5.0, a.as_ref(), None, None);
        let xs = world.intersect(&r);
        let comps = i.prepare_computations(&r, &xs);
        assert!(comps.under_point.z > EPSILON / 2.0);
        assert!(comps.point.z < comps.under_point.z);
    }

    #[test]
    fn transluscence_3() {
        let w = World::default();
        let shape = &w.shapes[0];
        let r = Ray::new(0.0, 0.0, -5.0, 0.0, 0.0, 1.0);

        let xs = vec![
            Intersection::new(4.0, shape.as_ref(), None, None),
            Intersection::new(6.0, shape.as_ref(), None, None),
        ];
        let comps = xs[0].prepare_computations(&r, &xs);
        let c = w.refracted_color(&comps, 5);
        assert_eq!(c, Color::black());
    }
    #[test]
    fn transluscence_4() {
        let mut w = World {
            light: PointLight::new(Vec4::point(-10.0, 10.0, -10.0), Color::white()),
            shapes: Vec::new(),
        };

        let mut s1 = Sphere::new();
        let mut mat1 = Material::default();
        mat1.set_color(Color {
            r: 0.8,
            g: 1.0,
            b: 0.6,
        });
        mat1.diffuse = 0.7;
        mat1.specular = 0.2;
        mat1.transparency = 1.0;
        mat1.refractive_index = 1.5;
        s1.set_material(mat1);
        let s1 = Arc::new(s1);

        let mut s2 = Sphere::new();
        s2.set_material(Material::default());
        s2.set_transformation(Matrix::scaling(0.5, 0.5, 0.5));
        let s2 = Arc::new(s2);

        w.add_shape(s1);
        w.add_shape(s2);

        let shape = &w.shapes[0];

        let r = Ray::new(0.0, 0.0, -5.0, 0.0, 0.0, 1.0);

        let xs = vec![
            Intersection::new(4.0, shape.as_ref(), None, None),
            Intersection::new(6.0, shape.as_ref(), None, None),
        ];
        let comps = xs[0].prepare_computations(&r, &xs);
        let c = w.refracted_color(&comps, 0);
        assert_eq!(c, Color::black());
    }

    #[test]
    fn transluscence_5() {
        let mut w = World {
            light: PointLight::new(Vec4::point(-10.0, 10.0, -10.0), Color::white()),
            shapes: Vec::new(),
        };

        let mut s1 = Sphere::new();
        let mut mat1 = Material::default();
        mat1.set_color(Color {
            r: 0.8,
            g: 1.0,
            b: 0.6,
        });
        mat1.diffuse = 0.7;
        mat1.specular = 0.2;
        mat1.transparency = 1.0;
        mat1.refractive_index = 1.5;
        s1.set_material(mat1);
        let s1 = Arc::new(s1);

        let mut s2 = Sphere::new();
        s2.set_material(Material::default());
        s2.set_transformation(Matrix::scaling(0.5, 0.5, 0.5));
        let s2 = Arc::new(s2);

        w.add_shape(s1);
        w.add_shape(s2);

        let shape = &w.shapes[0];

        let r = Ray::new(0.0, 0.0, SQRT_2 / 2.0, 0.0, 1.0, 0.0);

        let xs = vec![
            Intersection::new(-SQRT_2 / 2.0, shape.as_ref(), None, None),
            Intersection::new(SQRT_2 / 2.0, shape.as_ref(), None, None),
        ];
        let comps = xs[1].prepare_computations(&r, &xs);
        let c = w.refracted_color(&comps, 5);
        assert_eq!(c, Color::black());
    }
    #[test]
    fn transluscence_6() {
        let mut w = World {
            light: PointLight::new(Vec4::point(-10.0, 10.0, -10.0), Color::white()),
            shapes: Vec::new(),
        };

        let mut s1 = Sphere::new();
        let mut mat1 = Material::default();
        mat1.set_color(Color {
            r: 0.8,
            g: 1.0,
            b: 0.6,
        });
        mat1.diffuse = 0.7;
        mat1.specular = 0.2;
        mat1.ambient = 1.0;
        mat1.set_pattern(TestPattern::new());
        s1.set_material(mat1);
        let s1 = Arc::new(s1);

        let mut s2 = Sphere::new();
        let mut mat2 = Material::default();
        mat2.transparency = 1.0;
        mat2.refractive_index = 1.5;
        s2.set_material(mat2);
        s2.set_transformation(Matrix::scaling(0.5, 0.5, 0.5));
        let s2 = Arc::new(s2);

        w.add_shape(s1);
        w.add_shape(s2);

        let a = &w.shapes[0];
        let b = &w.shapes[1];

        let r = Ray::new(0.0, 0.0, 0.1, 0.0, 1.0, 0.0);

        let xs = vec![
            Intersection::new(-0.9899, a.as_ref(), None, None),
            Intersection::new(-0.4899, b.as_ref(), None, None),
            Intersection::new(0.4899, b.as_ref(), None, None),
            Intersection::new(0.9899, a.as_ref(), None, None),
        ];
        let comps = xs[2].prepare_computations(&r, &xs);
        let c = w.refracted_color(&comps, 5);
        assert_eq!(c, Color::new(0.0, 0.9988, 0.04725));
    }

    #[test]
    fn transluscence_7() {
        let mut w = World::default();

        let mut floor = Plane::new();
        floor.set_transformation(Matrix::translation(0.0, -1.0, 0.0));
        let mut mat = Material::default();
        mat.transparency = 0.5;
        mat.refractive_index = 1.5;
        floor.set_material(mat);
        w.add_shape(Arc::new(floor));

        let mut ball = Sphere::new();
        let mut mat = Material::default();
        mat.color = Color::new(1.0, 0.0, 0.0);
        mat.ambient = 0.5;
        ball.set_material(mat);
        ball.set_transformation(Matrix::translation(0.0, -3.5, -0.5));
        w.add_shape(Arc::new(ball));
        let r = Ray::new(0.0, 0.0, -3.0, 0.0, -SQRT_2 / 2.0, SQRT_2 / 2.0);
        let floor = w.shapes[2].as_ref();
        let xs = vec![Intersection::new(SQRT_2, floor, None, None)];
        let comps = xs[0].prepare_computations(&r, &xs);
        let color = w.shade_hit(comps, 5);
        assert_eq!(color, Color::new(0.93642, 0.68642, 0.68642));
    }

    #[test]
    fn fresnel1() {
        let shape = Sphere::glas(1.5);
        let r = Ray::new(0.0, 0.0, SQRT_2 / 2.0, 0.0, 1.0, 0.0);
        let mut w = World::default();
        w.shapes.remove(0);
        w.shapes.remove(0);
        w.add_shape(Arc::new(shape));
        let shape = w.shapes[0].as_ref();
        let xs = vec![
            Intersection::new(-SQRT_2 / 2.0, shape, None, None),
            Intersection::new(SQRT_2 / 2.0, shape, None, None),
        ];
        let comps = xs[1].prepare_computations(&r, &xs);
        let reflectance = comps.schlick();
        assert_eq!(reflectance, 1.0);
    }
    #[test]
    fn fresnel2() {
        let shape = Sphere::glas(1.5);
        let r = Ray::new(0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
        let mut w = World::default();
        w.shapes.remove(0);
        w.shapes.remove(0);
        w.add_shape(Arc::new(shape));
        let shape = w.shapes[0].as_ref();
        let xs = vec![
            Intersection::new(-1.0, shape, None, None),
            Intersection::new(1.0, shape, None, None),
        ];
        let comps = xs[1].prepare_computations(&r, &xs);
        let reflectance = comps.schlick();
        assert!(reflectance.approx_eq(&0.04));
    }
    #[test]
    fn fresnel3() {
        let shape = Sphere::glas(1.5);
        let r = Ray::new(0.0, 0.99, -2.0, 0.0, 0.0, 1.0);
        let mut w = World::default();
        w.shapes.remove(0);
        w.shapes.remove(0);
        w.add_shape(Arc::new(shape));
        let shape = w.shapes[0].as_ref();
        let xs = vec![Intersection::new(1.8589, shape, None, None)];
        let comps = xs[0].prepare_computations(&r, &xs);
        let reflectance = comps.schlick();
        assert!(reflectance.approx_eq(&0.48873));
    }

    #[test]
    fn fresnel4() {
        let mut w = World::default();

        let mut floor = Plane::new();
        floor.set_transformation(Matrix::translation(0.0, -1.0, 0.0));
        let mut mat = Material::default();
        mat.transparency = 0.5;
        mat.refractive_index = 1.5;
        mat.reflective = 0.5;
        floor.set_material(mat);
        w.add_shape(Arc::new(floor));

        let mut ball = Sphere::new();
        let mut mat = Material::default();
        mat.color = Color::new(1.0, 0.0, 0.0);
        mat.ambient = 0.5;
        ball.set_material(mat);
        ball.set_transformation(Matrix::translation(0.0, -3.5, -0.5));
        w.add_shape(Arc::new(ball));
        let r = Ray::new(0.0, 0.0, -3.0, 0.0, -SQRT_2 / 2.0, SQRT_2 / 2.0);
        let floor = w.shapes[2].as_ref();
        let xs = vec![Intersection::new(SQRT_2, floor, None, None)];
        let comps = xs[0].prepare_computations(&r, &xs);
        let color = w.shade_hit(comps, 5);
        assert!(color.r.approx_eq(&0.93391), "red channel off");
        assert!(color.g.approx_eq(&0.69643), "green channel off");
        assert!(color.b.approx_eq(&0.69243), "blue channel off");
    }
}
