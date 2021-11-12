use crate::aabb::Aabb;
use crate::ray::Ray3d;
use crate::{Mat4f, Point3d, Point4d, Vector3d};

#[derive(Copy, Clone)]
pub struct Sphere {
    pub(crate) center: Point3d,
    pub(crate) radius: f32,
}

impl Sphere {
    pub fn new(center: Point3d, radius: f32) -> Sphere {
        Sphere { center, radius }
    }
}
