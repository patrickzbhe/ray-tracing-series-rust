use crate::vec3::{Color, Point3};
use std::sync::Arc;
use crate::perlin::Perlin;

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
        let sines = f64::sin(10.0 * p.x()) * f64::sin(10.0 * p.y()) * f64::sin(10.0 * p.z());
        if sines < 0.0 {
            self.odd.value(u, v, p)
        } else {
            self.even.value(u, v, p)
        }
    }
}


pub struct Noise {
    noise: Perlin,
}

impl Noise {
    pub fn new() -> Noise {
        Noise { noise: Perlin::new() }
    }
}

impl Texture for Noise {
    fn value(&self, _u: f64, _v: f64, p: &Point3) -> Color {

        Color::new(1,1,1) * self.noise.noise(p)
    }
}