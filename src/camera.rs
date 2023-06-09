use rand::{thread_rng, Rng};

use crate::ray::Ray;
use crate::vec3::{random_in_unit_disk, Point3, Vec3};

pub struct Camera {
    origin: Point3,
    lower_left_corner: Point3,
    horizontal: Vec3,
    vertical: Vec3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    lens_radius: f64,
    time1: f64,
    time2: f64,
}

impl Camera {
    pub fn new(
        lookfrom: Point3,
        lookat: Point3,
        vup: Vec3,
        vfov: f64,
        aspect_ratio: f64,
        aperture: f64,
        focus_dist: f64,
        time1: f64,
        time2: f64,
    ) -> Camera {
        let theta = f64::to_radians(vfov);
        let h = f64::tan(theta / 2.0);
        let viewport_height = 2.0 * h;
        let viewport_width = aspect_ratio * viewport_height;

        let w = (lookfrom - lookat).unit();
        let u = (vup.cross(&w)).unit();
        let v = w.cross(&u);

        let origin = lookfrom;
        let horizontal = focus_dist * viewport_width * u;
        let vertical = focus_dist * viewport_height * v;
        let lower_left_corner = origin - horizontal / 2 - vertical / 2 - focus_dist * w;

        Camera {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
            u,
            v,
            w,
            lens_radius: aperture / 2.0,
            time1,
            time2,
        }
    }

    pub fn get_ray(&self, s: f64, t: f64) -> Ray {
        let mut rng = thread_rng();
        let rd = self.lens_radius * random_in_unit_disk();
        let offset = self.u * rd.get_x() + self.v * rd.get_y();

        Ray::new(
            &(self.origin + offset),
            &(self.lower_left_corner + s * self.horizontal + t * self.vertical
                - self.origin
                - offset),
            rng.gen_range(self.time1..self.time2),
        )
    }
}
