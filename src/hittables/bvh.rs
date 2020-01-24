use crate::hit::{Hit, HitResult};
use crate::hittables::aabb::AABB;
use crate::ray::Ray;
use crate::vec3::Vec3;

use std::sync::Arc;

/*
I feel like this whole class needs a refactoring. I mean it works, but I don't like it.
We should try to be idiomatic and do it like

struct Tree {
    nodes: Vec<Node>,
}
struct Node {
    hittable: Arc<dyn Hit>,
    left: Option<usize>,
    right: Option<usize>
}
*/

/// A Node of the Bounding Volume Hierarchy Tree
#[derive(Clone)]
pub struct BvhNode {
    /// the bounding box of this node
    bb: AABB,
    /// the actual object if leaf, else None
    hittable: Option<Arc<dyn Hit>>,
    /// the left child node, if any
    left: Option<Box<BvhNode>>,
    /// the right child node, if any
    right: Option<Box<BvhNode>>,
}

impl BvhNode {
    /// transforms a list of hittables into a bvh tree, returning the root node
    pub fn from_hittables(list: &[Arc<dyn Hit>]) -> Option<BvhNode> {
        //if empty list, return nothing
        if list.is_empty() {
            None
        }
        //if list is 1 element
        else if list.len() == 1 {
            Some(BvhNode {
                bb: list[0].bounding_box().unwrap(),
                hittable: Some(list[0].clone()),
                left: None,
                right: None,
            })
        } else {
            //clone the slice we got so we can sort it!
            //IDEA: pass a mutable slice?
            let mut sorted_list: Vec<_> = Vec::from(list);

            //sort it along some (random) axis
            let i: u32 = rand::random::<u32>() % 3;
            match i {
                0 => sorted_list
                    .sort_unstable_by(|a, b| a.center().x.partial_cmp(&b.center().x).unwrap()),
                1 => sorted_list
                    .sort_unstable_by(|a, b| a.center().y.partial_cmp(&b.center().y).unwrap()),
                2 => sorted_list
                    .sort_unstable_by(|a, b| a.center().z.partial_cmp(&b.center().z).unwrap()),
                _ => unreachable!(),
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

            Some(BvhNode {
                bb: sorted_list.bounding_box().unwrap(),
                hittable: None,
                left: left_node,
                right: right_node,
            })
        }
    }
}

impl Hit for BvhNode {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitResult> {
        //only proceed if the bounding box was hit
        if let Some(hr) = self.bb.hit(ray, t_min, t_max) {
            //limit t_max, we cannot hit anything *behind* the current hit!
            //aabb returns the *backside* of it, *not* the front
            let t_max = hr.ray_param;

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

            match (left_hit, right_hit) {
                (Some(lh), Some(rh)) => {
                    if lh.ray_param < rh.ray_param {
                        return Some(lh);
                    } else {
                        return Some(rh);
                    }
                }
                (Some(lh), None) => return Some(lh),
                (None, Some(rh)) => return Some(rh),
                (None, None) => return None,
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
