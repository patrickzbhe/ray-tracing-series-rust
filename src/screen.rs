use crate::vec3::Color;
use std::io::Write;

pub struct Screen {
    width: usize,
    height: usize,
    pixels: Vec<Color>,
}

impl Screen {
    pub fn new(width: usize, height: usize) -> Screen {
        assert!(height > 0 && width > 0);
        Screen {
            height: height,
            width,
            pixels: vec![Color::new(0, 0, 0); height * width],
        }
    }

    pub fn get(&self, i: usize, j: usize) -> &Color {
        assert!(i * self.width + j < self.height * self.width);
        &self.pixels[i * self.width + j]
    }

    pub fn update(&mut self, i: usize, j: usize, color: Color) {
        assert!(i * self.width + j < self.height * self.width);
        self.pixels[i * self.width + j] = color;
    }

    pub fn write_to_ppm(&self) {
        let mut stdout = std::io::stdout().lock();
        writeln!(stdout, "P3\n{} {}\n255", self.width, self.height).unwrap();
        for j in (0..self.height).rev() {
            for i in (0..self.width) {
                writeln!(stdout, "{}", self.get(j, i).get_color()).unwrap();
            }
        }
    }
}
