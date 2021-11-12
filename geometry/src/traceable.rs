use crate::aabb::Aabb;
use crate::ray::Ray3d;
use crate::sphere::Sphere;
use crate::triangle::Triangle;
use crate::{max_of_three_f32, min_of_three_f32, Mat4f, Point3d, Point4d, Vector3d};
use std::mem;

pub trait TraceablePrimitive {
    fn get_distance_to(&self, ray: &Ray3d) -> Option<f32>;
    fn get_normal(&self, surface_pt: &Point3d) -> Vector3d;
    fn get_bounding_box(&self) -> Aabb;
    fn get_centroid(&self) -> Point3d;
    fn model_to_world(&self, model: &Mat4f) -> Self;
}

#[derive(Copy, Clone)]
pub enum PrimitiveType {
    Triangle(Triangle),
    Sphere(Sphere),
    //Point3d(Point3d)
}

impl TraceablePrimitive for PrimitiveType {
    fn get_distance_to(&self, ray: &Ray3d) -> Option<f32> {
        match self {
            PrimitiveType::Triangle(t) => t.get_distance_to(ray),
            PrimitiveType::Sphere(s) => s.get_distance_to(ray),
            //PrimitiveType::Point3d(p) => p.get_distance_to(ray),
        }
    }

    fn get_normal(&self, surface_pt: &Point3d) -> Vector3d {
        match self {
            PrimitiveType::Triangle(t) => t.get_normal(surface_pt),
            PrimitiveType::Sphere(s) => s.get_normal(surface_pt),
        }
    }

    fn get_bounding_box(&self) -> Aabb {
        match self {
            PrimitiveType::Triangle(t) => t.get_bounding_box(),
            PrimitiveType::Sphere(s) => s.get_bounding_box(),
        }
    }

    fn get_centroid(&self) -> Point3d {
        match self {
            PrimitiveType::Triangle(t) => t.get_centroid(),
            PrimitiveType::Sphere(s) => s.get_centroid(),
        }
    }

    fn model_to_world(&self, model: &Mat4f) -> Self {
        match self {
            PrimitiveType::Triangle(t) => PrimitiveType::Triangle(t.model_to_world(model)),
            PrimitiveType::Sphere(s) => PrimitiveType::Sphere(s.model_to_world(model)),
        }
    }
}

// TODO: Use the implementation from "An Efficient and Robust Ray-Box Intersection Algorithm" by Williams et al
impl TraceablePrimitive for Aabb {
    fn get_distance_to(&self, ray: &Ray3d) -> Option<f32> {
        let mut tmin = (self.min.x - ray.get_origin().x) / ray.get_direction().x;
        let mut tmax = (self.max.x - ray.get_origin().x) / ray.get_direction().x;
        if tmin > tmax {
            mem::swap(&mut tmin, &mut tmax);
        }

        let mut tymin = (self.min.y - ray.get_origin().y) / ray.get_direction().y;
        let mut tymax = (self.max.y - ray.get_origin().y) / ray.get_direction().y;
        if tymin > tymax {
            mem::swap(&mut tymin, &mut tymax);
        }

        if tmin > tymax || tymin > tmax {
            return None;
        }

        if tymin > tmin {
            tmin = tymin;
        }
        if tymax < tmax {
            tmax = tymax;
        }

        let mut tzmin = (self.min.z - ray.get_origin().z) / ray.get_direction().z;
        let mut tzmax = (self.max.z - ray.get_origin().z) / ray.get_direction().z;
        if tzmin > tzmax {
            mem::swap(&mut tzmin, &mut tzmax);
        }

        if tmin > tzmax || tzmin > tmax {
            return None;
        }

        if tzmin > tmin {
            tmin = tzmin;
        }
        if tzmax < tmax {
            tmax = tzmax;
        }

        if tmin >= 0.0 {
            Some(tmin)
        } else {
            Some(tmax)
        }
    }

    //fn intersect (&self, ray r)

    fn get_normal(&self, _: &Point3d) -> Vector3d {
        Vector3d::new() //TODO: how to find out normal to Aabb?
    }

    fn get_bounding_box(&self) -> Aabb {
        *self
    }

    fn get_centroid(&self) -> Point3d {
        Point3d::from_coords(
            (self.min.x + self.max.x) * 0.5,
            (self.min.y + self.max.y) * 0.5,
            (self.min.z + self.max.z) * 0.5,
        )
    }

    fn model_to_world(&self, model: &Mat4f) -> Self {
        Aabb::from_points(
            Point3d::from(model * Point4d::from(self.min)),
            Point3d::from(model * Point4d::from(self.max)),
        )
    }
}

impl TraceablePrimitive for Point3d {
    fn get_distance_to(&self, ray: &Ray3d) -> Option<f32> {
        // create a new vector
        let new = Vector3d::from_points(ray.get_origin(), *self);
        // dot product tells us cos of angle between the ray and the new vector,
        // if the angle is zero then our point lays on the ray
        let cos = new.normalize() * ray.get_direction().normalize();
        if cos > 0.99999 {
            Some(new.len())
        } else {
            None
        }
    }
    fn get_normal(&self, surface_pt: &Point3d) -> Vector3d {
        Vector3d::new()
    }

    fn get_bounding_box(&self) -> Aabb {
        Aabb::from_points(*self, *self)
    }

    fn get_centroid(&self) -> Point3d {
        *self
    }

    fn model_to_world(&self, model: &Mat4f) -> Self {
        Point3d::new()
    }
}

impl TraceablePrimitive for Sphere {
    fn get_distance_to(&self, ray: &Ray3d) -> Option<f32> {
        let l = self.center - ray.get_origin();
        let tca = l * ray.get_direction();
        let d_squared = l * l - tca * tca;
        if d_squared > (self.radius * self.radius) {
            return None;
        }
        let thc = (self.radius * self.radius - d_squared).sqrt();
        let t0 = tca - thc;
        let t1 = tca + thc;
        if t0 >= 0.0 {
            Some(t0)
        } else if t0 < 0.0 && t1 >= 0.0 {
            Some(t1)
        } else {
            None
        }
    }

    fn get_normal(&self, surface_pt: &Point3d) -> Vector3d {
        (*surface_pt - self.center).normalize()
    }

    fn get_bounding_box(&self) -> Aabb {
        Aabb::from_points(self.center - self.radius, self.center + self.radius)
    }
    fn get_centroid(&self) -> Point3d {
        self.center
    }

    fn model_to_world(&self, model: &Mat4f) -> Self {
        Sphere::new(
            Point3d::from(model * Point4d::from(self.center)),
            self.radius, //TODO: need to scale it too!
        )
    }
}

impl TraceablePrimitive for Triangle {
    fn get_distance_to(&self, ray: &Ray3d) -> Option<f32> {
        if let Some((t, _, _)) = self.moller_trumbore(ray) {
            Some(t)
        } else {
            None
        }
    }

    fn get_normal(&self, _: &Point3d) -> Vector3d {
        // Vec3f::new(0.0, 0.0, 0.0)
        self.normal
    }

    fn get_bounding_box(&self) -> Aabb {
        Aabb::from_points(
            Point3d::from_coords(
                min_of_three_f32(self.v[0].x, self.v[1].x, self.v[2].x),
                min_of_three_f32(self.v[0].y, self.v[1].y, self.v[2].y),
                min_of_three_f32(self.v[0].z, self.v[1].z, self.v[2].z),
            ),
            Point3d::from_coords(
                max_of_three_f32(self.v[0].x, self.v[1].x, self.v[2].x),
                max_of_three_f32(self.v[0].y, self.v[1].y, self.v[2].y),
                max_of_three_f32(self.v[0].z, self.v[1].z, self.v[2].z),
            ),
        )
    }

    fn get_centroid(&self) -> Point3d {
        Point3d::from_coords(
            self.v.iter().map(|p| p.x).sum::<f32>() / 3.0,
            self.v.iter().map(|p| p.y).sum::<f32>() / 3.0,
            self.v.iter().map(|p| p.z).sum::<f32>() / 3.0,
        )
    }

    fn model_to_world(&self, model: &Mat4f) -> Self {
        Triangle::new(
            Point3d::from(model * Point4d::from(self.v[0])),
            Point3d::from(model * Point4d::from(self.v[1])),
            Point3d::from(model * Point4d::from(self.v[2])),
        )
    }
}
