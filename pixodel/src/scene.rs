//use std::ops::Range;

//use mesh::Mesh;

use geometry::ray::Ray3d;
use geometry::{Mat4f, Point3d, Point4d, PrimitiveType, TraceablePrimitive, Vector3d};
use geometry::aabb::Aabb;
use geometry::triangle::Triangle;
pub use crate::scene::light::Light;
pub use crate::scene::sphere::SphereObj;
pub use crate::scene::triangle::TriObj;
pub use crate::scene::wfobj::WfObj;
use std::ops::Deref;
use std::sync::Arc;

use lbvh::*;
use wavefront_obj::obj::Primitive;
//use crate::traceable::PrimitiveType;

pub mod light;
pub mod triangle;
pub mod wfobj;
//pub mod tracing;
pub mod shading;
mod sphere;

pub struct Scene {
    pub lights: Vec<Light>,
    pub objects: Vec<SceneObj>,
    primitives: Vec<PrimitiveType>,
}

type IndexedCentroid = (usize, Point3d);

fn reflection_dir(surface_normal: Vector3d, surface_to_camera: Vector3d) -> Vector3d {
    let l2n_cos = surface_to_camera * surface_normal;
    surface_normal * l2n_cos * 2.0 - surface_to_camera
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            lights: Vec::new(),
            objects: Vec::new(),
            primitives: Vec::new(),
        }
    }
    pub fn add_obj(mut self, obj: SceneObj) -> Self {
        let model_mtx = obj.get_model_mtx();
        obj.object.to_primitives().into_iter().for_each(|prim| {
            self.primitives.push(prim.model_to_world(&model_mtx));
        });
        self
    }
    pub fn add_light(mut self, light: Light) -> Self {
        self.lights.push(light);
        self
    }

    pub fn build_lbvh<const N: usize>(&self) -> Octree<PrimitiveType, N>{
        lbvh::Octree::<PrimitiveType, N>::new(&self.primitives)
    }
    
    pub fn build_par_lbvh<const N: usize>(&self) -> ParOctree<PrimitiveType, N>{
        lbvh::ParOctree::<PrimitiveType, N>::new(&self.primitives)
    }
    
    pub fn cast_ray_lbvh<F, const N: usize>(&self, lbvh: &ParOctree<PrimitiveType, N>, ray: &Ray3d, vtx_shader: &F, depth: usize) -> [u8; 3]
        where
            F: FnOnce(Point3d, Point3d, Vector3d, &Vec<Light>) -> f32 + Send + Copy + 'static,
    {
        const BG_COLOR: [u8; 3] = [30u8; 3];
        const DEPTH_THRESHOLD: usize = 0;
        
        if depth < DEPTH_THRESHOLD {
            return BG_COLOR;
        }
        
        let nearest: Option<(usize, f32)> = lbvh.traverse(ray);
        
        if nearest != None {
            let (nearest_obj, dist) = nearest.unwrap();
            let surface_pt = *ray * dist;
            let surface_normal: Vector3d = self.primitives[nearest_obj].get_normal(&surface_pt);
            
            let refl_dir = reflection_dir(surface_normal, -ray.get_direction()).normalize(); //TODO: normalize really needed?
            
            //let refl_color = self.cast_ray(&surface_pt, &refl_dir, vtx_shader, depth + 1);
            
            let mut illumination =
                vtx_shader(surface_pt, ray.get_origin(), surface_normal, &self.lights);
            if illumination > 1.0 {
                illumination = 1.0
            }
            let self_color = [(illumination * u8::MAX as f32) as u8; 3];
            //[refl_color[0] + self_color[0]; 3]
            [self_color[0]; 3]
            //[100,100,100]
        } else {
            BG_COLOR
        }
    }
}

pub trait IntoPrimitives {
    fn to_primitives(&self) -> Vec<PrimitiveType>;
}
pub struct SceneObj {
    object: Arc<dyn IntoPrimitives + Sync + Send>,
    scale: [f32; 3],
    rotation: [f32; 3],
    translation: [f32; 3],
}

impl SceneObj {
    pub fn new(obj: Arc<dyn IntoPrimitives + Send + Sync>) -> Self {
        SceneObj {
            object: obj,
            scale: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            translation: [0.0, 0.0, 0.0],
        }
    }

    fn get_model_mtx(&self) -> Mat4f {
        Mat4f::identity()
            .translate_xyz(&self.translation)
            .rotate_about_x(self.rotation[0])
            .rotate_about_y(self.rotation[1])
            .rotate_about_z(self.rotation[2])
            .scale_xyz(&self.scale)
    }

    pub fn rotate(mut self, x: f32, y: f32, z: f32) -> Self {
        self.rotation = [x, y, z];
        self
    }
    pub fn scale(mut self, x: f32, y: f32, z: f32) -> Self {
        self.scale = [x, y, z];
        self
    }
    pub fn translate(mut self, x: f32, y: f32, z: f32) -> Self {
        self.translation = [x, y, z];
        self
    }
}

