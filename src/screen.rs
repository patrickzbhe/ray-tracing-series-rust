use crate::vec3::Color;
use std::fs::{self, File};
use std::io::prelude::*;
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

    pub fn write_to_ppm_file(&self, path: &str) {
        let mut output = String::new();
        output += &format!("P3\n{} {}\n255\n", self.get_width(),self.get_height());
        for j in (0..self.height).rev() {
            for i in 0..self.width {
                output +=  &format!("{}\n", self.get(j, i).get_color());
            }
        }
        fs::write(path, output).unwrap();
    }

    pub fn from_ppm_p3(name: &str) -> Screen {
        let mut file = File::open(name).expect("Couldn't open the file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Trouble reading file...");
        let mut contents = contents.split("\n");
        contents.next();
        let wh: Vec<&str> = contents.next().unwrap().split(" ").collect();
        let width = wh[0].parse::<usize>().unwrap();
        let height = wh[1].parse::<usize>().unwrap();
        let mut pixels: Vec<Color> = vec![Color::new(0, 0, 0); height * width];
        contents.next();
        let nums: Vec<&str> = contents.map(|l| l.split_whitespace()).flatten().collect();
        let mut num_iter = nums.iter();
        for j in 0..height {
            for i in 0..width {
                let (x, y, z) = (
                    num_iter.next().unwrap(),
                    num_iter.next().unwrap(),
                    num_iter.next().unwrap(),
                );

                pixels[j * width + i] = Color::new(
                    x.parse::<f64>().unwrap(),
                    y.parse::<f64>().unwrap(),
                    z.parse::<f64>().unwrap(),
                );
            }
        }
        Screen {
            width,
            height,
            pixels,
        }
    }
}
