use crate::color::Color;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Color>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Canvas {
        Canvas {
            width: width,
            height: height,
            pixels: vec![Color::black(); width * height],
        }
    }
    pub fn to_ppm(self) -> String {
        let mut ppm_string = String::new();
        ppm_string.push_str(format!("P3\n{} {}\n{}\n", self.width, self.height, 255).as_str());
        for y in 0..self.height {
            let mut row = String::new();
            for x in 0..self.width {
                let color = self[(x, y)];
                let (r, g, b) = color.to_rgb_u8();

                row.push_str(&format!("{} {} {} ", r, g, b));
            }
            row.pop();
            row.push('\n');
            ppm_string.push_str(&row);
        }
        ppm_string.push('\n');
        ppm_string
    }
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        self[(x, y)] = color;
    }
}

impl Index<(usize, usize)> for Canvas {
    type Output = Color;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let (x, y) = index;
        let i = y * self.width + x;
        &self.pixels[i]
    }
}

impl IndexMut<(usize, usize)> for Canvas {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let (x, y) = index;
        let i = y * self.width + x;
        &mut self.pixels[i]
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn header() {
        let c: Canvas = Canvas::new(5, 3);
        let ppm = c.to_ppm();
        assert!(ppm.starts_with("P3\n5 3\n255"));
    }
    #[test]
    fn ppm_pixel_data_lines_4_to_6() {
        let mut c = Canvas::new(5, 3);

        let c1 = Color {
            r: 1.5,
            g: 0.0,
            b: 0.0,
        };
        let c2 = Color {
            r: 0.0,
            g: 0.5,
            b: 0.0,
        };
        let c3 = Color {
            r: -0.5,
            g: 0.0,
            b: 1.0,
        };

        c[(0, 0)] = c1;
        c[(2, 1)] = c2;
        c[(4, 2)] = c3;

        let ppm = c.to_ppm();
        let lines: Vec<&str> = ppm.lines().collect();

        assert_eq!(lines[3], "255 0 0 0 0 0 0 0 0 0 0 0 0 0 0");
        assert_eq!(lines[4], "0 0 0 0 0 0 0 128 0 0 0 0 0 0 0");
        assert_eq!(lines[5], "0 0 0 0 0 0 0 0 0 0 0 0 0 0 255");
    }
}
