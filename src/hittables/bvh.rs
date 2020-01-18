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
            let left_node = match BvhNode::from_hittables(&list[..list.len() / 2]) {
                Some(node) => Some(Box::new(node)),
                None => None,
            };
            let right_node = match BvhNode::from_hittables(&list[list.len() / 2..]) {
                Some(node) => Some(Box::new(node)),
                None => None,
            };

            return Some(BvhNode {
                bb: list.bounding_box().unwrap(),
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
}
