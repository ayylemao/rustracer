use std::ops::Neg;

use crate::{color::Color, light::PointLight, vec4::Vec4};

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub color: Color,
    pub ambient: f64,
    pub diffuse: f64,
    pub specular: f64,
    pub shininess: f64,
}

impl Material {
    pub fn default() -> Self {
        Self {
            color: Color::white(),
            ambient: 0.1,
            diffuse: 0.9,
            specular: 0.9,
            shininess: 200.0,
        }
    }
    pub fn new(color: Color, ambient: f64, diffuse: f64, specular: f64, shininess: f64) -> Self {
        Self {
            color,
            ambient,
            diffuse,
            specular,
            shininess,
        }
    }
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    pub fn lighting(&self, light: &PointLight, point: &Vec4, eyev: &Vec4, normalv: &Vec4) -> Color {
        let effective_color = self.color * light.intensity;
        let lightv = (light.position - *point).norm();
        let ambient = effective_color * self.ambient;
        let light_dot_normal = lightv.dot(&normalv);

        let (diffuse, specular) = if light_dot_normal < 0.0 {
            (Color::black(), Color::black())
        } else {
            let diffuse = effective_color * self.diffuse * light_dot_normal;

            let reflectv = lightv.neg().reflect(&normalv);
            let reflect_dot_eye = reflectv.dot(&eyev);

            let specular = if reflect_dot_eye <= 0.0 {
                Color::black()
            } else {
                let factor = reflect_dot_eye.powf(self.shininess);
                light.intensity * self.specular * factor
            };
            (diffuse, specular)
        };
        ambient + diffuse + specular
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::f64::consts::SQRT_2;

    #[test]
    pub fn lighting_with_eye_offset_45_degrees() {
        let m = Material::default();
        let position = Vec4::point(0.0, 0.0, 0.0);
        let eyev = Vec4::vector(0.0, SQRT_2 / 2.0, -SQRT_2 / 2.0);
        let normalv = Vec4::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Vec4::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0));

        let result = m.lighting(&light, &position, &eyev, &normalv);

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

        let result = m.lighting(&light, &position, &eyev, &normalv);

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

        let result = m.lighting(&light, &position, &eyev, &normalv);

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

        let result = m.lighting(&light, &position, &eyev, &normalv);

        let expected = Color::new(0.1, 0.1, 0.1);
        assert_eq!(result, expected);
    }
}
