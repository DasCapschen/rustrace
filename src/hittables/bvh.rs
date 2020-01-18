use crate::vec3::Vec3;
use std::cmp::Ordering;
use crate::hittable::{HitResult, Hittable};
use crate::hittables::aabb::AABB;
use crate::ray::Ray;

use std::sync::Arc;

pub struct BvhNode {
    bb: AABB,
    hittable: Option<Arc<dyn Hittable>>,
    left: Option<Box<BvhNode>>,
    right: Option<Box<BvhNode>>,
}

impl BvhNode {
    /* hack af
    pub fn debug_hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Vec3 {
        if let Some(l) = &self.left {
            if l.bb.hit(ray, t_min, t_max) {

                if let Some(left) = &l.left {
                    if let Some(nl) = &left.left {
                        if nl.bb.hit(ray, t_min, t_max) {
                            return Vec3::rgb(0, 0, 255);
                        }
                    }
                    if let Some(nr) = &left.right {
                        if nr.bb.hit(ray, t_min, t_max) {
                            return Vec3::rgb(255, 0, 255);
                        }
                    }
                }
                if let Some(right) = &l.right {
                    if right.bb.hit(ray, t_min, t_max) {
                        return Vec3::rgb(255, 0, 0);
                    }
                }
            }
        }

        if let Some(r) = &self.right {
            if r.bb.hit(ray, t_min, t_max) {
                return Vec3::rgb(0, 255, 0);
            }
        }

        if self.bb.hit(ray, t_min, t_max) {
            return Vec3::rgb(255, 255, 255);
        }

        return Vec3::rgb(0, 0, 0);
    }
    */

    // this is recursive!
    pub fn from_hittables(list: &[Arc<dyn Hittable>]) -> Option<BvhNode> {
        //if empty list, return nothing
        if list.is_empty() {
            return None;
        }
        //if list is 1 element
        else if list.len() == 1 {
            return Some(BvhNode {
                bb: list[0].bounding_box().unwrap(),
                hittable: Some(list[0].clone()),
                left: None,
                right: None,
            });
        } else {
            //clone the slice we got so we can sort it!
            //IDEA: pass a mutable slice?
            let mut sorted_list: Vec<_> = Vec::from(list);

            //sort it along some (random) axis
            let i: u32 = rand::random::<u32>() % 3;
            match i {
                0 => sorted_list.sort_unstable_by(|a, b| a.center().x.partial_cmp(&b.center().x).unwrap()),
                1 => sorted_list.sort_unstable_by(|a, b| a.center().y.partial_cmp(&b.center().y).unwrap()),
                2 => sorted_list.sort_unstable_by(|a, b| a.center().z.partial_cmp(&b.center().z).unwrap()),
                _ => panic!("int % 3 was not 0, 1 or 2"), //should not happen
            }

            //split it along that axis into 2
            let left_node = match BvhNode::from_hittables(&sorted_list[..sorted_list.len() / 2]) {
                Some(node) => Some(Box::new(node)),
                None => None,
            };
            let right_node = match BvhNode::from_hittables(&sorted_list[sorted_list.len() / 2..]) {
                Some(node) => Some(Box::new(node)),
                None => None,
            };

            return Some(BvhNode {
                bb: sorted_list.bounding_box().unwrap(),
                hittable: None,
                left: left_node,
                right: right_node,
            });
        }
    }
}

impl Hittable for BvhNode {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitResult> {
        //only proceed if the bounding box was hit
        if self.bb.hit(ray, t_min, t_max) {
            //if we are a hittable (-> leaf), trace it!
            if let Some(h) = &self.hittable {
                return h.hit(ray, t_min, t_max);
            }

            //otherwise, check left and right children
            let left_hit = match &self.left {
                Some(node) => node.hit(ray, t_min, t_max),
                None => None,
            };
            let right_hit = match &self.right {
                Some(node) => node.hit(ray, t_min, t_max),
                None => None,
            };

            if let Some(hit1) = left_hit {
                if let Some(hit2) = right_hit {
                    if hit1.ray_param > hit2.ray_param {
                        return Some(hit2);
                    }
                }
                return Some(hit1);
            } else if let Some(hit2) = right_hit {
                return Some(hit2);
            } else {
                return None;
            }
        }
        None
    }

    fn bounding_box(&self) -> Option<AABB> {
        Some(self.bb)
    }

    fn center(&self) -> Vec3 {
        todo!()
    }
}
