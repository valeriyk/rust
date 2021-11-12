use crate::{Point3d, Vector3d};

#[derive(Copy, Clone)]
pub struct Ray3d {
    origin: Point3d,
    direction: Vector3d,
}

impl Ray3d {
    pub fn new() -> Ray3d {
        Ray3d {
            origin: Point3d::new(),
            direction: Vector3d::new(),
        }
    }
    pub fn from(origin: Point3d, direction: Vector3d) -> Ray3d {
        Ray3d { origin, direction }
    }

    pub fn get_origin(&self) -> Point3d {
        self.origin
    }

    pub fn get_direction(&self) -> Vector3d {
        self.direction
    }
}

// impl core::ops::Add<Vector3d> for Vector3d {
// 	type Output = Self;
//
// 	fn add(self, other: Self) -> Self::Output {
// 		Self {
// 			x: self.x + other.x,
// 			y: self.y + other.y,
// 			z: self.z + other.z,
// 		}
// 	}
// }
//
// impl core::ops::Sub<Vector3d> for Vector3d {
// 	type Output = Self;
//
// 	fn sub(self, other: Self) -> Self::Output {
// 		Self {
// 			x: self.x - other.x,
// 			y: self.y - other.y,
// 			z: self.z - other.z,
// 		}
// 	}
// }
//
// impl core::ops::Add<Point3d> for Vector3d {
// 	type Output = Point3d;
//
// 	fn add(self, other: Point3d) -> Self::Output {
// 		Self::Output {
// 			x: self.x + other.x,
// 			y: self.y + other.y,
// 			z: self.z + other.z,
// 		}
// 	}
// }
//
// /// Dot product
// impl core::ops::Mul<Vector3d> for Vector3d {
// 	type Output = f32;
//
// 	#[inline]
// 	fn mul(self, other: Self) -> Self::Output {
// 		self.x * other.x + self.y * other.y + self.z * other.z
// 	}
// }
//
impl core::ops::Mul<f32> for Ray3d {
    type Output = Point3d;

    fn mul(self, other: f32) -> Self::Output {
        (self.origin + self.direction) * other
    }
}
//
// impl core::ops::Neg for Vector3d {
// 	type Output = Self;
//
// 	fn neg(self) -> Self::Output {
// 		Self {
// 			x: -self.x,
// 			y: -self.y,
// 			z: -self.z,
// 		}
// 	}
// }
