use geometry::{Point3d, Vector3d};
use crate::scene::light::Light;

pub fn phong(
    surface_pt: Point3d,
    camera_pt: Point3d,
    surface_normal: Vector3d,
    lights: &Vec<Light>,
) -> f32 {
    let _shininess: f32 = 20.0; //TODO: use object material instead
    let diffuse_reflection: f32 = 1.0; //TODO: use object material instead
    let _specular_reflection: f32 = 0.1; //TODO: use object material instead
    let ambient_reflection: f32 = 0.1; //TODO: use object material instead

    let _surface_to_camera = (camera_pt - surface_pt).normalize();

    let mut illumination = ambient_reflection;
    for l in lights {
        let surface_to_light = (l.position - surface_pt).normalize();
        let diffuse_factor = surface_to_light * surface_normal; // cos of the light to normal angle
        if diffuse_factor > 0.0 {
            // let specular_factor = reflection_dir * surface_to_camera; // cos of the camera to reflected ray angle
            // let mut specular_factor = specular_factor.powf(shininess);
            // if specular_factor < 0.0 {
            // 	specular_factor = 0.0;
            // }
            illumination += diffuse_factor * diffuse_reflection; // + specular_factor * specular_reflection;
        }
    }
    illumination
}
