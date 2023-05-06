use crate::vec3::{Point3, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    origin: Point3,
    direction: Vec3,
    time: f64,
}

impl Ray {
    pub fn new(&origin: &Point3, &direction: &Vec3, time: f64) -> Ray {
        Ray {
            origin,
            direction,
            time,
        }
    }

    pub fn get_origin(&self) -> &Point3 {
        &self.origin
    }

    pub fn get_direction(&self) -> &Vec3 {
        &self.direction
    }

    pub fn get_time(&self) -> f64 {
        self.time
    }

    pub fn at(&self, t: f64) -> Point3 {
        self.origin + self.direction * t
    }
}
