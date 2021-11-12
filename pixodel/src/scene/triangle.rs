//use crate::geometry::{Mat4f, Point3d, Point4d};
use geometry::triangle::Triangle;
use geometry::PrimitiveType;

use crate::scene::IntoPrimitives;
//use crate::scene::IntoTriangles;

pub struct TriObj {
    model: Triangle,
}

impl TriObj {
    pub fn new(model: Triangle) -> Self {
        TriObj { model }
    }

    pub fn iter(&self) -> IterTriObj {
        IterTriObj {
            triobj: &self,
            idx: 0,
        }
    }
}

// impl IntoTriangles for TriObj {
// 	fn triangulate(&self) -> Vec<Triangle> {
// 		self.iter().collect()
// 	}
// }
impl IntoPrimitives for TriObj {
    fn to_primitives(&self) -> Vec<PrimitiveType> {
        self.iter().map(|x| PrimitiveType::Triangle(x)).collect()
    }
}

pub struct IterTriObj<'a> {
    triobj: &'a TriObj,
    idx: usize,
}

impl<'a> Iterator for IterTriObj<'a> {
    type Item = Triangle;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == 0 {
            self.idx += 1;
            Some(self.triobj.model)
        } else {
            None
        }
    }
}
