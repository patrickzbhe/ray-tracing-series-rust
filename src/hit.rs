use crate::aabb::Aabb;
use crate::bvh::BvhNode;
use crate::ray::Ray;
use crate::texture::{SolidColor, Texture, TextureWrapper};
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
    mat_ptr: Arc<MaterialWrapper>,
}

impl HitRecord {
    pub fn new(
        p: Point3,
        normal: Vec3,
        t: f64,
        u: f64,
        v: f64,
        front_face: bool,
        material: Arc<MaterialWrapper>,
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

    pub fn get_material(&self) -> Arc<MaterialWrapper> {
        Arc::clone(&self.mat_ptr)
    }

    fn create_normal_face(r: &Ray, outward_normal: &Vec3) -> (Vec3, bool) {
        let front_face = r.get_direction().dot(outward_normal) < 0.0;
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

pub enum HittableWrapper {
    Sphere(Sphere),
    MovingSphere(MovingSphere),
    GravitySphere(GravitySphere),
    XyRect(XyRect),
    XzRect(XzRect),
    YzRect(YzRect),
    HittableList(HittableList),
    RectPrism(RectPrism),
    Translate(Translate),
    RotateY(RotateY),
    ConstantMedium(ConstantMedium),
    BvhNode(BvhNode),
}

impl Hittable for HittableWrapper {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        match &self {
            HittableWrapper::Sphere(obj) => obj.hit(r, t_min, t_max),
            HittableWrapper::MovingSphere(obj) => obj.hit(r, t_min, t_max),
            HittableWrapper::GravitySphere(obj) => obj.hit(r, t_min, t_max),
            HittableWrapper::XyRect(obj) => obj.hit(r, t_min, t_max),
            HittableWrapper::XzRect(obj) => obj.hit(r, t_min, t_max),
            HittableWrapper::YzRect(obj) => obj.hit(r, t_min, t_max),
            HittableWrapper::HittableList(obj) => obj.hit(r, t_min, t_max),
            HittableWrapper::RectPrism(obj) => obj.hit(r, t_min, t_max),
            HittableWrapper::Translate(obj) => obj.hit(r, t_min, t_max),
            HittableWrapper::RotateY(obj) => obj.hit(r, t_min, t_max),
            HittableWrapper::ConstantMedium(obj) => obj.hit(r, t_min, t_max),
            HittableWrapper::BvhNode(obj) => obj.hit(r, t_min, t_max),
        }
    }
    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
        match self {
            HittableWrapper::Sphere(obj) => obj.bounding_box(time0, time1),
            HittableWrapper::MovingSphere(obj) => obj.bounding_box(time0, time1),
            HittableWrapper::GravitySphere(obj) => obj.bounding_box(time0, time1),
            HittableWrapper::XyRect(obj) => obj.bounding_box(time0, time1),
            HittableWrapper::XzRect(obj) => obj.bounding_box(time0, time1),
            HittableWrapper::YzRect(obj) => obj.bounding_box(time0, time1),
            HittableWrapper::HittableList(obj) => obj.bounding_box(time0, time1),
            HittableWrapper::RectPrism(obj) => obj.bounding_box(time0, time1),
            HittableWrapper::Translate(obj) => obj.bounding_box(time0, time1),
            HittableWrapper::RotateY(obj) => obj.bounding_box(time0, time1),
            HittableWrapper::ConstantMedium(obj) => obj.bounding_box(time0, time1),
            HittableWrapper::BvhNode(obj) => obj.bounding_box(time0, time1),
        }
    }
}

#[derive(Clone)]
pub struct Sphere {
    center: Point3,
    radius: f64,
    mat_ptr: Arc<MaterialWrapper>,
}

impl Sphere {
    pub fn new(center: Point3, radius: f64, mat_ptr: Arc<MaterialWrapper>) -> Sphere {
        Sphere {
            center,
            radius,
            mat_ptr,
        }
    }

    pub fn get_sphere_uv(p: &Point3) -> (f64, f64) {
        //  4.2 ray tracing next week math
        let theta = f64::acos(-p.get_y());
        let phi = f64::atan2(-p.get_z(), p.get_x()) + PI;
        (phi / (2.0 * PI), theta / PI)
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = *r.get_origin() - self.center;
        let a = r.get_direction().length_squared();
        let half_b = oc.dot(r.get_direction());
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

#[derive(Clone)]
pub struct MovingSphere {
    center0: Point3,
    center1: Point3,
    time0: f64,
    time1: f64,
    radius: f64,
    mat_ptr: Arc<MaterialWrapper>,
}

impl MovingSphere {
    pub fn new(
        center0: Point3,
        center1: Point3,
        time0: f64,
        time1: f64,
        radius: f64,
        mat_ptr: Arc<MaterialWrapper>,
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
        let a = r.get_direction().length_squared();
        let half_b = oc.dot(r.get_direction());
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

#[derive(Clone)]
pub struct GravitySphere {
    start: Point3,
    time0: f64,
    radius: f64,
    mat_ptr: Arc<MaterialWrapper>,
    pub stored: Vec<f64>,
}

// Fix this up later
impl GravitySphere {
    pub fn new(
        start: Point3,
        time0: f64,
        radius: f64,
        mat_ptr: Arc<MaterialWrapper>,
    ) -> GravitySphere {
        let mut stored = vec![start.get_y()];
        let incr = 0.001;
        let mut t = time0;
        let mut cur_pos = start;
        let mut vel = 0.0;
        while t < 100.0 {
            t += incr;
            vel -= 0.000001;
            if cur_pos.get_y() - 1.0 * radius <= 0.0 {
                vel *= -0.92;
            }
            cur_pos.set_y(f64::max(1.0 * radius, cur_pos.get_y() + vel));
            stored.push(cur_pos.get_y());
        }
        let output = GravitySphere {
            start,
            time0,
            radius,
            mat_ptr,
            stored: stored,
        };
        output
    }

    pub fn get_center(&self, time: f64) -> Point3 {
        let incr = 0.001;
        // brute force lmao
        if (time / incr) as usize + 1 <= self.stored.len() {
            return Vec3::new(
                self.start.get_x(),
                self.stored[(time / incr) as usize],
                self.start.get_z(),
            );
        }
        // TODO: figure out radius x2 bug?
        let mut t = self.time0;
        let mut cur_pos = self.start.clone();
        let mut vel = 0.0;
        while t < time {
            t += incr;
            vel -= 0.000001;
            if cur_pos.get_y() - 2.0 * self.radius <= 0.0 {
                vel *= -0.8;
            }
            cur_pos.set_y(f64::max(2.0 * self.radius, cur_pos.get_y() + vel));
        }

        return cur_pos;
    }
}

impl Hittable for GravitySphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let cur_time = self.get_center(r.get_time());
        let oc = *r.get_origin() - cur_time;
        let a = r.get_direction().length_squared();
        let half_b = oc.dot(r.get_direction());
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

#[derive(Clone)]
pub struct XyRect {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    k: f64,
    mat_ptr: Arc<MaterialWrapper>,
}

impl XyRect {
    pub fn new(
        x0: f64,
        x1: f64,
        y0: f64,
        y1: f64,
        k: f64,
        mat_ptr: Arc<MaterialWrapper>,
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
        let t = (self.k - r.get_origin().get_z()) / r.get_direction().get_z();
        if t < t_min || t > t_max {
            return None;
        }
        let x = r.get_origin().get_x() + t * r.get_direction().get_x();
        let y = r.get_origin().get_y() + t * r.get_direction().get_y();
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

#[derive(Clone)]
pub struct XzRect {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    k: f64,
    mat_ptr: Arc<MaterialWrapper>,
}

impl XzRect {
    pub fn new(
        x0: f64,
        x1: f64,
        y0: f64,
        y1: f64,
        k: f64,
        mat_ptr: Arc<MaterialWrapper>,
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
        let t = (self.k - r.get_origin().get_y()) / r.get_direction().get_y();
        if t < t_min || t > t_max {
            return None;
        }
        let x = r.get_origin().get_x() + t * r.get_direction().get_x();
        let y = r.get_origin().get_z() + t * r.get_direction().get_z();
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
            Point3::new(self.x0, self.k - 0.0001, self.y0),
            Point3::new(self.x1, self.k + 0.0001, self.y1),
        ))
    }
}

#[derive(Clone)]
pub struct YzRect {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    k: f64,
    mat_ptr: Arc<MaterialWrapper>,
}

impl YzRect {
    pub fn new(
        x0: f64,
        x1: f64,
        y0: f64,
        y1: f64,
        k: f64,
        mat_ptr: Arc<MaterialWrapper>,
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
        let t = (self.k - r.get_origin().get_x()) / r.get_direction().get_x();
        if t < t_min || t > t_max {
            return None;
        }
        let x = r.get_origin().get_y() + t * r.get_direction().get_y();
        let y = r.get_origin().get_z() + t * r.get_direction().get_z();
        if x < self.x0 || x > self.x1 || y < self.y0 || y > self.y1 {
            return None;
        }
        let u = (x - self.x0) / (self.x1 - self.x0);
        let v = (y - self.y0) / (self.y1 - self.y0);
        let outward_normal = Vec3::new(1, 0, 0);
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
            Point3::new(self.k - 0.0001, self.x0, self.y0),
            Point3::new(self.k + 0.0001, self.x1, self.y1),
        ))
    }
}

#[derive(Clone)]
pub struct HittableList {
    objects: Vec<Arc<HittableWrapper>>,
}

impl HittableList {
    pub fn new() -> HittableList {
        HittableList { objects: vec![] }
    }

    pub fn add(&mut self, object: Arc<HittableWrapper>) {
        self.objects.push(Arc::clone(&object));
    }

    pub fn get_objects(&self) -> &Vec<Arc<HittableWrapper>> {
        &self.objects
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut hit_anything = false;
        let mut closest_so_far = t_max;

        let temp_mat: Arc<MaterialWrapper> =
            Arc::new(MaterialWrapper::Metal(Metal::new(Vec3::new(0, 0, 0), 0.0)));
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

#[derive(Clone)]
pub struct RectPrism {
    box_min: Point3,
    box_max: Point3,
    sides: HittableList,
}

impl RectPrism {
    pub fn new(p0: &Point3, p1: &Point3, mat: Arc<MaterialWrapper>) -> RectPrism {
        let mut sides = HittableList::new();
        sides.add(Arc::new(HittableWrapper::XyRect(XyRect::new(
            p0.get_x(),
            p1.get_x(),
            p0.get_y(),
            p1.get_y(),
            p1.get_z(),
            mat.clone(),
        ))));
        sides.add(Arc::new(HittableWrapper::XyRect(XyRect::new(
            p0.get_x(),
            p1.get_x(),
            p0.get_y(),
            p1.get_y(),
            p0.get_z(),
            mat.clone(),
        ))));
        sides.add(Arc::new(HittableWrapper::XzRect(XzRect::new(
            p0.get_x(),
            p1.get_x(),
            p0.get_z(),
            p1.get_z(),
            p1.get_y(),
            mat.clone(),
        ))));
        sides.add(Arc::new(HittableWrapper::XzRect(XzRect::new(
            p0.get_x(),
            p1.get_x(),
            p0.get_z(),
            p1.get_z(),
            p0.get_y(),
            mat.clone(),
        ))));
        sides.add(Arc::new(HittableWrapper::YzRect(YzRect::new(
            p0.get_y(),
            p1.get_y(),
            p0.get_z(),
            p1.get_z(),
            p1.get_x(),
            mat.clone(),
        ))));
        sides.add(Arc::new(HittableWrapper::YzRect(YzRect::new(
            p0.get_y(),
            p1.get_y(),
            p0.get_z(),
            p1.get_z(),
            p0.get_x(),
            mat.clone(),
        ))));
        RectPrism {
            box_min: p0.clone(),
            box_max: p1.clone(),
            sides,
        }
    }
}

impl Hittable for RectPrism {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        self.sides.hit(r, t_min, t_max)
    }
    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
        Some(Aabb::new(self.box_min, self.box_max))
    }
}

#[derive(Clone)]
pub struct Translate {
    obj: Arc<HittableWrapper>,
    offset: Vec3,
}

impl Translate {
    pub fn new(offset: &Vec3, obj: Arc<HittableWrapper>) -> Translate {
        Translate {
            obj: obj.clone(),
            offset: offset.clone(),
        }
    }
}

impl Hittable for Translate {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let moved_r = Ray::new(
            &(*r.get_origin() - self.offset),
            r.get_direction(),
            r.get_time(),
        );
        match self.obj.hit(&moved_r, t_min, t_max) {
            Some(rec) => {
                let (normal, front_face) = HitRecord::create_normal_face(&moved_r, &rec.normal);
                return Some(HitRecord {
                    p: *rec.get_p() + self.offset,
                    normal,
                    t: rec.get_t(),
                    u: rec.get_u(),
                    v: rec.get_v(),
                    front_face,
                    mat_ptr: rec.get_material().clone(),
                });
            }
            None => return None,
        }
    }
    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
        match self.obj.bounding_box(time0, time1) {
            Some(a) => Some(Aabb::new(
                *a.get_min() + self.offset,
                *a.get_max() + self.offset,
            )),
            None => None,
        }
    }
}

#[derive(Clone)]
pub struct RotateY {
    obj: Arc<HittableWrapper>,
    sin_theta: f64,
    cos_theta: f64,
    bbox: Option<Aabb>,
}

impl RotateY {
    pub fn new(angle: f64, obj: Arc<HittableWrapper>) -> RotateY {
        let angle = f64::to_radians(angle);
        let sin_theta = f64::sin(angle);
        let cos_theta = f64::cos(angle);
        // ?
        let bbox = obj.bounding_box(0.0, 1.0);
        if bbox.is_none() {
            return RotateY {
                obj,
                sin_theta,
                cos_theta,
                bbox: None,
            };
        }
        let bbox = bbox.unwrap();
        let mut min = Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut max = Point3::new(-f64::INFINITY, -f64::INFINITY, -f64::INFINITY);
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let i = i as f64;
                    let j = j as f64;
                    let k = k as f64;
                    let x = i * bbox.get_max().get_x() + (1.0 - i) * bbox.get_min().get_x();
                    let y = j * bbox.get_max().get_y() + (1.0 - j) * bbox.get_min().get_y();
                    let z = k * bbox.get_max().get_z() + (1.0 - k) * bbox.get_min().get_z();

                    let newx = cos_theta * x + sin_theta * z;
                    let newz = -sin_theta * x + cos_theta * z;
                    let tester = Vec3::new(newx, y, newz);
                    min.set_x(f64::min(min.get_x(), tester.get_x()));
                    max.set_x(f64::max(max.get_x(), tester.get_x()));
                    min.set_y(f64::min(min.get_y(), tester.get_y()));
                    max.set_y(f64::max(max.get_y(), tester.get_y()));
                    min.set_z(f64::min(min.get_z(), tester.get_z()));
                    max.set_z(f64::max(max.get_z(), tester.get_z()));
                }
            }
        }
        RotateY {
            obj,
            sin_theta,
            cos_theta,
            bbox: Some(bbox),
        }
    }
}

impl Hittable for RotateY {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let origin = Vec3::new(
            self.cos_theta * r.get_origin().get_x() - self.sin_theta * r.get_origin().get_z(),
            r.get_origin().get_y(),
            self.sin_theta * r.get_origin().get_x() + self.cos_theta * r.get_origin().get_z(),
        );
        let direction = Vec3::new(
            self.cos_theta * r.get_direction().get_x() - self.sin_theta * r.get_direction().get_z(),
            r.get_direction().get_y(),
            self.sin_theta * r.get_direction().get_x() + self.cos_theta * r.get_direction().get_z(),
        );

        let rotated_r = Ray::new(&origin, &direction, r.get_time());
        let rec = self.obj.hit(&rotated_r, t_min, t_max);
        if rec.is_none() {
            return None;
        }
        let rec = rec.unwrap();
        let p = Vec3::new(
            self.cos_theta * rec.get_p().get_x() + self.sin_theta * rec.get_p().get_z(),
            rec.get_p().get_y(),
            -self.sin_theta * rec.get_p().get_x() + self.cos_theta * rec.get_p().get_z(),
        );

        let normal = Vec3::new(
            self.cos_theta * rec.get_normal().get_x() + self.sin_theta * rec.get_normal().get_z(),
            rec.get_normal().get_y(),
            -self.sin_theta * rec.get_normal().get_x() + self.cos_theta * rec.get_normal().get_z(),
        );
        let (normal, front_face) = HitRecord::create_normal_face(&rotated_r, &normal);
        Some(HitRecord::new(
            p,
            normal,
            rec.get_t(),
            rec.get_u(),
            rec.get_v(),
            front_face,
            rec.get_material(),
        ))
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
        self.bbox.clone()
    }
}

#[derive(Clone)]
pub struct ConstantMedium {
    boundary: Arc<HittableWrapper>,
    phase_function: Arc<MaterialWrapper>,
    neg_inv_density: f64,
}

impl ConstantMedium {
    pub fn from_color(c: &Color, d: f64, b: Arc<HittableWrapper>) -> ConstantMedium {
        ConstantMedium {
            boundary: b.clone(),
            phase_function: Arc::new(MaterialWrapper::Isotropic(Isotropic::from_color(c))),
            neg_inv_density: -1.0 / d,
        }
    }
}

impl Hittable for ConstantMedium {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let rec1 = self.boundary.hit(r, -f64::INFINITY, f64::INFINITY)?;
        let rec2 = self.boundary.hit(r, rec1.get_t() + 0.0001, f64::INFINITY)?;

        let mut t1 = f64::max(rec1.get_t(), t_min);
        let t2 = f64::min(rec2.get_t(), t_max);
        if t1 >= t2 {
            return None;
        }
        if t1 < 0.0 {
            t1 = 0.0;
        }
        let ray_length = r.get_direction().length();
        let distance_inside_boundary = (t2 - t1) * ray_length;
        let hit_distance = self.neg_inv_density * f64::ln(thread_rng().gen());
        if hit_distance > distance_inside_boundary {
            return None;
        }
        let t = t1 + hit_distance / ray_length;
        let p = r.at(t);
        let normal = Vec3::new(0, 0, 0);
        let front_face = true;
        Some(HitRecord {
            p,
            normal,
            t,
            u: 0.0,
            v: 0.0,
            front_face,
            mat_ptr: self.phase_function.clone(),
        })
    }
    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
        self.boundary.bounding_box(time0, time1)
    }
}

#[derive(Clone)]
pub struct Isotropic {
    albedo: Arc<TextureWrapper>,
}

impl Isotropic {
    pub fn from_color(c: &Color) -> Isotropic {
        Isotropic {
            albedo: Arc::new(TextureWrapper::SolidColor(SolidColor::new(c))),
        }
    }
}

impl Material for Isotropic {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        Some((
            Ray::new(&rec.p, &random_in_unit_sphere(), r_in.get_time()),
            self.albedo.value(rec.get_u(), rec.get_v(), rec.get_p()),
        ))
    }
}

pub trait Material: Send + Sync {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)>;
    fn emitted(&self, _u: f64, _v: f64, _p: &Point3) -> Color {
        Color::new(0, 0, 0)
    }
}

pub enum MaterialWrapper {
    Isotropic(Isotropic),
    Lambertian(Lambertian),
    Metal(Metal),
    Dielectric(Dielectric),
    DiffuseLight(DiffuseLight),
}

impl Material for MaterialWrapper {
    fn emitted(&self, u: f64, v: f64, p: &Point3) -> Color {
        match self {
            MaterialWrapper::Isotropic(obj) => obj.emitted(u, v, p),
            MaterialWrapper::Lambertian(obj) => obj.emitted(u, v, p),
            MaterialWrapper::Metal(obj) => obj.emitted(u, v, p),
            MaterialWrapper::Dielectric(obj) => obj.emitted(u, v, p),
            MaterialWrapper::DiffuseLight(obj) => obj.emitted(u, v, p),
        }
    }
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        match self {
            MaterialWrapper::Isotropic(obj) => obj.scatter(r_in, rec),
            MaterialWrapper::Lambertian(obj) => obj.scatter(r_in, rec),
            MaterialWrapper::Metal(obj) => obj.scatter(r_in, rec),
            MaterialWrapper::Dielectric(obj) => obj.scatter(r_in, rec),
            MaterialWrapper::DiffuseLight(obj) => obj.scatter(r_in, rec),
        }
    }
}

#[derive(Clone)]
pub struct Lambertian {
    albedo: Arc<TextureWrapper>,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Lambertian {
        Lambertian {
            albedo: Arc::new(TextureWrapper::SolidColor(SolidColor::new(&albedo))),
        }
    }

    pub fn from_pointer(texture: Arc<TextureWrapper>) -> Lambertian {
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

#[derive(Clone)]
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
        let reflected = r_in.get_direction().unit().reflect(rec.get_normal());

        let scattered = Ray::new(
            rec.get_p(),
            &(reflected + self.fuzz * random_in_unit_sphere()),
            r_in.get_time(),
        );

        if scattered.get_direction().dot(&rec.normal) > 0.0 {
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
        let unit_direction = r_in.get_direction().unit();

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
    emit: Arc<TextureWrapper>,
}

impl DiffuseLight {
    pub fn new(c: &Color) -> DiffuseLight {
        DiffuseLight {
            emit: Arc::new(TextureWrapper::SolidColor(SolidColor::new(c))),
        }
    }

    pub fn from_pointer(a: Arc<TextureWrapper>) -> DiffuseLight {
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
