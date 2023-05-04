pub mod camera {
    use crate::ray::ray::Ray;
    use crate::vec3::vec3::{point3, random_in_unit_disk, Vec3};

    pub struct Camera {
        origin: point3,
        lower_left_corner: point3,
        horizontal: Vec3,
        vertical: Vec3,
        u: Vec3,
        v: Vec3,
        w: Vec3,
        lens_radius: f64,
    }

    impl Camera {
        // TODO: fix this
        // pub fn default() -> Camera {
        //     let aspect_ratio = 16.0 / 9.0;
        //     let viewport_height = 2.0;
        //     let viewport_width = aspect_ratio * viewport_height;
        //     let focal_length = 1.0;

        //     let origin = Vec3::new(0, 0, 0);
        //     let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
        //     let vertical = Vec3::new(0, viewport_height, 0);
        //     let lower_left_corner =
        //         origin - horizontal / 2 - vertical / 2 - Vec3::new(0, 0, focal_length);
        //     Camera { origin, lower_left_corner, horizontal, vertical }
        // }

        pub fn new(
            lookfrom: point3,
            lookat: point3,
            vup: Vec3,
            vfov: f64,
            aspect_ratio: f64,
            aperture: f64,
            focus_dist: f64,
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
            }
        }

        pub fn get_ray(&self, s: f64, t: f64) -> Ray {
            let rd = self.lens_radius * random_in_unit_disk();
            let offset = self.u * rd.x() + self.v * rd.y();

            Ray::new(
                &(self.origin + offset),
                &(self.lower_left_corner + s * self.horizontal + t * self.vertical
                    - self.origin
                    - offset),
            )
        }
    }
}
