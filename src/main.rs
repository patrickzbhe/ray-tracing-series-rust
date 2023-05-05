use rand::{thread_rng, Rng};
use ray_tracing_series_rust::aabb::Aabb;
use ray_tracing_series_rust::bvh::{self, BvhNode};
use ray_tracing_series_rust::camera::Camera;
use ray_tracing_series_rust::hit::{
    Dielectric, DiffuseLight, Hittable, HittableList, Lambertian, Material, Metal, MovingSphere,
    Sphere, XyRect,
};
use ray_tracing_series_rust::ray::Ray;
use ray_tracing_series_rust::screen::Screen;
use ray_tracing_series_rust::texture::{Checker, Image, Noise};
use ray_tracing_series_rust::vec3::{random, random_range, Color, Vec3};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::Instant;
use std::{backtrace, thread};

const THREADS: usize = 10;

fn ray_color(
    &r: &Ray,
    background: &Color,
    world: &Box<dyn Hittable + Sync>,
    mut depth: i32,
) -> Color {
    // TODO: make this iterative instead of recursive
    let mut product = Vec3::new(1, 1, 1);
    let mut output = Vec3::new(0, 0, 0);
    let mut current_ray = r;

    loop {
        depth -= 1;
        if depth < 0 {
            return Vec3::new(0, 0, 0);
        }
        match world.hit(&current_ray, 0.001, f64::INFINITY) {
            Some(rec) => match rec.get_material().scatter(&current_ray, &rec) {
                Some((scattered, attenuation)) => {
                    let emitted = rec
                        .get_material()
                        .emitted(rec.get_u(), rec.get_v(), rec.get_p());
                    output += emitted * product;
                    product *= attenuation;
                    current_ray = scattered;
                }
                None => {
                    let emitted = rec
                        .get_material()
                        .emitted(rec.get_u(), rec.get_v(), rec.get_p());
                    output += emitted * product;
                    break;
                }
            },
            None => {
                output += product * *background;
                break;
            }
        }
    }
    output
}

fn gen_random_scene() -> Box<dyn Hittable + Sync> {
    let mut rng = thread_rng();
    let mut list = HittableList::new();
    let ground: Arc<Box<dyn Material>> =
        Arc::new(Box::new(Lambertian::from_pointer(Arc::new(Box::new(
            Checker::from_colors(&Color::new(0.2, 0.3, 0.1), &Color::new(0.9, 0.9, 0.9)),
        )))));
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
                let sphere_material: Box<dyn Material> = if choose_mat < 0.3 {
                    // diffuse
                    let albedo = random() * random();
                    Box::new(Lambertian::new(albedo))
                } else if choose_mat < 0.6 {
                    let albedo = random_range(0.5, 1.0);
                    let fuzz = rng.gen_range::<f64, std::ops::Range<f64>>(0.0..0.5);
                    Box::new(Metal::new(albedo, fuzz))
                } else {
                    Box::new(Dielectric::new(1.5))
                };
                if choose_mat < 0.8 {
                    let center2 = center + Vec3::new(0, rng.gen_range(0.0..0.5), 0);
                    list.add(Arc::new(Box::new(MovingSphere::new(
                        center,
                        center2,
                        0.0,
                        1.0,
                        0.2,
                        Arc::new(sphere_material),
                    ))));
                    continue;
                }

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
    list.add(Arc::new(Box::new(Sphere::new(
        Vec3::new(-4, 1, 0),
        1.0,
        m2,
    ))));
    list.add(Arc::new(Box::new(Sphere::new(Vec3::new(4, 1, 0), 1.0, m3))));

    let bvhnode = BvhNode::from_list(&list, 0.0, 1.0);

    let world: Box<dyn Hittable + Sync> = Box::new(bvhnode);
    //let world: Box<dyn Hittable + Sync> = Box::new(list);
    world
}

fn gen_checkered_sphere() -> Box<dyn Hittable + Sync> {
    let mut list = HittableList::new();
    let ground: Arc<Box<dyn Material>> =
        Arc::new(Box::new(Lambertian::from_pointer(Arc::new(Box::new(
            Checker::from_colors(&Color::new(0.2, 0.3, 0.1), &Color::new(0.9, 0.9, 0.9)),
        )))));
    list.add(Arc::new(Box::new(Sphere::new(
        Vec3::new(0, -10, 0),
        10.0,
        ground.clone(),
    ))));

    list.add(Arc::new(Box::new(Sphere::new(
        Vec3::new(0, 10, 0),
        10.0,
        ground,
    ))));

    Box::new(list)
}

fn gen_two_perlin() -> Box<dyn Hittable + Sync> {
    let mut list = HittableList::new();
    let ground: Arc<Box<dyn Material>> = Arc::new(Box::new(Lambertian::from_pointer(Arc::new(
        Box::new(Noise::new(4.0)),
    ))));
    list.add(Arc::new(Box::new(Sphere::new(
        Vec3::new(0, -1000, 0),
        1000.0,
        ground.clone(),
    ))));

    list.add(Arc::new(Box::new(Sphere::new(
        Vec3::new(0, 2, 0),
        2.0,
        ground,
    ))));

    Box::new(list)
}

fn earth() -> Box<dyn Hittable + Sync> {
    let mut list = HittableList::new();
    let ground: Arc<Box<dyn Material>> = Arc::new(Box::new(Lambertian::from_pointer(Arc::new(
        Box::new(Image::from_ppm("test_checker.ppm")),
    ))));
    list.add(Arc::new(Box::new(Sphere::new(
        Vec3::new(0, -1000, 0),
        1000.0,
        ground.clone(),
    ))));

    list.add(Arc::new(Box::new(Sphere::new(
        Vec3::new(0, 2, 0),
        2.0,
        ground,
    ))));

    Box::new(list)
}

fn gen_simple_light() -> Box<dyn Hittable + Sync> {
    let mut list = HittableList::new();
    let ground: Arc<Box<dyn Material>> = Arc::new(Box::new(Lambertian::from_pointer(Arc::new(
        Box::new(Noise::new(4.0)),
    ))));
    list.add(Arc::new(Box::new(Sphere::new(
        Vec3::new(0, -1000, 0),
        1000.0,
        ground.clone(),
    ))));

    list.add(Arc::new(Box::new(Sphere::new(
        Vec3::new(0, 2, 0),
        2.0,
        ground,
    ))));

    let difflight: Arc<Box<dyn Material>> =
        Arc::new(Box::new(DiffuseLight::new(&Color::new(10, 10, 10))));
    list.add(Arc::new(Box::new(XyRect::new(
        3.0,
        5.0,
        1.0,
        3.0,
        -2.0,
        difflight.clone(),
    ))));

    list.add(Arc::new(Box::new(Sphere::new(
        Vec3::new(0, 10, 0),
        3.0,
        difflight,
    ))));

    Box::new(list)
}

fn cornell_box() -> Box<dyn Hittable + Sync> {
    let mut list = HittableList::new();
    let ground: Arc<Box<dyn Material>> = Arc::new(Box::new(Lambertian::from_pointer(Arc::new(
        Box::new(Noise::new(4.0)),
    ))));
    list.add(Arc::new(Box::new(Sphere::new(
        Vec3::new(0, -1000, 0),
        1000.0,
        ground.clone(),
    ))));

    list.add(Arc::new(Box::new(Sphere::new(
        Vec3::new(0, 2, 0),
        2.0,
        ground,
    ))));

    let difflight: Arc<Box<dyn Material>> =
        Arc::new(Box::new(DiffuseLight::new(&Color::new(10, 10, 10))));
    list.add(Arc::new(Box::new(XyRect::new(
        3.0,
        5.0,
        1.0,
        3.0,
        -2.0,
        difflight.clone(),
    ))));

    list.add(Arc::new(Box::new(Sphere::new(
        Vec3::new(0, 10, 0),
        3.0,
        difflight,
    ))));

    Box::new(list)
}

fn get_world_cam(config_num: usize) -> (Arc<Box<dyn Hittable + Sync>>, Arc<Camera>, Color) {
    // TODO: do something smart, load from file maybe?
    let aspect_ratio: f64 = 16.0 / 9.0;
    let background = Color::new(0.7, 0.8, 1);
    match config_num {
        0 => {
            let world: Arc<Box<dyn Hittable + Sync>> = Arc::new(gen_checkered_sphere());
            // camera
            let lookfrom = Vec3::new(13, 2, 3);
            let lookat = Vec3::new(0, 0, 0);
            let vup = Vec3::new(0, 1, 0);
            let dist_to_focus = 10.0;
            let aperture = 0.1;
            let cam = Arc::new(Camera::new(
                lookfrom,
                lookat,
                vup,
                20.0,
                aspect_ratio,
                aperture,
                dist_to_focus,
                0.0,
                1.0,
            ));
            return (world, cam, background);
        }
        1 => {
            let world: Arc<Box<dyn Hittable + Sync>> = Arc::new(gen_two_perlin());
            // camera
            let lookfrom = Vec3::new(13, 2, 3);
            let lookat = Vec3::new(0, 0, 0);
            let vup = Vec3::new(0, 1, 0);
            let dist_to_focus = 10.0;
            let aperture = 0.1;
            let cam = Arc::new(Camera::new(
                lookfrom,
                lookat,
                vup,
                20.0,
                aspect_ratio,
                aperture,
                dist_to_focus,
                0.0,
                1.0,
            ));
            return (world, cam, background);
        }
        2 => {
            let world: Arc<Box<dyn Hittable + Sync>> = Arc::new(earth());
            // camera
            let lookfrom = Vec3::new(13, 2, 3);
            let lookat = Vec3::new(0, 0, 0);
            let vup = Vec3::new(0, 1, 0);
            let dist_to_focus = 10.0;
            let aperture = 0.1;
            let cam = Arc::new(Camera::new(
                lookfrom,
                lookat,
                vup,
                20.0,
                aspect_ratio,
                aperture,
                dist_to_focus,
                0.0,
                1.0,
            ));
            return (world, cam, background);
        }

        3 => {
            let world: Arc<Box<dyn Hittable + Sync>> = Arc::new(gen_simple_light());
            // camera
            let lookfrom = Vec3::new(26, 3, 6);
            let lookat = Vec3::new(0, 2, 0);
            let vup = Vec3::new(0, 1, 0);
            let dist_to_focus = 10.0;
            let aperture = 0.1;
            let cam = Arc::new(Camera::new(
                lookfrom,
                lookat,
                vup,
                20.0,
                aspect_ratio,
                aperture,
                dist_to_focus,
                0.0,
                1.0,
            ));
            let background = Color::new(0, 0, 0);
            return (world, cam, background);
        }
        _ => {
            let world: Arc<Box<dyn Hittable + Sync>> = Arc::new(gen_random_scene());
            // camera
            let lookfrom = Vec3::new(13, 2, 3);
            let lookat = Vec3::new(0, 0, 0);
            let vup = Vec3::new(0, 1, 0);
            let dist_to_focus = 10.0;
            let aperture = 0.1;
            let cam = Arc::new(Camera::new(
                lookfrom,
                lookat,
                vup,
                20.0,
                aspect_ratio,
                aperture,
                dist_to_focus,
                0.0,
                1.0,
            ));
            return (world, cam, background);
        }
    }
}

fn main() {
    let (sender, receiver) = channel();

    // timer
    let start = Instant::now();

    // random
    let mut rng = thread_rng();

    // image
    let aspect_ratio: f64 = 16.0 / 9.0;
    let image_width = 400;
    let image_height = (image_width as f64 / aspect_ratio) as i32;
    let samples_per_pixel = 400;
    let max_depth = 50;

    // let world: Box<dyn Hittable + Sync> = gen_random_scene();
    let (world, cam, background) = get_world_cam(10);

    let mut screen = Screen::new(image_width as usize, image_height as usize);

    let chunk_size = image_height as usize / THREADS;

    for t in 0..THREADS {
        let start = t * chunk_size;
        let end = usize::min(t * chunk_size + chunk_size, image_height as usize);
        let send_clone = sender.clone();
        let shared_world: Arc<Box<dyn Hittable + Sync>> = world.clone();
        let shared_cam = cam.clone();
        let rand1 = rng.gen::<f64>();
        let rand2 = rng.gen::<f64>();

        thread::spawn(move || {
            for j in start..end {
                for i in 0..image_width {
                    let mut pixel = Vec3::new(0, 0, 0);
                    for _ in 0..samples_per_pixel {
                        let u = (i as f64 + rand1) / (image_width - 1) as f64;
                        let v = (j as f64 + rand2) / (image_height - 1) as f64;
                        let r = shared_cam.get_ray(u, v);
                        pixel += ray_color(&r, &background, shared_world.as_ref(), max_depth);
                    }
                    send_clone
                        .send((
                            j as usize,
                            i as usize,
                            pixel.get_normalized_color(samples_per_pixel),
                        ))
                        .unwrap();
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
            Ok((j, i, color)) => {
                screen.update(j, i, color);
            }
            Err(_) => {
                break;
            }
        }
        if (loops % 10000) == 0 {
            eprintln!("\rDone {} many loops out of {}", loops, total);
        }
    }

    eprintln!("\nDone!");
    screen.write_to_ppm();

    eprintln!("Time taken: {:.3?}", start.elapsed());
}
