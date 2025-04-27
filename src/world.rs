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
    pub shapes: Vec<Box<dyn Shape>>,
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
        let s1 = Box::new(s1);

        let mut s2 = Sphere::new();
        s2.set_material(Material::default());
        s2.set_transformation(Matrix::scaling(0.5, 0.5, 0.5));
        let s2 = Box::new(s2);

        world.add_shape(s1);
        world.add_shape(s2);
        world
    }
    pub fn add_shape(&mut self, shape: Box<dyn Shape>) {
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
    pub fn shade_hit(&self, comps: Computations) -> Color {
        let in_shadow = self.is_shadowed(&comps.over_point);
        Material::lighting(
            comps.object().material(),
            comps.object,
            &self.light,
            &comps.point,
            &comps.eyev,
            &comps.normalv,
            in_shadow,
        )
    }
    pub fn color_at(&self, ray: &Ray) -> Color {
        let xs = self.intersect(ray);
        if let Some(hit) = Intersection::hit(&xs) {
            let comps = hit.prepare_computations(ray);
            self.shade_hit(comps)
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
}

#[cfg(test)]
pub mod tests {
    use crate::color::Color;
    use crate::intersection::Intersection;
    use crate::light::PointLight;
    use crate::ray::Ray;
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
        let comps = i.prepare_computations(&r);
        let c = w.shade_hit(comps);

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
        let comps = i.prepare_computations(&r);
        let c = w.shade_hit(comps);

        let expected = Color::new(0.90498, 0.90498, 0.90498);
        assert_eq!(c, expected);
    }
    #[test]
    fn ray_miss() {
        let w = World::default();
        let r = Ray::new(0.0, 0.0, -5.0, 0.0, 1.0, 0.0);
        let c = w.color_at(&r);
        assert_eq!(c, Color::black());
    }
    #[test]
    fn ray_hit() {
        let w = World::default();
        let r = Ray::new(0.0, 0.0, -5.0, 0.0, 0.0, 1.0);
        let c = w.color_at(&r);
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
}
