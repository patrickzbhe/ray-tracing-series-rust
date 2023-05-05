use crate::aabb::Aabb;
use crate::ray::Ray;
use crate::texture::{SolidColor, Texture};
use crate::vec3::{random_in_unit_sphere, random_unit_vector, Color, Point3, Vec3};
use rand::{thread_rng, Rng};
use std::f64::consts::PI;
use std::sync::Arc;

#[derive(Clone)]
pub struct HitRecord {
    p: Point3,
    normal: Vec3,
    t: f64,
    u: f64,
    v: f64,
    front_face: bool,
    mat_ptr: Arc<Box<dyn Material>>,
}

impl HitRecord {
    pub fn new(
        p: Point3,
        normal: Vec3,
        t: f64,
        u: f64,
        v: f64,
        front_face: bool,
        material: Arc<Box<dyn Material>>,
    ) -> HitRecord {
        HitRecord {
            p,
            normal,
            t,
            u,
            v,
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

    pub fn get_u(&self) -> f64 {
        return self.u;
    }

    pub fn get_v(&self) -> f64 {
        return self.v;
    }

    pub fn get_front_face(&self) -> bool {
        return self.front_face;
    }

    pub fn get_material(&self) -> Arc<Box<dyn Material>> {
        Arc::clone(&self.mat_ptr)
    }

    fn create_normal_face(r: &Ray, outward_normal: &Vec3) -> (Vec3, bool) {
        let front_face = r.direction().dot(outward_normal) < 0.0;
        (
            if front_face {
                *outward_normal
            } else {
                -*outward_normal
            },
            front_face,
        )
    }
}

pub trait Hittable: Send + Sync {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb>;
}

pub struct Sphere {
    center: Point3,
    radius: f64,
    mat_ptr: Arc<Box<dyn Material>>,
}

impl Sphere {
    pub fn new(center: Point3, radius: f64, mat_ptr: Arc<Box<dyn Material>>) -> Sphere {
        Sphere {
            center,
            radius,
            mat_ptr,
        }
    }

    pub fn get_sphere_uv(p: &Point3) -> (f64, f64) {
        //  4.2 ray tracing next week math
        let theta = f64::acos(-p.y());
        let phi = f64::atan2(-p.z(), p.x()) + PI;
        (phi / (2.0 * PI), theta / PI)
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = *r.get_origin() - self.center;
        let a = r.direction().length_squared();
        let half_b = oc.dot(r.direction());
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }
        let sqrtd = f64::sqrt(discriminant);

        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }
        let t = root;
        let p = r.at(t);
        let outward_normal = (p - self.center) / self.radius;
        let (normal, front_face) = HitRecord::create_normal_face(r, &outward_normal);
        let (u, v) = Sphere::get_sphere_uv(&outward_normal);

        Some(HitRecord::new(
            p,
            normal,
            t,
            u,
            v,
            front_face,
            Arc::clone(&self.mat_ptr),
        ))
        // TODO return an option here?
    }
    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
        Some(Aabb::new(
            self.center - Point3::new(self.radius, self.radius, self.radius),
            self.center + Point3::new(self.radius, self.radius, self.radius),
        ))
    }
}

pub struct MovingSphere {
    center0: Point3,
    center1: Point3,
    time0: f64,
    time1: f64,
    radius: f64,
    mat_ptr: Arc<Box<dyn Material>>,
}

impl MovingSphere {
    pub fn new(
        center0: Point3,
        center1: Point3,
        time0: f64,
        time1: f64,
        radius: f64,
        mat_ptr: Arc<Box<dyn Material>>,
    ) -> MovingSphere {
        MovingSphere {
            center0,
            center1,
            time0,
            time1,
            radius,
            mat_ptr,
        }
    }

    pub fn get_center(&self, time: f64) -> Point3 {
        return self.center0
            + ((time - self.time0) / (self.time1 - self.time0)) * (self.center1 - self.center0);
    }
}

impl Hittable for MovingSphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let cur_time = self.get_center(r.get_time());
        let oc = *r.get_origin() - cur_time;
        let a = r.direction().length_squared();
        let half_b = oc.dot(r.direction());
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }
        let sqrtd = f64::sqrt(discriminant);

        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }
        let t = root;
        let p = r.at(t);
        let outward_normal = (p - cur_time) / self.radius;
        let (normal, front_face) = HitRecord::create_normal_face(r, &outward_normal);

        Some(HitRecord::new(
            p,
            normal,
            t,
            0.0,
            0.0,
            front_face,
            Arc::clone(&self.mat_ptr),
        ))
        // TODO return an option here?
    }
    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
        let box0 = Aabb::new(
            self.get_center(time0) - Point3::new(self.radius, self.radius, self.radius),
            self.get_center(time0) + Point3::new(self.radius, self.radius, self.radius),
        );
        let box1 = Aabb::new(
            self.get_center(time1) - Point3::new(self.radius, self.radius, self.radius),
            self.get_center(time1) + Point3::new(self.radius, self.radius, self.radius),
        );
        Some(Aabb::surrounding_box(&box0, &box1))
    }
}

pub struct XyRect {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    k: f64,
    mat_ptr: Arc<Box<dyn Material>>,
}

impl XyRect {
    pub fn new(
        x0: f64,
        x1: f64,
        y0: f64,
        y1: f64,
        k: f64,
        mat_ptr: Arc<Box<dyn Material>>,
    ) -> XyRect {
        XyRect {
            x0,
            x1,
            y0,
            y1,
            k,
            mat_ptr,
        }
    }
}

impl Hittable for XyRect {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let t = (self.k - r.get_origin().z()) / r.direction().z();
        if t < t_min || t > t_max {
            return None;
        }
        let x = r.get_origin().x() + t * r.direction().x();
        let y = r.get_origin().y() + t * r.direction().y();
        if x < self.x0 || x > self.x1 || y < self.y0 || y > self.y1 {
            return None;
        }
        let u = (x - self.x0) / (self.x1 - self.x0);
        let v = (y - self.y0) / (self.y1 - self.y0);
        let outward_normal = Vec3::new(0, 0, 1);
        let (normal, front) = HitRecord::create_normal_face(r, &outward_normal);

        let p = r.at(t);
        Some(HitRecord::new(
            p,
            normal,
            t,
            u,
            v,
            front,
            self.mat_ptr.clone(),
        ))
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
        Some(Aabb::new(
            Point3::new(self.x0, self.y0, self.k - 0.0001),
            Point3::new(self.x1, self.y1, self.k + 0.0001),
        ))
    }
}

pub struct XzRect {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    k: f64,
    mat_ptr: Arc<Box<dyn Material>>,
}

impl XzRect {
    pub fn new(
        x0: f64,
        x1: f64,
        y0: f64,
        y1: f64,
        k: f64,
        mat_ptr: Arc<Box<dyn Material>>,
    ) -> XzRect {
        XzRect {
            x0,
            x1,
            y0,
            y1,
            k,
            mat_ptr,
        }
    }
}

impl Hittable for XzRect {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let t = (self.k - r.get_origin().y()) / r.direction().y();
        if t < t_min || t > t_max {
            return None;
        }
        let x = r.get_origin().x() + t * r.direction().x();
        let y = r.get_origin().z() + t * r.direction().z();
        if x < self.x0 || x > self.x1 || y < self.y0 || y > self.y1 {
            return None;
        }
        let u = (x - self.x0) / (self.x1 - self.x0);
        let v = (y - self.y0) / (self.y1 - self.y0);
        let outward_normal = Vec3::new(0, 1, 0);
        let (normal, front) = HitRecord::create_normal_face(r, &outward_normal);

        let p = r.at(t);
        Some(HitRecord::new(
            p,
            normal,
            t,
            u,
            v,
            front,
            self.mat_ptr.clone(),
        ))
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
        Some(Aabb::new(
            Point3::new(self.x0, self.k - 0.0001, self.y0, ),
            Point3::new(self.x1, self.k + 0.0001, self.y1, ),
        ))
    }
}

pub struct YzRect {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    k: f64,
    mat_ptr: Arc<Box<dyn Material>>,
}

impl YzRect {
    pub fn new(
        x0: f64,
        x1: f64,
        y0: f64,
        y1: f64,
        k: f64,
        mat_ptr: Arc<Box<dyn Material>>,
    ) -> YzRect {
        YzRect {
            x0,
            x1,
            y0,
            y1,
            k,
            mat_ptr,
        }
    }
}

impl Hittable for YzRect {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let t = (self.k - r.get_origin().x()) / r.direction().x();
        if t < t_min || t > t_max {
            return None;
        }
        let x = r.get_origin().y() + t * r.direction().y();
        let y = r.get_origin().z() + t * r.direction().z();
        if x < self.x0 || x > self.x1 || y < self.y0 || y > self.y1 {
            return None;
        }
        let u = (x - self.x0) / (self.x1 - self.x0);
        let v = (y - self.y0) / (self.y1 - self.y0);
        let outward_normal = Vec3::new(1,0,0);
        let (normal, front) = HitRecord::create_normal_face(r, &outward_normal);

        let p = r.at(t);
        Some(HitRecord::new(
            p,
            normal,
            t,
            u,
            v,
            front,
            self.mat_ptr.clone(),
        ))
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
        Some(Aabb::new(
            Point3::new(self.k - 0.0001, self.x0, self.y0, ),
            Point3::new(self.k + 0.0001, self.x1, self.y1, ),
        ))
    }
}

pub struct HittableList {
    objects: Vec<Arc<Box<dyn Hittable + Sync>>>,
}

impl HittableList {
    pub fn new() -> HittableList {
        HittableList { objects: vec![] }
    }

    pub fn add(&mut self, object: Arc<Box<dyn Hittable + Sync>>) {
        self.objects.push(Arc::clone(&object));
    }

    pub fn get_objects(&self) -> &Vec<Arc<Box<dyn Hittable + Sync>>> {
        &self.objects
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut hit_anything = false;
        let mut closest_so_far = t_max;

        let temp_mat: Arc<Box<dyn Material>> =
            Arc::new(Box::new(Metal::new(Vec3::new(0, 0, 0), 0.0)));
        let mut temp_rec = HitRecord::new(
            Vec3::new(0, 0, 0),
            Vec3::new(0, 0, 0),
            0.0,
            0.0,
            0.0,
            false,
            temp_mat,
        );
        for object in self.objects.iter() {
            match object.hit(&r, t_min, closest_so_far) {
                Some(rec) => {
                    hit_anything = true;
                    closest_so_far = rec.t;
                    temp_rec = rec.clone();
                }
                None => (),
            }
        }
        if hit_anything {
            Some(temp_rec)
        } else {
            None
        }
    }
    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
        if self.objects.is_empty() {
            return None;
        }
        let mut obj_iter = self.objects.iter();
        let first = obj_iter.next().unwrap();
        let temp_box = first.bounding_box(time0, time1);
        let mut temp_box = match temp_box {
            Some(a) => a,
            None => return None,
        };
        for obj in obj_iter {
            let other_box = match obj.bounding_box(time0, time1) {
                Some(a) => a,
                None => return None,
            };
            temp_box = Aabb::surrounding_box(&temp_box, &other_box);
        }
        Some(temp_box)
    }
}

pub trait Material: Send + Sync {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)>;
    fn emitted(&self, u: f64, v: f64, p: &Point3) -> Color {
        Color::new(0, 0, 0)
    }
}

pub struct Lambertian {
    albedo: Arc<Box<dyn Texture>>,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Lambertian {
        Lambertian {
            albedo: Arc::new(Box::new(SolidColor::new(&albedo))),
        }
    }

    pub fn from_pointer(texture: Arc<Box<dyn Texture>>) -> Lambertian {
        Lambertian {
            albedo: texture.clone(),
        }
    }
}

impl Material for Lambertian {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        let mut scatter_direction = *rec.get_normal() + random_unit_vector();

        // catch degenerate scatter directions
        if scatter_direction.near_zero() {
            scatter_direction = *rec.get_normal();
        }

        Some((
            Ray::new(rec.get_p(), &scatter_direction, r_in.get_time()),
            self.albedo.value(rec.u, rec.v, &rec.p),
        ))
    }
}

pub struct Metal {
    albedo: Color,
    fuzz: f64,
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Metal {
        Metal {
            albedo,
            fuzz: if fuzz < 1.0 { fuzz } else { 1.0 },
        }
    }
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        let reflected = r_in.direction().unit().reflect(rec.get_normal());

        let scattered = Ray::new(
            rec.get_p(),
            &(reflected + self.fuzz * random_in_unit_sphere()),
            r_in.get_time(),
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
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
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

        Some((Ray::new(&rec.p, &direction, r_in.get_time()), attenuation))
    }
}

pub struct DiffuseLight {
    emit: Arc<Box<dyn Texture>>,
}

impl DiffuseLight {
    pub fn new(c: &Color) -> DiffuseLight {
        DiffuseLight {
            emit: Arc::new(Box::new(SolidColor::new(c))),
        }
    }

    pub fn from_pointer(a: Arc<Box<dyn Texture>>) -> DiffuseLight {
        DiffuseLight { emit: a.clone() }
    }
}

impl Material for DiffuseLight {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        None
    }
    fn emitted(&self, u: f64, v: f64, p: &Point3) -> Color {
        self.emit.value(u, v, p)
    }
}
