use crate::aabb::Aabb;
use crate::ray::Ray3d;
use crate::{max_of_three_f32, min_of_three_f32, Mat4f, Point3d, Point4d, Vector3d};

#[derive(Copy, Clone)]
pub struct Triangle {
    pub v: [Point3d; 3],
    pub(crate) normal: Vector3d,
    //parent: &Object,
}

impl Triangle {
    pub fn new(v0: Point3d, v1: Point3d, v2: Point3d) -> Self {
        let v0v1 = v1 - v0;
        let v0v2 = v2 - v0;
        let normal = v0v1.crossprod(&v0v2).normalize();
        Triangle {
            v: [v0, v1, v2],
            normal,
        }
    }

    fn _get_uv(&self, ray: &Ray3d) -> Option<(f32, f32)> {
        if let Some((_, u, v)) = self.moller_trumbore(ray) {
            Some((u, v))
        } else {
            None
        }
    }

    pub(crate) fn moller_trumbore(&self, ray: &Ray3d) -> Option<(f32, f32, f32)> {
        const EPSILON: f32 = 0.001;
        let v0v1 = self.v[1] - self.v[0];
        let v0v2 = self.v[2] - self.v[0];
        let pvec = ray.get_direction().crossprod(&v0v2);
        let det = v0v1 * pvec;

        if det < EPSILON {
            return None;
        }

        let inv_det = 1.0 / det;
        let tvec = ray.get_origin() - self.v[0];
        let u = tvec * pvec * inv_det;

        if u < 0.0 || u > 1.0 {
            return None;
        }

        let qvec = tvec.crossprod(&v0v1);
        let v = ray.get_direction() * qvec * inv_det;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = v0v2 * qvec * inv_det;
        Some((t, u, v))
    }
}
