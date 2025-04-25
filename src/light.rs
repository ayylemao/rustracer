use crate::{color::Color, vec4::Vec4};

#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub position: Vec4,
    pub intensity: Color,
}

impl PointLight {
    pub fn new(position: Vec4, intensity: Color) -> Self {
        Self {
            position,
            intensity,
        }
    }
}

#[cfg(test)]
pub mod tests {}
