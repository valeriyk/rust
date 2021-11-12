use crate::ray::Ray3d;
use crate::{max_of_two_f32, min_of_two_f32, Mat4f, Point3d, Point4d, Vector3d};
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Aabb {
    pub(crate) min: Point3d,
    pub(crate) max: Point3d,
}

impl Aabb {
    pub fn new() -> Aabb {
        Aabb {
            min: Point3d::from_coords(f32::MAX, f32::MAX, f32::MAX),
            max: Point3d::from_coords(f32::MIN, f32::MIN, f32::MIN),
        }
    }
    pub fn from_points(min: Point3d, max: Point3d) -> Aabb {
        Aabb { min, max }
    }
    pub fn from_arrays(min: [f32; 3], max: [f32; 3]) -> Aabb {
        Aabb {
            min: Point3d::from_array(&min),
            max: Point3d::from_array(&max),
        }
    }
    pub fn get_min(&self) -> Point3d {
        self.min
    }
    pub fn get_max(&self) -> Point3d {
        self.max
    }

    fn get_superset(&self, other: Self) -> Self {
        Aabb::from_points(
            Point3d::from_coords(
                min_of_two_f32(self.min.x, other.min.x),
                min_of_two_f32(self.min.y, other.min.y),
                min_of_two_f32(self.min.z, other.min.z),
            ),
            Point3d::from_coords(
                max_of_two_f32(self.max.x, other.max.x),
                max_of_two_f32(self.max.y, other.max.y),
                max_of_two_f32(self.max.z, other.max.z),
            ),
        )
    }
}

impl core::ops::Add<Aabb> for Aabb {
    type Output = Aabb;

    fn add(self, other: Self) -> Self::Output {
        self.get_superset(other)
    }
}

impl std::ops::AddAssign for Aabb {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::iter::Sum for Aabb {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(Aabb::new(), |a, b| a + b)
    }
}

impl Display for Aabb {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}, {}, {}],[{}, {}, {}]",
            self.min.x, self.min.y, self.min.z, self.max.x, self.max.y, self.max.z
        )
    }
}
