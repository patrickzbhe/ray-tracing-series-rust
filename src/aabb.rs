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
        let intervals = [
            (
                self.minimum.x(),
                self.maximum.x(),
                r.get_origin().x(),
                r.direction().x(),
            ),
            (
                self.minimum.y(),
                self.maximum.y(),
                r.get_origin().y(),
                r.direction().y(),
            ),
            (
                self.minimum.z(),
                self.maximum.z(),
                r.get_origin().z(),
                r.direction().z(),
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
            f64::min(box0.get_min().x(), box1.get_min().x()),
            f64::min(box0.get_min().y(), box1.get_min().y()),
            f64::min(box0.get_min().z(), box1.get_min().z()),
        );

        let big = Point3::new(
            f64::max(box0.get_max().x(), box1.get_max().x()),
            f64::max(box0.get_max().y(), box1.get_max().y()),
            f64::max(box0.get_max().z(), box1.get_max().z()),
        );

        Aabb::new(small, big)
    }
}
