use crate::hit::{Hit, HitResult};
use crate::hittables::aabb::AABB;
use crate::ray::Ray;
use crate::vec3::Vec3;

use std::sync::Arc;

/*
    This is more idiomatic, but it increases rendering time.
    We should probably try to have some order in the nodes vector.
    Right now it's kind of random...
*/

#[derive(Clone)]
pub struct BvhTree {
    nodes: Vec<BvhNode>,
    root: usize,
}

/// A Node of the Bounding Volume Hierarchy Tree
#[derive(Clone)]
struct BvhNode {
    /// the bounding box of this node
    bb: AABB, //24b
    /// the actual object if leaf, else None
    hittable: Option<Arc<dyn Hit>>, //8b
    /// the left child node, if any
    left: Option<usize>, //16b !!!
    /// the right child node, if any
    right: Option<usize>, //16b !!!
}

impl BvhTree {
    pub fn from_hittables(list: &[Arc<dyn Hit>]) -> Option<Self> {
        //clone the slice we got so we can sort it!
        let mut sorted_list: Vec<_> = Vec::from(list);
        
        let mut tree = BvhTree {
            nodes: vec![],
            root: 0,
        };
        tree.root = tree.add_node(&mut sorted_list[..]).unwrap();

        Some(tree)
    }

    fn add_node(&mut self, list: &mut [Arc<dyn Hit>]) -> Option<usize> {
        match list.len() {
            0 => None,
            1 => {
                self.nodes.push(BvhNode {
                    bb: list[0].bounding_box().unwrap(),
                    hittable: Some(list[0].clone()),
                    left: None,
                    right: None,
                });
                Some(self.nodes.len() - 1)
            },
            _ => {
                //sort it along some (random) axis
                let i: u32 = rand::random::<u32>() % 3;
                match i {
                    0 => list.sort_unstable_by(|a, b| a.center().x.partial_cmp(&b.center().x).unwrap()),
                    1 => list.sort_unstable_by(|a, b| a.center().y.partial_cmp(&b.center().y).unwrap()),
                    2 => list.sort_unstable_by(|a, b| a.center().z.partial_cmp(&b.center().z).unwrap()),
                    _ => unreachable!(),
                }

                let bb = list.bounding_box().unwrap();

                let (left_list, right_list) = list.split_at_mut(list.len()/2);
                let left_index = self.add_node(left_list);
                let right_index = self.add_node(right_list);

                self.nodes.push(BvhNode {
                    bb,
                    hittable: None,
                    left: left_index,
                    right: right_index,
                });

                Some( self.nodes.len() - 1 )
            }
        }
    }

    fn hit_node(&self, idx: usize, ray: &Ray, t_min: f32, mut t_max: f32) -> Option<HitResult> {
        let node = &self.nodes[idx];

        //only proceed if the bounding box was hit
        if let Some(hr) = node.bb.hit(ray, t_min, t_max) {
            //limit t_max, we cannot hit anything *behind* the current hit!
            //aabb returns the *backside* of it, *not* the front
            t_max = hr.ray_param;

            //if we are a hittable (-> leaf), trace it!
            if let Some(hittable) = &node.hittable {
                return hittable.hit(ray, t_min, t_max);
            }

            //otherwise, check left and right children
            let left_hit = match node.left {
                Some(index) => self.hit_node(index, ray, t_min, t_max),
                None => None,
            };
            let right_hit = match node.right {
                Some(index) => self.hit_node(index, ray, t_min, t_max),
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

        } else {
            None
        }
    }
}

impl Hit for BvhTree {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
        self.hit_node(self.root, ray, t_min, t_max)
    }

    fn bounding_box(&self) -> Option<AABB> {
        Some(self.nodes[self.root].bb)
    }

    fn center(&self) -> Vec3 {
        self.nodes[self.root].bb.center()
    }
}
