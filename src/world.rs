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
        surface + reflected
    }
    pub fn color_at(&self, ray: &Ray, remaining: usize) -> Color {
        let xs = self.intersect(ray);
        if let Some(hit) = Intersection::hit(&xs) {
            let comps = hit.prepare_computations(ray, &Vec::<Intersection>::new());
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
    use crate::matrix::Matrix;
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
        let i = Intersection::new(4.0, shape);
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
        let i = Intersection::new(0.5, shape);
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
        };
        let s1 = Arc::new(s1);
        let mut mat2 = Material::default();
        mat2.ambient = 1.0;
        let s2 = Sphere {
            id: 1,
            material: mat2,
            transform: Matrix::scaling(0.5, 0.5, 0.5),
        };
        let s2 = Arc::new(s2);

        world.add_shape(s1);
        world.add_shape(s2);

        let r = Ray::new(0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        let shape = &world.shapes[1];

        let i = Intersection::new(1.0, shape.as_ref());
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
        };
        let s1 = Arc::new(s1);
        let mut mat2 = Material::default();
        mat2.ambient = 1.0;
        let s2 = Sphere {
            id: 1,
            material: mat2,
            transform: Matrix::scaling(0.5, 0.5, 0.5),
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
        };
        world.add_shape(Arc::new(plane));

        let r = Ray::new(0.0, 0.0, -3.0, 0.0, -SQRT_2 / 2.0, SQRT_2 / 2.0);
        let shape = &world.shapes[2];

        let i = Intersection::new(SQRT_2, shape.as_ref());
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
        };
        let s1 = Arc::new(s1);
        let mut mat2 = Material::default();
        mat2.ambient = 1.0;
        let s2 = Sphere {
            id: 1,
            material: mat2,
            transform: Matrix::scaling(0.5, 0.5, 0.5),
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
        };
        world.add_shape(Arc::new(plane));

        let r = Ray::new(0.0, 0.0, -3.0, 0.0, -SQRT_2 / 2.0, SQRT_2 / 2.0);
        let shape = &world.shapes[2];

        let i = Intersection::new(SQRT_2, shape.as_ref());
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
        };
        world.add_shape(Arc::new(lower));

        let mut mat2 = Material::default();
        mat2.reflective = 1.0;
        let higher = Plane {
            id: 1,
            transform: Matrix::translation(0.0, 1.0, 0.0),
            material: mat2,
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
        };
        world.add_shape(Arc::new(plane));

        let r = Ray::new(0.0, 0.0, -3.0, 0.0, -SQRT_2 / 2.0, SQRT_2 / 2.0);
        let shape = &world.shapes[2];

        let i = Intersection::new(SQRT_2, shape.as_ref());
        let comps = i.prepare_computations(&r, &Vec::<Intersection>::new());
        let color = world.reflected_color(&comps, 0);
        assert_eq!(color, Color::black());
    }

    #[test]
    fn transluscence() {
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
            Intersection::new(2.0, a.as_ref()),
            Intersection::new(2.75, b.as_ref()),
            Intersection::new(3.25, c.as_ref()),
            Intersection::new(4.75, b.as_ref()),
            Intersection::new(5.25, c.as_ref()),
            Intersection::new(6.0, a.as_ref()),
        ];
        let expected = [[1.0, 1.5], [1.5, 2.0], [2.0, 2.5], [2.5, 2.5], [2.5, 1.5], [1.5, 1.0]];
        let mut index = 0;
        for i in &xs {
            let comps = i.prepare_computations(&r, &xs);
            assert_eq!(comps.n1, expected[index][0]);
            assert_eq!(comps.n2, expected[index][1]);
            index += 1;
        }
    }
}
