pub mod hit {
    use rand::{thread_rng, Rng};

    use crate::ray::ray::Ray;
    use crate::vec3::vec3::{color, point3, random_in_unit_sphere, random_unit_vector, Vec3};

    use std::rc::Rc;

    #[derive(Clone)]
    pub struct HitRecord {
        p: point3,
        normal: Vec3,
        t: f64,
        front_face: bool,
        mat_ptr: Rc<Box<dyn Material>>,
    }

    impl HitRecord {
        pub fn new(
            p: point3,
            normal: Vec3,
            t: f64,
            front_face: bool,
            material: Rc<Box<dyn Material>>,
        ) -> HitRecord {
            HitRecord {
                p,
                normal,
                t,
                front_face,
                mat_ptr: material,
            }
        }

        pub fn get_normal(&self) -> &Vec3 {
            return &self.normal;
        }

        pub fn get_p(&self) -> &Vec3 {
            return &self.p;
        }

        pub fn get_t(&self) -> f64 {
            return self.t;
        }

        pub fn get_front_face(&self) -> bool {
            return self.front_face;
        }

        pub fn get_material(&self) -> Rc<Box<dyn Material>> {
            Rc::clone(&self.mat_ptr)
        }

        fn set_face_normal(&mut self, r: &Ray, outward_normal: &Vec3) {
            self.front_face = r.direction().dot(outward_normal) < 0.0;
            self.normal = if self.front_face {
                outward_normal.clone()
            } else {
                -(outward_normal.clone())
            };
        }
    }

    pub trait Hittable {
        fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool;
    }

    pub struct Sphere {
        center: point3,
        radius: f64,
        mat_ptr: Rc<Box<dyn Material>>,
    }

    impl Sphere {
        pub fn new(center: point3, radius: f64, mat_ptr: Rc<Box<dyn Material>>) -> Sphere {
            Sphere {
                center,
                radius,
                mat_ptr,
            }
        }
    }

    impl Hittable for Sphere {
        fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool {
            let oc = *r.origin() - self.center;
            let a = r.direction().length_squared();
            let half_b = oc.dot(r.direction());
            let c = oc.length_squared() - self.radius * self.radius;
            let discriminant = half_b * half_b - a * c;
            if discriminant < 0.0 {
                return false;
            }
            let sqrtd = f64::sqrt(discriminant);

            let mut root = (-half_b - sqrtd) / a;
            if root < t_min || t_max < root {
                root = (-half_b + sqrtd) / a;
                if root < t_min || t_max < root {
                    return false;
                }
            }

            // TODO return an option here?
            rec.t = root;
            rec.p = r.at(rec.t);
            rec.mat_ptr = Rc::clone(&self.mat_ptr);

            // TODO: take ownership?
            let outward_normal = (rec.p - self.center) / self.radius;
            rec.set_face_normal(r, &outward_normal);

            true
        }
    }

    pub struct HittableList {
        objects: Vec<Rc<Box<dyn Hittable>>>,
    }

    impl HittableList {
        pub fn new() -> HittableList {
            HittableList { objects: vec![] }
        }

        pub fn add(&mut self, object: Rc<Box<dyn Hittable>>) {
            self.objects.push(Rc::clone(&object));
        }
    }

    impl Hittable for HittableList {
        fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool {
            let mut hit_anything = false;
            let mut closest_so_far = t_max;

            let temp_mat: Rc<Box<dyn Material>> =
                Rc::new(Box::new(Metal::new(Vec3::new(0, 0, 0), 0.0)));
            let mut temp_rec =
                HitRecord::new(Vec3::new(0, 0, 0), Vec3::new(0, 0, 0), 0.0, false, temp_mat);
            for object in self.objects.iter() {
                if object.hit(&r, t_min, closest_so_far, &mut (temp_rec)) {
                    hit_anything = true;
                    closest_so_far = temp_rec.clone().t;
                    *rec = temp_rec.clone();
                }
            }
            return hit_anything;
        }
    }

    pub trait Material {
        fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, color)>;
    }

    pub struct Lambertian {
        albedo: color,
    }

    impl Lambertian {
        pub fn new(albedo: color) -> Lambertian {
            Lambertian { albedo }
        }
    }

    impl Material for Lambertian {
        fn scatter(&self, _r_in: &Ray, rec: &HitRecord) -> Option<(Ray, color)> {
            let mut scatter_direction = *rec.get_normal() + random_unit_vector();

            // catch degenerate scatter directions
            if scatter_direction.near_zero() {
                scatter_direction = *rec.get_normal();
            }

            Some((
                Ray::new(rec.get_p(), &scatter_direction),
                self.albedo.clone(),
            ))
        }
    }

    pub struct Metal {
        albedo: color,
        fuzz: f64,
    }

    impl Metal {
        pub fn new(albedo: color, fuzz: f64) -> Metal {
            Metal {
                albedo,
                fuzz: if fuzz < 1.0 { fuzz } else { 1.0 },
            }
        }
    }

    impl Material for Metal {
        fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, color)> {
            let reflected = r_in.direction().unit().reflect(rec.get_normal());

            let scattered = Ray::new(
                rec.get_p(),
                &(reflected + self.fuzz * random_in_unit_sphere()),
            );

            if scattered.direction().dot(&rec.normal) > 0.0 {
                Some((scattered, self.albedo.clone()))
            } else {
                None
            }
        }
    }

    pub struct Dielectric {
        ir: f64,
    }

    impl Dielectric {
        pub fn new(ir: f64) -> Dielectric {
            Dielectric { ir }
        }

        fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
            let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
            let r0 = r0 * r0;
            r0 + (1.0 - r0) * f64::powi(1.0 - cosine, 5)
        }
    }

    impl Material for Dielectric {
        fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, color)> {
            let mut rng = thread_rng();
            let attenuation = Vec3::new(1, 1, 1);
            let refraction_ratio = if rec.get_front_face() {
                1.0 / self.ir
            } else {
                self.ir
            };
            let unit_direction = r_in.direction().unit();

            let cos_theta = f64::min((-unit_direction).dot(&rec.normal), 1.0);
            let sin_theta = f64::sqrt(1.0 - cos_theta * cos_theta);

            let cannot_refract = refraction_ratio * sin_theta > 1.0;
            let direction = if cannot_refract
                || Dielectric::reflectance(cos_theta, refraction_ratio) > rng.gen::<f64>()
            {
                unit_direction.reflect(&rec.normal)
            } else {
                Vec3::refract(&unit_direction, &rec.get_normal(), refraction_ratio)
            };

            Some((Ray::new(&rec.p, &direction), attenuation))
        }
    }
}
