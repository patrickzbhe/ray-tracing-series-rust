use crate::vec3::Color;
use std::io::Write;
use std::fs::File;
use std::io::prelude::*;

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

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
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
            for i in 0..self.width {
                writeln!(stdout, "{}", self.get(j, i).get_color()).unwrap();
            }
        }
    }

    pub fn from_ppm(name: &str) -> Screen {
        let mut file = File::open(name).expect("Couldn't open the file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Trouble reading file...");
        let mut contents = contents.split("\n");
        contents.next();
        let wh: Vec<&str> = contents.next().unwrap().split(" ").collect();
        let width = wh[0].parse::<usize>().unwrap();
        let height = wh[1].parse::<usize>().unwrap();
        let mut pixels: Vec<Color> = vec![Color::new(0, 0, 0); height * width];
        contents.next();
        for j in (0..height).rev() {
            for i in 0..width {
                let line: Vec<usize> = contents.next().unwrap().split(" ").map(|e| e.parse::<usize>().unwrap()).collect();
                pixels[j * width + i] = Color::new(line[0] as f64,line[1] as f64,line[2] as f64);
            }
        }
        Screen { width, height, pixels }
    }
}
