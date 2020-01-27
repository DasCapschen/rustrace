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
    //root is always 0
    nodes: Vec<BvhNode>,
    objects: Vec<Arc<dyn Hit>>,
}

/// A Node of the Bounding Volume Hierarchy Tree
#[derive(Clone)]
struct BvhNode {
    /// the bounding box of this node
    bb: AABB, //24b
    /// the index of the left child (right is this +1)
    left: u32, //4b
    /// if leaf, amount of objects, else 0
    count: u32, //4b
}

impl BvhTree {
    pub fn from_hittables(list: Vec<Arc<dyn Hit>>) -> Self {
        //clone the slice we got so we can sort it!        
        let mut tree = BvhTree {
            nodes: vec![],
            objects: vec![],
        };

        tree.nodes.push(BvhNode{
            bb: list.bounding_box().unwrap(),
            left: 0, 
            count: 0,
        });
        tree.build_subtree(0, list);

        tree
    }

    fn build_subtree(&mut self, index: u32, mut list: Vec<Arc<dyn Hit>>) {
        //sort list along some (random) axis
        let i: u32 = rand::random::<u32>() % 3;
        match i {
            0 => list.sort_unstable_by(|a, b| a.center().x.partial_cmp(&b.center().x).unwrap()),
            1 => list.sort_unstable_by(|a, b| a.center().y.partial_cmp(&b.center().y).unwrap()),
            2 => list.sort_unstable_by(|a, b| a.center().z.partial_cmp(&b.center().z).unwrap()),
            _ => unreachable!(),
        }

        match list.len() {
            0 => panic!("plz no empty list thx"),
            1 => {
                let left = self.objects.len() as u32;
                self.objects.push(list.remove(0));

                self.nodes[index as usize].left = left;
                self.nodes[index as usize].count = 1;
            },
            2 => {
                let left = self.objects.len() as u32;

                self.objects.push(list.remove(0));
                self.objects.push(list.remove(0));

                self.nodes[index as usize].left = left;
                self.nodes[index as usize].count = 2;
            },
            _ => {
                let left = self.nodes.len() as u32;

                self.nodes[index as usize].left = left;
                self.nodes[index as usize].count = 0;

                let right_list = list.split_off(list.len()/2);

                self.nodes.push(BvhNode{
                    bb: list.bounding_box().unwrap(),
                    left: 0, 
                    count: 0,
                });

                self.nodes.push(BvhNode{
                    bb: right_list.bounding_box().unwrap(),
                    left: 0, 
                    count: 0,
                });

                self.build_subtree(left, list);
                self.build_subtree(left+1, right_list);
            }
        }
    }

    fn hit_node(&self, idx: u32, ray: &Ray, t_min: f32, mut t_max: f32) -> Option<HitResult> {
        let node = &self.nodes[idx as usize];

        //only proceed if the bounding box was hit
        if let Some(hr) = node.bb.hit(ray, t_min, t_max) {
            //limit t_max, we cannot hit anything *behind* the current hit!
            //aabb returns the *backside* of it, *not* the front
            t_max = hr.ray_param;

            //early stop if single leaf
            if node.count == 1 {
                return self.objects[node.left as usize].hit(ray, t_min, t_max);
            }

            let (left_hit, right_hit) = match node.count {
                0 => {
                    //recurse further
                    (self.hit_node(node.left, ray, t_min, t_max),
                    self.hit_node(node.left+1, ray, t_min, t_max))
                },
                2 => {
                    //hit children only
                    (self.objects[node.left as usize].hit(ray, t_min, t_max),
                    self.objects[(node.left+1) as usize].hit(ray, t_min, t_max))
                },
                _ => unreachable!(),
            };

            match (left_hit, right_hit) {
                (Some(lh), Some(rh)) => {
                    if lh.ray_param < rh.ray_param {
                        Some(lh)
                    } else {
                        Some(rh)
                    }
                },
                (Some(lh), None) => Some(lh),
                (None, Some(rh)) => Some(rh),
                _ => None,
            }
        } else {
            None
        }
    }
}

impl Hit for BvhTree {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
        self.hit_node(0, ray, t_min, t_max)
    }

    fn bounding_box(&self) -> Option<AABB> {
        Some(self.nodes[0].bb)
    }

    fn center(&self) -> Vec3 {
        self.nodes[0].bb.center()
    }
}