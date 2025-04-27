use crate::{color::Color, light::PointLight, math::ApproxEq, patterns::Pattern, shapes::Shape, vec4::Vec4};

#[derive(Debug)]
pub struct Material {
    pub color: Color,
    pub ambient: f64,
    pub diffuse: f64,
    pub specular: f64,
    pub shininess: f64,
    pub pattern: Option<Box<dyn Pattern>>,
}

impl Material {
    pub fn default() -> Self {
        Self {
            color: Color::white(),
            ambient: 0.1,
            diffuse: 0.9,
            specular: 0.9,
            shininess: 200.0,
            pattern: None,
        }
    }
    pub fn new(color: Color, ambient: f64, diffuse: f64, specular: f64, shininess: f64) -> Self {
        Self {
            color,
            ambient,
            diffuse,
            specular,
            shininess,
            pattern: None,
        }
    }
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    pub fn set_pattern(&mut self, pattern: impl Pattern + 'static) {
        self.pattern = Some(Box::new(pattern));
    }
    pub fn lighting(
        material: &Material,
        object: &dyn Shape,
        light: &PointLight,
        point: &Vec4,
        eyev: &Vec4,
        normalv: &Vec4,
        in_shadow: bool,
    ) -> Color {
        let mut effective_color = match material.pattern {
            Some(ref p) => p.pattern_at(object, point),
            None => material.color,
        };
        effective_color = effective_color * light.intensity;
        let lightv = (light.position - *point).norm();
        let ambient = effective_color * material.ambient;
        let light_dot_normal = lightv.dot(&normalv);

        if in_shadow {
            return ambient;
        }

        let (diffuse, specular) = if light_dot_normal < 0.0 {
            (Color::black(), Color::black())
        } else {
            let diffuse = effective_color * material.diffuse * light_dot_normal;

            let reflectv = (-lightv).reflect(&normalv);
            let reflect_dot_eye = reflectv.dot(&eyev);

            let specular = if reflect_dot_eye <= 0.0 {
                Color::black()
            } else {
                let factor = reflect_dot_eye.powf(material.shininess);
                light.intensity * material.specular * factor
            };
            (diffuse, specular)
        };
        ambient + diffuse + specular
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{Sphere, patterns::StripePattern};

    use super::*;
    use std::f64::consts::SQRT_2;

    #[test]
    pub fn lighting_with_eye_offset_45_degrees() {
        let m = Material::default();
        let position = Vec4::point(0.0, 0.0, 0.0);
        let eyev = Vec4::vector(0.0, SQRT_2 / 2.0, -SQRT_2 / 2.0);
        let normalv = Vec4::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Vec4::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let s = Box::new(Sphere::new());
        let result = Material::lighting(&m, s.as_ref(), &light, &position, &eyev, &normalv, false);

        let expected = Color::new(1.0, 1.0, 1.0);
        assert_eq!(result, expected);
    }

    #[test]
    pub fn lighting_with_light_offset_45_degrees() {
        let m = Material::default();
        let position = Vec4::point(0.0, 0.0, 0.0);
        let eyev = Vec4::vector(0.0, 0.0, -1.0);
        let normalv = Vec4::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Vec4::point(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));

        let s = Box::new(Sphere::new());
        let result = Material::lighting(&m, s.as_ref(), &light, &position, &eyev, &normalv, false);

        let expected = Color::new(0.7364, 0.7364, 0.7364);
        assert_eq!(result, expected);
    }
    #[test]
    pub fn lighting_with_eye_in_reflection_path() {
        use std::f64::consts::SQRT_2;

        let m = Material::default();
        let position = Vec4::point(0.0, 0.0, 0.0);
        let eyev = Vec4::vector(0.0, -SQRT_2 / 2.0, -SQRT_2 / 2.0);
        let normalv = Vec4::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Vec4::point(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));

        let s = Box::new(Sphere::new());
        let result = Material::lighting(&m, s.as_ref(), &light, &position, &eyev, &normalv, false);

        let expected = Color::new(1.6364, 1.6364, 1.6364);
        assert_eq!(result, expected);
    }
    #[test]
    pub fn lighting_with_light_behind_surface() {
        let m = Material::default();
        let position = Vec4::point(0.0, 0.0, 0.0);
        let eyev = Vec4::vector(0.0, 0.0, -1.0);
        let normalv = Vec4::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Vec4::point(0.0, 0.0, 10.0), Color::new(1.0, 1.0, 1.0));

        let s = Box::new(Sphere::new());
        let result = Material::lighting(&m, s.as_ref(), &light, &position, &eyev, &normalv, false);

        let expected = Color::new(0.1, 0.1, 0.1);
        assert_eq!(result, expected);
    }
    #[test]
    fn lighting_with_surface_shadow() {
        let m = Material::default();
        let position = Vec4::point(0.0, 0.0, 0.0);
        let eyev = Vec4::vector(0.0, 0.0, -1.0);
        let normalv = Vec4::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Vec4::point(0.0, 0.0, 10.0), Color::new(1.0, 1.0, 1.0));

        let s = Box::new(Sphere::new());
        let result = Material::lighting(&m, s.as_ref(), &light, &position, &eyev, &normalv, false);
        let expected = Color::new(0.1, 0.1, 0.1);
        assert_eq!(result, expected);
    }
    #[test]
    fn pattern_test() {
        let mut m = Material::default();
        m.ambient = 1.0;
        m.diffuse = 0.0;
        m.specular = 0.0;
        m.set_pattern(StripePattern::new(Color::white(), Color::black()));
        let eyev = Vec4::vector(0.0, 0.0, -1.0);
        let normalv = Vec4::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Vec4::point(0.0, 0.0, -10.0), Color::white());
        let s = Box::new(Sphere::new());
        let c1 = Material::lighting(
            &m,
            s.as_ref(),
            &light,
            &Vec4::point(0.9, 0.0, 0.0),
            &eyev,
            &normalv,
            false,
        );
        let c2 = Material::lighting(
            &m,
            s.as_ref(),
            &light,
            &Vec4::point(1.1, 0.0, 0.0),
            &eyev,
            &normalv,
            false,
        );
        assert_eq!(c1, Color::white());
        assert_eq!(c2, Color::black());
    }
}
