use rand::{thread_rng, Rng};
use ray_tracing_series_rust::camera::Camera;
use ray_tracing_series_rust::hit::{
    Dielectric, Hittable, HittableList, Lambertian, Material, Metal, Sphere,
};
use ray_tracing_series_rust::ray::Ray;
use ray_tracing_series_rust::screen::Screen;
use ray_tracing_series_rust::vec3::{random, random_range, Color, Vec3};
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Instant;
use std::sync::{Arc};

const THREADS: usize = 10;

fn ray_color(&r: &Ray, world: &Box<dyn Hittable + Sync>, mut depth: i32) -> Color {
    // TODO: make this iterative instead of recursive
    let mut product = Vec3::new(1, 1, 1);
    let mut current_ray = r;

    loop {
        depth -= 1;
        if depth < 0 {
            return Vec3::new(0, 0, 0);
        }
        match world.hit(&current_ray, 0.001, f64::INFINITY) {
            Some(rec) => match rec.get_material().scatter(&current_ray, &rec) {
                Some((scattered, attenuation)) => {
                    current_ray = scattered;
                    product *= attenuation;
                }
                None => return Vec3::new(0, 0, 0),
            },
            None => {
                let unit_direction = current_ray.direction().unit();
                let t = 0.5 * (unit_direction.y() + 1.0);
                product *= (1 as f64 - t) * Vec3::new(1, 1, 1) as Color
                    + t * Vec3::new(0.5, 0.7, 1) as Color;
                break;
            }
        }
    }
    product
}

fn gen_random_scene() -> Box<dyn Hittable + Sync> {
    let mut rng = thread_rng();
    let mut list = HittableList::new();
    let ground: Arc<Box<dyn Material>> =
        Arc::new(Box::new(Lambertian::new(Vec3::new(0.5, 0.5, 0.5))));
    list.add(Arc::new(Box::new(Sphere::new(
        Vec3::new(0, -1000, -1),
        1000.0,
        ground,
    ))));
    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = rng.gen::<f64>();
            let center = Vec3::new(
                a as f64 + 0.9 * rng.gen::<f64>(),
                0.2,
                b as f64 + 0.9 * rng.gen::<f64>(),
            );

            if (center - Vec3::new(4, 0.2, 0)).length() > 0.9 {
                let sphere_material: Box<dyn Material> = if choose_mat < 0.8 {
                    // diffuse
                    let albedo = random() * random();
                    Box::new(Lambertian::new(albedo))
                } else if choose_mat < 0.95 {
                    let albedo = random_range(0.5, 1.0);
                    let fuzz = rng.gen_range::<f64, std::ops::Range<f64>>(0.0..0.5);
                    Box::new(Metal::new(albedo, fuzz))
                } else {
                    Box::new(Dielectric::new(1.5))
                };
                list.add(Arc::new(Box::new(Sphere::new(
                    center,
                    0.2,
                    Arc::new(sphere_material),
                ))));
            }
        }
    }

    let m1: Arc<Box<dyn Material>> = Arc::new(Box::new(Dielectric::new(1.5)));
    let m2: Arc<Box<dyn Material>> = Arc::new(Box::new(Lambertian::new(Vec3::new(0.4, 0.2, 0.1))));
    let m3: Arc<Box<dyn Material>> = Arc::new(Box::new(Metal::new(Vec3::new(0.7, 0.6, 0.5), 0.0)));

    list.add(Arc::new(Box::new(Sphere::new(Vec3::new(0, 1, 0), 1.0, m1))));
    list.add(Arc::new(Box::new(Sphere::new(Vec3::new(-4, 1, 0), 1.0, m2))));
    list.add(Arc::new(Box::new(Sphere::new(Vec3::new(4, 1, 0), 1.0, m3))));

    let world: Box<dyn Hittable + Sync> = Box::new(list);
    world
}

fn main() {
    let (sender, receiver) = channel();

    // timer
    let start = Instant::now();

    // random
    let mut rng = thread_rng();

    // image
    let aspect_ratio: f64 = 3.0 / 2.0;
    let image_width = 800;
    let image_height = (image_width as f64 / aspect_ratio) as i32;
    let samples_per_pixel = 500;
    let max_depth = 50;

    // let world: Box<dyn Hittable + Sync> = gen_random_scene();
    let world: Arc<Box<dyn Hittable + Sync>> = Arc::new(gen_random_scene());

    // camera
    let lookfrom = Vec3::new(13, 2, 3);
    let lookat = Vec3::new(0, 0, 0);
    let vup = Vec3::new(0, 1, 0);
    let dist_to_focus = 10.0;
    let aperature = 0.1;
    let cam = Arc::new(Camera::new(
        lookfrom,
        lookat,
        vup,
        20.0,
        aspect_ratio,
        aperature,
        dist_to_focus,
    ));

    let mut screen = Screen::new(image_width as usize, image_height as usize);

    // for j in (0..image_height).rev() {
    //     eprint!("\rScanlines remaining: {j} ");
    //     for i in 0..image_width {
    //         let mut pixel = Vec3::new(0, 0, 0);
    //         for _ in 0..samples_per_pixel {
    //             let u = (i as f64 + rng.gen::<f64>()) / (image_width - 1) as f64;
    //             let v = (j as f64 + rng.gen::<f64>()) / (image_height - 1) as f64;
    //             let r = cam.get_ray(u, v);
    //             pixel += ray_color(&r, &world, max_depth);
    //         }
    //         screen.update(
    //             j as usize,
    //             i as usize,
    //             pixel.get_normalized_color(samples_per_pixel),
    //         );
    //     }
    // }
    let chunk_size = image_height as usize / THREADS;

    for t in 0..THREADS {
        let start = t * chunk_size;
        let end = usize::min(t * chunk_size + chunk_size, image_height as usize);
        let send_clone = sender.clone();
        let shared_world: Arc<Box<dyn Hittable + Sync>> = world.clone();
        let shared_cam = cam.clone();
        let rand1 = rng.gen::<f64>();
        let rand2 = rng.gen::<f64>();
        eprintln!("Thread starting at {} {}", start, end);
        
        thread::spawn(move || {
            for j in start..end {
                for i in 0..image_width {
          
                    let mut pixel = Vec3::new(0, 0, 0);
                    for _ in 0..samples_per_pixel {
                        let u = (i as f64 + rand1) / (image_width - 1) as f64;
                        let v = (j as f64 + rand2) / (image_height - 1) as f64;
                        let r = shared_cam.get_ray(u, v);
                        pixel += ray_color(&r, shared_world.as_ref(), max_depth);
                    }
                    send_clone.send((j as usize,i as usize,pixel.get_normalized_color(samples_per_pixel))).unwrap();
                }
            }
        });
    }
    drop(sender);
    let mut loops = 0;
    let total = image_height * image_width;
    loop {
        loops += 1;
        match receiver.recv() {
            Ok((j,i,color)) => {screen.update(j,i,color);}
            Err(_) => {break;}
        }
        if (loops % 10000) == 0 {
            eprintln!("\rDone {} many loops out of {}", loops, total);
        }
    }

    eprintln!("\nDone!");
    screen.write_to_ppm();

    eprintln!("Time taken: {:.3?}", start.elapsed());
}
