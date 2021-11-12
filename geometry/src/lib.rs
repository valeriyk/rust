pub use matrix::Mat4f;
pub use point::{Point3d, Point4d};
pub use traceable::PrimitiveType;
pub use traceable::TraceablePrimitive;
pub use vector::Vector3d;

use crate::aabb::Aabb;
pub use crate::ray::Ray3d;
use crate::sphere::Sphere;
use crate::triangle::Triangle;

pub mod aabb;
pub mod matrix;
pub mod point;
pub mod ray;
pub mod sphere;
pub mod traceable;
pub mod triangle;
pub mod vector;

#[inline]
fn min_of_two_f32(a: f32, b: f32) -> f32 {
    if a < b {
        a
    } else {
        b
    }
}

#[inline]
fn max_of_two_f32(a: f32, b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}

#[inline]
fn min_of_three_f32(a: f32, b: f32, c: f32) -> f32 {
    min_of_two_f32(a, min_of_two_f32(b, c))
}

#[inline]
fn max_of_three_f32(a: f32, b: f32, c: f32) -> f32 {
    max_of_two_f32(a, max_of_two_f32(b, c))
}
