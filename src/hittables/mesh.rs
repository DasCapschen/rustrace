use crate::gfx::material::*;
use crate::gfx::texture::ConstantTexture;
use std::path::Path;
use std::sync::Arc;

use crate::hit::{Hit, HitResult};
use crate::hittables::aabb::AABB;
use crate::hittables::bvh::BvhTree;
use crate::math::vec3::Vec3;
use crate::ray::Ray;

#[derive(Clone)]
pub struct Mesh {
    position: Vec3,
    faces: BvhTree<Triangle>,
}

impl Mesh {
    pub fn new<P: AsRef<Path>>(file: P) -> Self {
        let (models, _mats) = tobj::load_obj(file.as_ref()).expect("couldn't load file");

        //load material
        let material = Arc::new(Lambertian::new(
            Arc::new(ConstantTexture::new(Vec3::new(0.9, 0.9, 0.9))),
            None,
        ));

        // just assume there is only 1 model in the obj!
        let mesh: Vec<Triangle> = models[0]
            .mesh
            .indices
            .chunks(3)
            .map(|chunk| {
                let p1 = Vec3 {
                    x: models[0].mesh.positions[3 * chunk[0] as usize],
                    y: models[0].mesh.positions[3 * chunk[0] as usize + 1],
                    z: models[0].mesh.positions[3 * chunk[0] as usize + 2],
                };

                let n1 = if !models[0].mesh.normals.is_empty() {
                    Some(Vec3 {
                        x: models[0].mesh.normals[3 * chunk[0] as usize ],
                        y: models[0].mesh.normals[3 * chunk[0] as usize + 1],
                        z: models[0].mesh.normals[3 * chunk[0] as usize + 2],
                    })
                } else { None };

                let uv1 = if !models[0].mesh.texcoords.is_empty() {
                    Some(( models[0].mesh.texcoords[2 * chunk[0] as usize],
                           models[0].mesh.texcoords[2 * chunk[0] as usize +1] ))
                } else { None };

                let p2 = Vec3 {
                    x: models[0].mesh.positions[3 * chunk[1] as usize],
                    y: models[0].mesh.positions[3 * chunk[1] as usize + 1],
                    z: models[0].mesh.positions[3 * chunk[1] as usize + 2],
                };
                let n2 = if !models[0].mesh.normals.is_empty() {
                    Some(Vec3 {
                        x: models[0].mesh.normals[3 * chunk[1] as usize ],
                        y: models[0].mesh.normals[3 * chunk[1] as usize + 1],
                        z: models[0].mesh.normals[3 * chunk[1] as usize + 2],
                    })
                } else { None };

                let uv2 = if !models[0].mesh.texcoords.is_empty() {
                    Some(( models[0].mesh.texcoords[2 * chunk[1] as usize],
                           models[0].mesh.texcoords[2 * chunk[1] as usize +1] ))
                } else { None };

                let p3 = Vec3 {
                    x: models[0].mesh.positions[3 * chunk[2] as usize],
                    y: models[0].mesh.positions[3 * chunk[2] as usize + 1],
                    z: models[0].mesh.positions[3 * chunk[2] as usize + 2],
                };
                let n3 = if !models[0].mesh.normals.is_empty() {
                    Some(Vec3 {
                        x: models[0].mesh.normals[3 * chunk[2] as usize ],
                        y: models[0].mesh.normals[3 * chunk[2] as usize + 1],
                        z: models[0].mesh.normals[3 * chunk[2] as usize + 2],
                    })
                } else { None };

                let uv3 = if !models[0].mesh.texcoords.is_empty() {
                    Some(( models[0].mesh.texcoords[2 * chunk[2] as usize],
                           models[0].mesh.texcoords[2 * chunk[2] as usize +1] ))
                } else { None };

                Triangle {
                    a: Vertex::new(p1, n1, uv1),
                    b: Vertex::new(p2, n2, uv2),
                    c: Vertex::new(p3, n3, uv3),
                    material: material.clone(),
                }
            })
            .collect();

        let bvh = BvhTree::from_hittables(mesh);

        Mesh {
            position: Vec3::new(0.0, 0.0, 0.0),
            faces: bvh,
        }
    }
}

//Vec<Hit> implements hittable!
impl Hit for Mesh {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
        //instead of offsetting every face by mesh's position
        //we offset the ray in the opposite direction
        let modified_ray = Ray {
            origin: ray.origin - self.position,
            direction: ray.direction,
        };

        //if we hit, undo the offsetting of the ray and correct the hit position
        if let Some(mut hit) = self.faces.hit(&modified_ray, t_min, t_max) {
            hit.hit_position += self.position;
            return Some(hit);
        }

        None
    }

    fn bounding_box(&self) -> Option<AABB> {
        self.faces.bounding_box()
    }

    fn center(&self) -> Vec3 {
        self.position
    }
}


#[derive(Copy, Clone, Debug)]
struct Vertex {
    pub position: Vec3,
    pub normal: Option<Vec3>,
    pub uv_coords: Option<(f32, f32)>,
}

impl Vertex {
    pub fn new(position: Vec3, normal: Option<Vec3>, uv_coords: Option<(f32, f32)>) -> Self {
        Self {
            position,
            normal,
            uv_coords
        }
    }
}

#[derive(Clone)]
struct Triangle {
    a: Vertex,
    b: Vertex,
    c: Vertex,
    material: Arc<dyn Material>,
}

impl Hit for Triangle {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
        let span_a = self.b.position - self.a.position;
        let span_b = self.c.position - self.a.position;
        let tri_normal = span_a.cross(span_b).normalised();

        let parameter = -(ray.origin - self.a.position).dot(tri_normal) / ray.direction.dot(tri_normal);

        //no hit if outside [min, max]
        if parameter < t_min || parameter > t_max {
            return None;
        }

        let hit_position = ray.point_at(parameter);
        let relative_hit = hit_position - self.a.position;

        // get UV coords (in [0,1] if hit)
        let ada = span_a.dot(span_a);
        let bdb = span_b.dot(span_b);
        let rda = relative_hit.dot(span_a);
        let rdb = relative_hit.dot(span_b);
        let adb = span_a.dot(span_b);

        let denom = 1.0 / ((adb * adb) - (ada * bdb));

        let alpha = ((adb * rdb) - (bdb * rda)) * denom; //along spanA
        let beta = ((adb * rda) - (ada * rdb)) * denom; //along spanB

        // u, v must be positive, smaller 1, and if a triangle, their sum must by < 1 too
        if alpha < 0.0 || beta < 0.0 || (alpha + beta) > 1.0 {
            None
        } else {

            //linear interpolate normal
            let normal = match (self.a.normal, self.b.normal, self.c.normal) {
                (Some(an), Some(bn), Some(cn)) => {
                    (1.0 - alpha - beta) * an + alpha * bn + beta * cn
                },
                _ => tri_normal,
            };

            //linear interpolate uv coordinates
            let uvcoords = match (self.a.uv_coords, self.b.uv_coords, self.c.uv_coords) {
                (Some(auv), Some(buv), Some(cuv)) => {
                    //linear interpolate u coordinate
                    let u_coord = (1.0 - alpha - beta) * auv.0 + alpha * buv.0 + beta * cuv.0;

                    //linear interpolate v coordinate
                    let v_coord = (1.0 - alpha - beta) * auv.1 + alpha * buv.1 + beta * cuv.1;

                    (u_coord, v_coord)
                },
                _ => (alpha, beta)
            };

            Some(HitResult {
                ray_param: parameter,
                hit_position,
                normal,
                material: Some(self.material.clone()),
                uv_coords: Some(uvcoords),
            })
        }
    }

    fn bounding_box(&self) -> Option<AABB> {
        let min_x = self.a.position.x.min(self.b.position.x).min(self.c.position.x);
        let min_y = self.a.position.y.min(self.b.position.y).min(self.c.position.y);
        let min_z = self.a.position.z.min(self.b.position.z).min(self.c.position.z);

        let max_x = self.a.position.x.max(self.b.position.x).max(self.c.position.x);
        let max_y = self.a.position.y.max(self.b.position.y).max(self.c.position.y);
        let max_z = self.a.position.z.max(self.b.position.z).max(self.c.position.z);

        Some(AABB::new(
            Vec3::new(min_x, min_y, min_z),
            Vec3::new(max_x, max_y, max_z)
        ))
    }

    fn center(&self) -> Vec3 {
        self.bounding_box().unwrap().center()
    }
}