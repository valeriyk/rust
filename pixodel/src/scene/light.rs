//pub mod light {
use geometry::Point3d;

#[derive(Copy, Clone)]
pub struct Light {
    pub position: Point3d,
    pub intensity: f32,
}

impl Light {
    pub fn new(position: Point3d, intensity: f32) -> Light {
        Light {
            position,
            intensity,
        }
    }
}
//}
