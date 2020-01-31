use crate::hit::{Hit, HitResult};
use crate::hittables::aabb::{Axis, AABB};
use crate::math::vec3::Vec3;
use crate::ray::Ray;

/*
    This is more idiomatic, but it increases rendering time.
    We should probably try to have some order in the nodes vector.
    Right now it's kind of random...
*/

#[derive(Clone)]
pub struct BvhTree<T: Hit + Sized> {
    //root is always 0
    nodes: Vec<BvhNode>,
    objects: Vec<T>,
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

impl<T: Hit> BvhTree<T> {
    pub fn from_hittables(list: Vec<T>) -> Self {
        //clone the slice we got so we can sort it!
        let mut tree = BvhTree {
            nodes: vec![],
            objects: vec![],
        };

        tree.nodes.push(BvhNode {
            bb: list.bounding_box().unwrap(),
            left: 0,
            count: 0,
        });
        tree.build_subtree(0, list);

        tree
    }

    fn build_subtree(&mut self, index: u32, mut list: Vec<T>) {
        //sort by longest axis instead of randomly
        //thus, with each division, we maximise the effect the bvh has!
        //this cut the rendering time roughly in half!
        let bb = list.bounding_box().unwrap();
        match bb.longest_axis() {
            Axis::X => {
                list.sort_unstable_by(|a, b| a.center().x.partial_cmp(&b.center().x).unwrap())
            }
            Axis::Y => {
                list.sort_unstable_by(|a, b| a.center().y.partial_cmp(&b.center().y).unwrap())
            }
            Axis::Z => {
                list.sort_unstable_by(|a, b| a.center().z.partial_cmp(&b.center().z).unwrap())
            }
        }

        match list.len() {
            0 => panic!("plz no empty list thx"),
            1 => {
                let left = self.objects.len() as u32;
                self.objects.push(list.remove(0));

                self.nodes[index as usize].left = left;
                self.nodes[index as usize].count = 1;
            }
            2 => {
                let left = self.objects.len() as u32;

                self.objects.push(list.remove(0));
                self.objects.push(list.remove(0));

                self.nodes[index as usize].left = left;
                self.nodes[index as usize].count = 2;
            }
            _ => {
                let left = self.nodes.len() as u32;

                self.nodes[index as usize].left = left;
                self.nodes[index as usize].count = 0;

                //make sure we always split into EVEN sublists!
                let right_list = if (list.len() / 2) % 2 == 0 {
                    list.split_off(list.len() / 2)
                } else {
                    list.split_off((list.len() / 2) + 1)
                };

                self.nodes.push(BvhNode {
                    bb: list.bounding_box().unwrap(), //recalculate bounding box! list changed!!!
                    left: 0,
                    count: 0,
                });

                self.nodes.push(BvhNode {
                    bb: right_list.bounding_box().unwrap(),
                    left: 0,
                    count: 0,
                });

                self.build_subtree(left, list);
                self.build_subtree(left + 1, right_list);
            }
        }
    }

    fn hit_node(&self, idx: u32, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
        let node = &self.nodes[idx as usize];

        //only proceed if the bounding box was hit
        if let Some(_hr) = node.bb.hit(ray, t_min, t_max) {
            //early stop if single leaf
            if node.count == 1 {
                return self.objects[node.left as usize].hit(ray, t_min, t_max);
            }

            let (left_hit, right_hit) = match node.count {
                0 => {
                    //recurse further
                    (
                        self.hit_node(node.left, ray, t_min, t_max),
                        self.hit_node(node.left + 1, ray, t_min, t_max),
                    )
                }
                2 => {
                    //hit children only
                    (
                        self.objects[node.left as usize].hit(ray, t_min, t_max),
                        self.objects[(node.left + 1) as usize].hit(ray, t_min, t_max),
                    )
                }
                _ => unreachable!(),
            };

            match (left_hit, right_hit) {
                (Some(lh), Some(rh)) => {
                    if lh.ray_param < rh.ray_param {
                        Some(lh)
                    } else {
                        Some(rh)
                    }
                }
                (Some(lh), None) => Some(lh),
                (None, Some(rh)) => Some(rh),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn get_left_node_index(&self, idx: usize) -> usize {
        if self.nodes[idx].count != 0 {
            panic!("dont do that");
        }

        self.nodes[idx].left as usize
    }

    pub fn get_right_node_index(&self, idx: usize) -> usize {
        if self.nodes[idx].count != 0 {
            panic!("dont do that");
        }
        self.nodes[idx].left as usize + 1
    }

    pub fn debug_hit(
        &self,
        idx: usize,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> (Vec3, Vec3, Vec3, f32) {
        let node = &self.nodes[idx];

        let root_hit = node.bb.hit(ray, t_min, t_max);
        let (mut color, mut albedo, mut normal, mut depth) = if let Some(hit) = &root_hit {
            (
                Vec3::rgb(255, 0, 0),
                Vec3::rgb(255, 0, 0),
                hit.normal,
                1.0 / hit.ray_param,
            )
        } else {
            (
                Vec3::rgb(0, 0, 0),
                Vec3::rgb(0, 0, 0),
                Vec3::rgb(0, 0, 0),
                0.0,
            )
        };

        if root_hit.is_some() {
            let (left_hit, right_hit) = if node.count == 0 {
                let left_node = &self.nodes[node.left as usize];
                let right_node = &self.nodes[node.left as usize + 1];
                (
                    left_node.bb.hit(ray, t_min, t_max),
                    right_node.bb.hit(ray, t_min, t_max),
                )
            } else if node.count == 1 {
                let left = &self.objects[node.left as usize];
                (left.hit(ray, t_min, t_max), None)
            } else {
                let left = &self.objects[node.left as usize];
                let right = &self.objects[node.left as usize + 1];
                (left.hit(ray, t_min, t_max), right.hit(ray, t_min, t_max))
            };

            //get closer hit
            match (left_hit, right_hit) {
                (Some(lh), Some(rh)) => {
                    if lh.ray_param < rh.ray_param {
                        color = Vec3::rgb(0, 255, 0);
                        albedo = color;
                        normal = lh.normal;
                        depth = 1.0 / lh.ray_param;
                    } else {
                        color = Vec3::rgb(0, 0, 255);
                        albedo = color;
                        normal = rh.normal;
                        depth = 1.0 / rh.ray_param;
                    }
                }
                (Some(lh), None) => {
                    color = Vec3::rgb(0, 255, 0);
                    albedo = color;
                    normal = lh.normal;
                    depth = 1.0 / lh.ray_param;
                }
                (None, Some(rh)) => {
                    color = Vec3::rgb(0, 0, 255);
                    albedo = color;
                    normal = rh.normal;
                    depth = 1.0 / rh.ray_param;
                }
                _ => {}
            };
        }

        (color, albedo, normal, depth)
    }
}

impl<T: Hit> Hit for BvhTree<T> {
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
