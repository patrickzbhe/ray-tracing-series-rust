pub mod ray {
    use crate::vec3::vec3::{point3, Vec3};

    #[derive(Debug, Clone, Copy)]
    pub struct Ray {
        origin: point3,
        direction: Vec3,
    }

    impl Ray {
        pub fn new(&origin: &point3, &direction: &Vec3) -> Ray {
            Ray {
                origin: origin.clone(),
                direction: direction.clone(),
            }
        }

        pub fn origin(&self) -> &point3 {
            &self.origin
        }

        pub fn direction(&self) -> &Vec3 {
            &self.direction
        }

        pub fn at(&self, t: f64) -> point3 {
            self.origin.clone() + self.direction * t
        }
    }
}
