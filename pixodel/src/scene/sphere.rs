//use crate::geometry::{Mat4f, Point3d, Point4d};
use geometry::sphere::Sphere;
use geometry::triangle::Triangle;
use geometry::PrimitiveType;

use crate::scene::IntoPrimitives;
//use crate::scene::IntoTriangles;

pub struct SphereObj {
    model: Sphere,
}

impl SphereObj {
    pub fn new(model: Sphere) -> Self {
        SphereObj { model }
    }

    pub fn iter(&self) -> IterSphereObj {
        IterSphereObj {
            sphereobj: &self,
            idx: 0,
        }
    }
}

// impl IntoTriangles for TriObj {
// 	fn triangulate(&self) -> Vec<Triangle> {
// 		self.iter().collect()
// 	}
// }
impl IntoPrimitives for SphereObj {
    fn to_primitives(&self) -> Vec<PrimitiveType> {
        self.iter().map(|x| PrimitiveType::Sphere(x)).collect()
    }
}

pub struct IterSphereObj<'a> {
    sphereobj: &'a SphereObj,
    idx: usize,
}

impl<'a> Iterator for IterSphereObj<'a> {
    type Item = Sphere;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == 0 {
            self.idx += 1;
            Some(self.sphereobj.model)
        } else {
            None
        }
    }
}
