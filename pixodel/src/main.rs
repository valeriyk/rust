extern crate image;
extern crate rayon;

use geometry::{self, sphere::Sphere, triangle::Triangle, Mat4f, Point3d, Point4d, Ray3d};
use image::{ImageBuffer, Rgb};
use pixodel::scene::{self, *};
use rayon::prelude::*;

use std::sync::Arc;
use std::time::Instant;

const FRAME_WIDTH: u32 = 1280;
const FRAME_HEIGHT: u32 = 720;

fn create_scene() -> Scene {
    let head_model = Arc::new(scene::WfObj::new("pixodel/models/african_head.obj"));
    let cube_model = Arc::new(scene::WfObj::new("pixodel/models/cube.obj"));
    let stanf_bunny_model = Arc::new(scene::WfObj::new("pixodel/models/bunny.obj"));
    let nefertiti_model = Arc::new(scene::WfObj::new("pixodel/models/Nefertiti.obj"));
    let triangle_model = Arc::new(scene::TriObj::new(Triangle::new(
        Point3d::from_coords(-1.0, 1.0, 0.0),
        Point3d::from_coords(0.0, -1.0, 0.0),
        Point3d::from_coords(1.0, 0.8, 0.0),
    )));
    let sphere_model = Arc::new(scene::SphereObj::new(Sphere::new(
        Point3d::from_coords(0.0, 0.0, -20.0),
        10.0,
    )));

    Scene::new()
        // .add_obj(
        //     scene::SceneObj::new(nefertiti_model.clone())
        //         .scale(0.1, 0.1, 0.1)
        //         .rotate(90.0, 180.0, 135.0)
        //         .translate(0.0, 0.0, -100.0),
        // )
        .add_obj(
            scene::SceneObj::new(stanf_bunny_model.clone())
                .scale(7.0, 7.0, 7.0)
                .rotate(30.0, -50.0, 0.0)
                .translate(5.0, -8.0, -50.0),
        )
        // .add_obj(
        //     scene::SceneObj::new(head_model.clone())
        //         .scale(7.0, 7.0, 7.0)
        //         .rotate(0.0, 0.0, 0.0)
        //         .translate(3.0, 0.0, -30.0),
        // )
        // .add_obj(
        //     scene::SceneObj::new(head_model.clone())
        //         .scale(7.0, 7.0, 7.0)
        //         .rotate(0.0, 0.0, 0.0)
        //         .translate(-3.0, 0.0, -30.0),
        // )
        //.add_obj(scene::SceneObj::new(sphere_model)
        //         .scale(1.0, 1.0, 1.0)
        // )
        //.add_obj(scene::SceneObj::new(cube_model.clone())
        //         .scale(4.0, 4.0, 4.0)
        //         .rotate(45.0, 45.0, 0.0)
        //         .translate(5.0, 0.0, -30.0)
        // )
        //.add_obj(scene::SceneObj::new(cube_model.clone())
        //         .scale(4.0, 4.0, 4.0)
        //         .rotate(45.0, 45.0, 0.0)
        //         .translate(-5.0, 0.0, -30.0)
        // )
        //.add_obj(scene::SceneObj::new(triangle_model)
        //         .scale(10.0, 10.0, 10.0)
        //         .rotate(-45.0, 0.0, 0.0)
        //         .translate(0.0, 0.0, -40.0)
        // )
        //.add_light(Light::new(Point3d::from_coords(-50.0, -50.0, 50.0), 0.5))
        //.add_light(Light::new(Point3d::from_coords(10.0, 200.0, 20.0), 0.5))
        .add_light(Light::new(Point3d::from_coords(1.0, 0.0, 10.0), 0.5))
}

fn main() {
    let frame_width = FRAME_WIDTH;
    let frame_height = FRAME_HEIGHT;

    let aspect_ratio = (frame_width as f32) / (frame_height as f32);
    let fov_vert: f32 = 35.0;
    let fov_scaling_factor = (fov_vert / 2.0).to_radians().tan();

    // First scale from the viewport shape to NDC: [0; screen] -> [0; 2]
    let screen_to_world = Mat4f::from_rows(
        [
            2.0 * fov_scaling_factor * aspect_ratio / frame_width as f32,
            0.0,
            0.0,
            -fov_scaling_factor * aspect_ratio,
        ],
        [
            0.0,
            2.0 * fov_scaling_factor / frame_height as f32,
            0.0,
            -fov_scaling_factor,
        ],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    );

    let ray_orig = Point3d::from_coords(0.0, 0.0, 0.0);

    let recursion_depth = 4;

    loop {
        //let timer = Instant::now();
        let mesh_glob = create_scene();
        //println!("Elapsed time: {:.2?}", timer.elapsed());

        let timer = Instant::now();
        let lbvh = mesh_glob.build_par_lbvh::<8>();
        println!("LBVH construction took: {:.2?}", timer.elapsed());
        //println!("{}", lbvh);

        let timer = Instant::now();

        let mut fbuf: Vec<[u8; 3]> = vec![[0, 0, 0]; (frame_width * frame_height) as usize];
        fbuf.par_iter_mut().enumerate().for_each(|(idx, pix)| {
            let x = idx as u32 % frame_width;
            let y = idx as u32 / frame_width;
            let ray_aim = Point3d::from(
                &screen_to_world * Point4d::from_coords(x as f32, y as f32, -1.0, 1.0),
            );
            let ray_dir = ray_aim - ray_orig;
            let ray = Ray3d::from(ray_orig, ray_dir.normalize());
            let color = mesh_glob.cast_ray_lbvh(
                &lbvh,
                &ray,
                &|a, b, c, d| shading::phong(a, b, c, d),
                recursion_depth,
            );
            *pix = color;
        });

        println!("Tracing took: {:.2?}", timer.elapsed());

        //let fbuf = fbuf.iter().flatten().map(|x| *x).collect();
        let fbuf = fbuf.iter().flat_map(|x| *x).collect();

        let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::from_vec(frame_width, frame_height, fbuf).unwrap();
        image::imageops::flip_vertical_in_place(&mut img);
        img.save("myimg2.png").unwrap();

        break;
    }
}
