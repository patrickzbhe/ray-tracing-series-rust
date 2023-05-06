use crate::mutil::clamp;
use crate::perlin::Perlin;
use crate::screen::Screen;
use crate::vec3::{Color, Point3};
use std::sync::Arc;

pub trait Texture: Send + Sync {
    fn value(&self, u: f64, v: f64, p: &Point3) -> Color;
}

pub struct SolidColor {
    color_value: Color,
}

impl SolidColor {
    pub fn new(color_value: &Color) -> SolidColor {
        SolidColor {
            color_value: color_value.clone(),
        }
    }

    pub fn from_colors(red: f64, green: f64, blue: f64) -> SolidColor {
        SolidColor::new(&Color::new(red, green, blue))
    }
}

impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _p: &Point3) -> Color {
        self.color_value
    }
}

pub struct Checker {
    even: Arc<Box<dyn Texture>>,
    odd: Arc<Box<dyn Texture>>,
}

impl Checker {
    pub fn new(even: Arc<Box<dyn Texture>>, odd: Arc<Box<dyn Texture>>) -> Checker {
        Checker {
            even: even.clone(),
            odd: odd.clone(),
        }
    }

    pub fn from_colors(even: &Color, odd: &Color) -> Checker {
        Checker {
            even: Arc::new(Box::new(SolidColor::new(even))),
            odd: Arc::new(Box::new(SolidColor::new(odd))),
        }
    }
}

impl Texture for Checker {
    fn value(&self, u: f64, v: f64, p: &Point3) -> Color {
        let sines = f64::sin(10.0 * p.get_x()) * f64::sin(10.0 * p.get_y()) * f64::sin(10.0 * p.get_z());
        if sines < 0.0 {
            self.odd.value(u, v, p)
        } else {
            self.even.value(u, v, p)
        }
    }
}

pub struct Noise {
    noise: Perlin,
    scale: f64,
}

impl Noise {
    pub fn new(scale: f64) -> Noise {
        Noise {
            noise: Perlin::new(),
            scale,
        }
    }
}

impl Texture for Noise {
    fn value(&self, _u: f64, _v: f64, p: &Point3) -> Color {
        //Color::new(1,1,1) * 0.5 * (1.0 + self.noise.noise(&(self.scale * *p)))
        //Color::new(1,1,1) * self.noise.turbulence(&(self.scale * *p), 7)
        Color::new(1, 1, 1)
            * 0.5
            * (1.0 + f64::sin(self.scale * p.get_z() + 10.0 * self.noise.turbulence(p, 7)))
    }
}

pub struct Image {
    data: Screen,
}

impl Image {
    pub fn from_ppm(name: &str) -> Image {
        Image {
            data: Screen::from_ppm_p3(name),
        }
    }
}

impl Texture for Image {
    fn value(&self, u: f64, v: f64, p: &Point3) -> Color {
        let u = clamp(u, 0.0, 1.0);
        let v = 1.0 - clamp(v, 0.0, 1.0);

        let mut i = (u * self.data.get_width() as f64) as i32;
        let mut j = (v * self.data.get_height() as f64) as i32;

        i = i32::min(i, self.data.get_width() as i32 - 1);
        j = i32::min(j, self.data.get_height() as i32 - 1);

        let color_scale = 1.0 / 255.0;
        let pixel = self.data.get(j as usize, i as usize);

        Color::new(
            color_scale * pixel.get_x(),
            color_scale * pixel.get_y(),
            color_scale * pixel.get_z(),
        )
    }
}
