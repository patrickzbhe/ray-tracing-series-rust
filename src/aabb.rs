use rand::{thread_rng, Rng};

use crate::ray::Ray;
use crate::vec3::Point3;

#[derive(Clone)]
pub struct Aabb {
    minimum: Point3,
    maximum: Point3,
}

impl Aabb {
    pub fn new(minimum: Point3, maximum: Point3) -> Aabb {
        Aabb { minimum, maximum }
    }

    pub fn get_min(&self) -> &Point3 {
        &self.minimum
    }

    pub fn get_max(&self) -> &Point3 {
        &self.maximum
    }

    pub fn hit(&self, r: &Ray, mut t_min: f64, mut t_max: f64) -> bool {
        // the manual loop unroll LMAO
        if thread_rng().gen::<f64>() < 0.001 {
            eprintln!("zz");
        }
        let intervals = [
            (
                self.minimum.get_x(),
                self.maximum.get_x(),
                r.get_origin().get_x(),
                r.get_direction().get_x(),
            ),
            (
                self.minimum.get_y(),
                self.maximum.get_y(),
                r.get_origin().get_y(),
                r.get_direction().get_y(),
            ),
            (
                self.minimum.get_z(),
                self.maximum.get_z(),
                r.get_origin().get_z(),
                r.get_direction().get_z(),
            ),
        ];

        for (min, max, origin, direction) in intervals {
            // x
            let inv_d = 1.0 / direction;
            let mut t0 = (min - origin) * inv_d;
            let mut t1 = (max - origin) * inv_d;
            if inv_d < 0.0 {
                (t0, t1) = (t1, t0)
            }
            t_min = if t0 > t_min { t0 } else { t_min };
            t_max = if t1 < t_max { t1 } else { t_max };
            if t_max <= t_min {
                return false;
            }
        }
        return true;
    }

    pub fn surrounding_box(box0: &Aabb, box1: &Aabb) -> Aabb {
        let small = Point3::new(
            f64::min(box0.get_min().get_x(), box1.get_min().get_x()),
            f64::min(box0.get_min().get_y(), box1.get_min().get_y()),
            f64::min(box0.get_min().get_z(), box1.get_min().get_z()),
        );

        let big = Point3::new(
            f64::max(box0.get_max().get_x(), box1.get_max().get_x()),
            f64::max(box0.get_max().get_y(), box1.get_max().get_y()),
            f64::max(box0.get_max().get_z(), box1.get_max().get_z()),
        );

        Aabb::new(small, big)
    }
}
