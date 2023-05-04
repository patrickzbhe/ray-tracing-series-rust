pub mod vec3 {
    use crate::mutil::mutil::clamp;
    use rand::{thread_rng, Rng};
    use std::{fmt, ops};

    #[derive(Debug, Clone, Copy)]
    pub struct Vec3(f64, f64, f64);
    pub type point3 = Vec3;
    pub type color = Vec3;

    const COLOR_MAX: f64 = 256.0;

    impl Vec3 {
        pub fn x(&self) -> f64 {
            return self.0;
        }
        pub fn y(&self) -> f64 {
            return self.1;
        }
        pub fn z(&self) -> f64 {
            return self.2;
        }

        pub fn length_squared(&self) -> f64 {
            self.dot(self)
        }

        pub fn length(&self) -> f64 {
            f64::sqrt(self.length_squared())
        }

        pub fn dot(&self, &other: &Vec3) -> f64 {
            self.x() * other.x() + self.y() * other.y() + self.z() * other.z()
        }

        pub fn cross(&self, &other: &Vec3) -> Vec3 {
            Vec3::new(
                self.y() * other.z() - self.z() * other.y(),
                self.z() * other.x() - self.x() * other.z(),
                self.x() * other.y() - self.y() * other.x(),
            )
        }

        pub fn unit(&self) -> Vec3 {
            *self / self.length()
        }

        pub fn near_zero(&self) -> bool {
            let s = 1e-8;
            f64::abs(self.x()) < s && f64::abs(self.y()) < s && f64::abs(self.z()) < s
        }

        pub fn reflect(&self, normal: &Vec3) -> Vec3 {
            *self - 2.0 * self.dot(normal) * *normal
        }

        pub fn write_color(&self, samples_per_pixel: u32) {
            // TODO: take output stream as param
            let mut r = self.x();
            let mut g = self.y();
            let mut b = self.z();

            let scale = 1.0 / samples_per_pixel as f64;
            r *= scale;
            g *= scale;
            b *= scale;
            r = f64::sqrt(r);
            g = f64::sqrt(g);
            b = f64::sqrt(b);
            println!(
                "{} {} {}",
                (COLOR_MAX * clamp(r, 0.0, 1.0)) as i32,
                (COLOR_MAX * clamp(g, 0.0, 1.0)) as i32,
                (COLOR_MAX * clamp(b, 0.0, 1.0)) as i32
            );
        }

        pub fn refract(uv: &Vec3, n: &Vec3, etai_over_etat: f64) -> Vec3 {
            let cos_theta = f64::min((-*uv).dot(n), 1.0);
            let r_out_perp = etai_over_etat * (*uv + cos_theta * *n);
            let r_out_parallel = -(f64::sqrt(f64::abs(1.0 - r_out_perp.length_squared()))) * *n;
            r_out_perp + r_out_parallel
        }
    }

    impl Vec3 {
        pub fn new<T, U, V>(x: T, y: U, z: V) -> Vec3
        where
            T: Into<f64>,
            U: Into<f64>,
            V: Into<f64>,
        {
            Vec3(x.into(), y.into(), z.into())
        }
    }

    impl ops::Mul for Vec3 {
        type Output = Vec3;

        fn mul(self, other: Vec3) -> Vec3 {
            Vec3(
                self.x() * other.x(),
                self.y() * other.y(),
                self.z() * other.z(),
            )
        }
    }

    impl ops::Neg for Vec3 {
        type Output = Self;

        fn neg(self) -> Vec3 {
            Vec3::new(
                self.x() * -1 as f64,
                self.y() * -1 as f64,
                self.z() * -1 as f64,
            )
        }
    }

    impl<T: Into<f64> + Copy> ops::Mul<T> for Vec3 {
        type Output = Vec3;

        fn mul(self, other: T) -> Vec3 {
            Vec3(
                self.x() * other.into(),
                self.y() * other.into(),
                self.z() * other.into(),
            )
        }
    }

    impl ops::Mul<Vec3> for f64 {
        type Output = Vec3;

        fn mul(self, other: Vec3) -> Vec3 {
            Vec3(self * other.x(), self * other.y(), self * other.z())
        }
    }

    impl<T: Into<f64> + Copy> ops::Div<T> for Vec3 {
        type Output = Vec3;

        fn div(self, other: T) -> Vec3 {
            Vec3(
                self.x() / other.into(),
                self.y() / other.into(),
                self.z() / other.into(),
            )
        }
    }

    impl ops::Add for Vec3 {
        type Output = Vec3;

        fn add(self, other: Vec3) -> Vec3 {
            Vec3(
                self.x() + other.x(),
                self.y() + other.y(),
                self.z() + other.z(),
            )
        }
    }

    impl ops::Sub for Vec3 {
        type Output = Vec3;

        fn sub(self, other: Vec3) -> Vec3 {
            Vec3(
                self.x() - other.x(),
                self.y() - other.y(),
                self.z() - other.z(),
            )
        }
    }

    impl ops::AddAssign for Vec3 {
        fn add_assign(&mut self, other: Vec3) {
            self.0 += other.x();
            self.1 += other.y();
            self.2 += other.z();
        }
    }

    impl<T: Into<f64> + Copy> ops::MulAssign<T> for Vec3 {
        fn mul_assign(&mut self, rhs: T) {
            self.0 *= rhs.into();
            self.1 *= rhs.into();
            self.2 *= rhs.into();
        }
    }

    impl<T: Into<f64> + Copy> ops::DivAssign<T> for Vec3 {
        fn div_assign(&mut self, rhs: T) {
            *self *= 1 as f64 / rhs.into();
        }
    }

    impl ops::MulAssign for Vec3 {
        fn mul_assign(&mut self, other: Vec3) {
            self.0 *= other.x();
            self.1 *= other.y();
            self.2 *= other.z();
        }
    }

    impl PartialEq for Vec3 {
        fn eq(&self, other: &Self) -> bool {
            self.x() == other.x() && self.y() == other.y() && self.z() == other.z()
        }
    }

    impl fmt::Display for Vec3 {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Vec3 ({} {} {})", self.x(), self.y(), self.z())
        }
    }

    pub fn random() -> Vec3 {
        let mut rng = thread_rng();
        Vec3::new(rng.gen::<f64>(), rng.gen::<f64>(), rng.gen::<f64>())
    }

    pub fn random_range(min: f64, max: f64) -> Vec3 {
        let mut rng = thread_rng();
        Vec3::new(
            rng.gen_range::<f64, ops::Range<f64>>(min..max),
            rng.gen_range::<f64, ops::Range<f64>>(min..max),
            rng.gen_range::<f64, ops::Range<f64>>(min..max),
        )
    }

    pub fn random_in_unit_sphere() -> Vec3 {
        loop {
            let p = random_range(-1.0, 1.0);

            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }

    pub fn random_unit_vector() -> Vec3 {
        random_in_unit_sphere().unit()
    }

    pub fn random_in_hemisphere(normal: &Vec3) -> Vec3 {
        let in_unit_sphere = random_in_unit_sphere();
        if (in_unit_sphere.dot(normal)) > 0.0 {
            in_unit_sphere
        } else {
            -in_unit_sphere
        }
    }

    pub fn random_in_unit_disk() -> Vec3 {
        let mut rng = thread_rng();
        loop {
            let p = Vec3::new(
                rng.gen_range::<f64, ops::Range<f64>>(-1.0..1.0),
                rng.gen_range::<f64, ops::Range<f64>>(-1.0..1.0),
                0,
            );
            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn general_vec3_stuff() {
        let v1 = vec3::Vec3::new(2, 2, 1);
        assert_eq!(v1.length_squared(), 9 as f64);
        assert_eq!(v1.length(), 3 as f64);
        assert_eq!(v1.x(), 2 as f64);
        assert_eq!(v1.y(), 2 as f64);
        assert_eq!(v1.z(), 1 as f64);

        let v2 = vec3::Vec3::new(5, 7, 4.1);

        assert_eq!(v2.x(), 5 as f64);
        assert_eq!(v2.y(), 7 as f64);
        assert_eq!(v2.z(), 4.1 as f64);
        assert_eq!(v1 + v2, vec3::Vec3::new(7, 9, 5.1));
        assert_eq!(-v2, vec3::Vec3::new(-5, -7, -4.1));
    }

    #[test]
    fn add_assign() {
        let mut v1 = vec3::Vec3::new(3, 2, 1);
        v1 += v1;
        assert_eq!(v1, vec3::Vec3::new(6, 4, 2));
    }

    #[test]
    fn mul_assign() {
        let mut v1 = vec3::Vec3::new(3, 2, 1);
        v1 *= 5;
        assert_eq!(v1, vec3::Vec3::new(15, 10, 5));
        v1 *= 2.5;
        assert_eq!(v1, vec3::Vec3::new(37.5, 25, 12.5));
    }

    #[test]
    fn div_assign() {
        let mut v1 = vec3::Vec3::new(27, 9, 3);
        v1 /= 3;
        assert_eq!(v1, vec3::Vec3::new(9, 3, 1));
    }

    #[test]
    fn f64_mul() {
        let v1 = vec3::Vec3::new(5, 10, 15);
        assert_eq!(0.5 * v1, vec3::Vec3::new(2.5, 5, 7.5))
    }

    #[test]
    fn div_f64() {
        let v1 = vec3::Vec3::new(5, 10, 15);
        assert_eq!(v1 / 5, vec3::Vec3::new(1, 2, 3))
    }

    #[test]
    fn cross() {
        assert_eq!(
            vec3::Vec3::new(-3, 6, -3),
            vec3::Vec3::new(2, 3, 4).cross(&vec3::Vec3::new(5, 6, 7))
        )
    }
}
