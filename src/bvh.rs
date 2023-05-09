use crate::aabb::Aabb;
use crate::hit::{HitRecord, Hittable, HittableList, HittableWrapper};
use rand::{thread_rng, Rng};
use std::cmp::Ordering;
use std::sync::Arc;

#[derive(Clone)]
pub struct BvhNode {
    left: Arc<HittableWrapper>,
    right: Arc<HittableWrapper>,
    bbox: Aabb,
}

impl BvhNode {
    pub fn new(
        src_objects: &Vec<Arc<HittableWrapper>>,
        start: usize,
        end: usize,
        time0: f64,
        time1: f64,
    ) -> BvhNode {
        let mut rng = thread_rng();

        let mut objects = src_objects.clone();
        let axis: u8 = rng.gen_range(0..2);
        let box_compare = move |a: &Arc<HittableWrapper>,
                                b: &Arc<HittableWrapper>| {
            let box_a = a.bounding_box(0.0, 0.0).unwrap();
            let box_b = b.bounding_box(0.0, 0.0).unwrap();
            match axis {
                0 => match box_a.get_min().get_x() < box_b.get_min().get_x() {
                    true => Ordering::Less,
                    _ => Ordering::Greater,
                },
                1 => match box_a.get_min().get_y() < box_b.get_min().get_y() {
                    true => Ordering::Less,
                    _ => Ordering::Greater,
                },
                2 => match box_a.get_min().get_z() < box_b.get_min().get_z() {
                    true => Ordering::Less,
                    _ => Ordering::Greater,
                },
                _ => {
                    panic!("Undefined axis")
                }
            }
        };

        let object_span = end - start;
        let left;
        let right;

        // consider adding case == 3 to reduce recursive base cases
        if object_span == 1 {
            left = objects[start].clone();
            right = objects[start].clone();
        } else if object_span == 2 {
            if box_compare(&objects[start], &objects[start + 1]) == Ordering::Less {
                left = objects[start].clone();
                right = objects[start + 1].clone();
            } else {
                left = objects[start + 1].clone();
                right = objects[start].clone();
            }
        } else {
            objects.sort_by(box_compare);
            let mid = start + object_span / 2;
            left = Arc::new(HittableWrapper::BvhNode(BvhNode::new(&objects, start, mid, time0, time1)));
            right = Arc::new(HittableWrapper::BvhNode(BvhNode::new(&objects, mid, end, time0, time1)));
        }

        let left_box = left
            .bounding_box(time0, time1)
            .expect("No bounding box in bvh node constructor..");
        let right_box = right
            .bounding_box(time0, time1)
            .expect("No bounding box in bvh node constructor..");

        let bbox = Aabb::surrounding_box(&left_box, &right_box);

        //eprintln!("{} {}", bbox.get_min(), bbox.get_max());

        BvhNode { left, right, bbox }
    }

    pub fn from_list(list: &HittableList, time0: f64, time1: f64) -> BvhNode {
        BvhNode::new(
            list.get_objects(),
            0,
            list.get_objects().len(),
            time0,
            time1,
        )
    }
}

impl Hittable for BvhNode {
    fn hit(&self, r: &crate::ray::Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        // TODO: wtf is this lol
        if !self.bbox.hit(r, t_min, t_max) {
            return None;
        }
        let leftside_hit = self.left.hit(r, t_min, t_max);
        if leftside_hit.is_some() {
            let left = leftside_hit.unwrap();
            match self.right.hit(r, t_min, left.get_t()) {
                Some(rec) => return Some(rec),
                None => (),
            }
            return Some(left);
        }
        self.right.hit(r, t_min, t_max)
    }
    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
        // TODO don't clone?
        Some(self.bbox.clone())
    }
}
