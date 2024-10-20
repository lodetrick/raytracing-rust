use std::sync::Arc;
use std::{env, process};

mod camera;
mod raycasting;
mod materials;
mod rand_util;
use camera::Camera;
use cgmath::MetricSpace;
use materials::{Dielectric, Lambertian, Metal};
use rand::Rng;
use raycasting::HittableList;
use raycasting::{Point3, Color3, Vec3};
use raycasting::Sphere;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Not Enough Arguments");
        process::exit(1);
    }

    // // World
    // let mut world = HittableList::new();

    // // Materials
    // let r = f64::cos(PI / 4.0);
    // let material_left = Rc::new(Lambertian::new(Color3::new(0.0,0.0,1.0)));
    // let material_right = Rc::new(Lambertian::new(Color3::new(1.0,0.0,0.0)));

    // // Objects
    // world.add(Rc::new(Sphere::new(Point3::new(-r, 0.0, -1.0), r, material_left)));
    // world.add(Rc::new(Sphere::new(Point3::new(r, 0.0, -1.0), r,material_right)));


    // Materials
    // let material_ground: Rc<Lambertian> = Rc::new(Lambertian::new(Color3::new(0.8,0.8,0.0)));
    // let material_center: Rc<Lambertian> = Rc::new(Lambertian::new(Color3::new(0.1,0.2,0.5)));
    // let material_left:   Rc<Dielectric> = Rc::new(Dielectric::new(1.50));
    // let material_bubble:   Rc<Dielectric> = Rc::new(Dielectric::new(1.00 / 1.50));
    // let material_right:  Rc<Metal>      = Rc::new(Metal::new(Color3::new(0.8,0.6,0.2), 1.0));

    // // Objects
    // world.add(Rc::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, material_ground)));
    // world.add(Rc::new(Sphere::new(Point3::new(0.0, 0.0, -1.2), 0.5,material_center)));
    // world.add(Rc::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.5, material_left)));
    // world.add(Rc::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.4, material_bubble)));
    // world.add(Rc::new(Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.5,material_right)));

    // let mut camera = Camera::default();
    // camera.aspect_ratio = 16.0 / 9.0;
    // camera.image_width = 400;
    // camera.samples_per_pixel = 100;
    // camera.max_depth = 50;

    // camera.v_fov = 20.;
    // camera.lookfrom = Point3::new(-2.,2.,1.);
    // camera.lookat = Point3::new(0.,0.,-1.);
    // camera.vup = Vec3::new(0.,1.,0.);

    // camera.defocus_angle = 10.0;
    // camera.focus_dist = 3.4;
    // camera.focus_dist = 1.0;

    let mut rng = rand::thread_rng();

    // World
    let mut world = HittableList::new();

    let material_ground = Arc::new(Lambertian::new(Color3::new(0.5,0.5,0.5)));
    world.add(Arc::new(Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, material_ground)));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = rng.gen_range(0.0..1.0);
            let center = Point3::new((a as f64) + 0.9 * rng.gen_range(0.0..1.0), 0.2, (b as f64) + 0.9 * rng.gen_range(0.0..1.0));

            if center.distance2(Point3::new(4.0,0.2,0.0)) > 0.81 {
                if choose_mat < 0.8 { // Diffuse
                    let albedo = Color3::new(rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0));
                    let material = Arc::new(Lambertian::new(albedo));
                    world.add(Arc::new(Sphere::new(center, 0.2, material)));
                }
                else if choose_mat < 0.95 { // Metal
                    let albedo = Color3::new(rng.gen_range(0.5..1.0), rng.gen_range(0.5..1.0), rng.gen_range(0.5..1.0));
                    let fuzz = rng.gen_range(0.0..0.5);
                    let material = Arc::new(Metal::new(albedo, fuzz));
                    world.add(Arc::new(Sphere::new(center, 0.2, material)));
                }
                else { // Glass
                    let material = Arc::new(Dielectric::new(1.5));
                    world.add(Arc::new(Sphere::new(center, 0.2, material)));
                }
            }
        }
    }

    let material1 = Arc::new(Dielectric::new(1.5));
    world.add(Arc::new(Sphere::new(Point3::new(0.0,1.0,0.0), 1.0, material1)));

    let material2 = Arc::new(Lambertian::new(Color3::new(0.4,0.2,0.1)));
    world.add(Arc::new(Sphere::new(Point3::new(-4.0,1.0,0.0), 1.0, material2)));

    let material3 = Arc::new(Metal::new(Color3::new(0.7,0.6,0.5), 0.0));
    world.add(Arc::new(Sphere::new(Point3::new(4.0,1.0,0.0), 1.0, material3)));

    let mut camera = Camera::default();

    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = 400;
    camera.samples_per_pixel = 20;
    camera.max_depth = 10;

    camera.v_fov = 20.;
    camera.lookfrom = Point3::new(13.,2.,3.);
    camera.lookat = Point3::new(0.,0.,0.);
    camera.vup = Vec3::new(0.,1.,0.);

    camera.defocus_angle = 0.6;
    camera.focus_dist = 10.0;

    camera.initialize();
    Camera::render_parallel(camera, &args[1], world);
    // camera.render(&args[1], &world);
}
