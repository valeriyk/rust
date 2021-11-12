use crate::Vector3d;

// #[derive(Copy, Clone)]
// pub struct Point2d<T> {
// 	x: T,
// 	y: T,
// }
// impl Point2d<T> {
// 	fn get_barycentric(self, t: [Triangle2d; 3]) {
//
// 	}
// }

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Point3d {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3d {
    pub fn new() -> Point3d {
        Point3d {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
    pub fn from_array(a: &[f32; 3]) -> Point3d {
        Point3d {
            x: a[0],
            y: a[1],
            z: a[2],
        }
    }
    pub fn from_coords(x: f32, y: f32, z: f32) -> Point3d {
        Point3d { x, y, z }
    }
}

impl core::convert::From<Point4d> for Point3d {
    fn from(other: Point4d) -> Point3d {
        let w_inv = 1.0 / other.w;
        Self {
            x: other.x * w_inv,
            y: other.y * w_inv,
            z: other.z * w_inv,
        }
    }
}

impl core::ops::Add<Point3d> for Point3d {
    type Output = Vector3d;

    fn add(self, other: Self) -> Self::Output {
        Self::Output {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl core::ops::Add<f32> for Point3d {
    type Output = Self;

    fn add(self, other: f32) -> Self::Output {
        Self::Output {
            x: self.x + other,
            y: self.y + other,
            z: self.z + other,
        }
    }
}

impl core::ops::Sub<Point3d> for Point3d {
    type Output = Vector3d;

    #[inline]
    fn sub(self, other: Self) -> Self::Output {
        Self::Output {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl core::ops::Sub<f32> for Point3d {
    type Output = Self;

    #[inline]
    fn sub(self, other: f32) -> Self::Output {
        Self::Output {
            x: self.x - other,
            y: self.y - other,
            z: self.z - other,
        }
    }
}

impl core::ops::Add<Vector3d> for Point3d {
    type Output = Self;

    fn add(self, other: Vector3d) -> Self::Output {
        Self::Output {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl core::ops::Mul<f32> for Point3d {
    type Output = Self;

    fn mul(self, other: f32) -> Self::Output {
        Self {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}
impl core::ops::Div<f32> for Point3d {
    type Output = Self;

    fn div(self, other: f32) -> Self::Output {
        Self {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
        }
    }
}

impl core::ops::Neg for Point3d {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl std::ops::Index<usize> for Point3d {
    type Output = f32;

    fn index(&self, idx: usize) -> &Self::Output {
        match idx {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!(),
        }
    }
}

impl std::ops::IndexMut<usize> for Point3d {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        match idx {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!(),
        }
    }
}

/*pub struct IterPoint3d {
    pt: Point3d,
    item_idx: usize,
}

impl Iterator for IterPoint3d {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.item_idx {
            0 => self.pt.x,
            1 => self.pt.y,
            2 => self.pt.z,
            _ => return None,
        };
        self.item_idx += 1;
        Some(result)
    }
}
impl IntoIterator for Point3d {
    type Item = f32;
    type IntoIter = IterPoint3d;
    fn into_iter(self) -> Self::IntoIter {
        IterPoint3d {
            pt: self,
            item_idx: 0,
        }
    }
}
*/

#[derive(Copy, Clone)]
pub struct Point4d {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) z: f32,
    pub(crate) w: f32,
}

impl Point4d {
    pub fn new() -> Point4d {
        Point4d {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        }
    }
    pub fn from_array(a: &[f32; 4]) -> Point4d {
        Point4d {
            x: a[0],
            y: a[1],
            z: a[2],
            w: a[3],
        }
    }
    pub fn from_coords(x: f32, y: f32, z: f32, w: f32) -> Point4d {
        Point4d { x, y, z, w }
    }
}

impl core::convert::From<Point3d> for Point4d {
    fn from(other: Point3d) -> Self {
        Self {
            x: other.x,
            y: other.y,
            z: other.z,
            w: 1.0,
        }
    }
}

impl std::ops::Index<usize> for Point4d {
    type Output = f32;

    fn index(&self, idx: usize) -> &Self::Output {
        match idx {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w,
            _ => panic!(),
        }
    }
}

impl std::ops::IndexMut<usize> for Point4d {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        match idx {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            3 => &mut self.w,
            _ => panic!(),
        }
    }
}

// impl std::cmp::Ord for Point3d {
// 	fn cmp(&self, other: &Self) -> Ordering {
// 		unimplemented!()
// 	}
// }
