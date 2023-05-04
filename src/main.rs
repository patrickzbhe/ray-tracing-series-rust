use rand::{thread_rng, Rng};
use ray_tracing_series_rust::camera::Camera;
use ray_tracing_series_rust::hit::{
    Dielectric, Hittable, HittableList, Lambertian, Material, Metal, Sphere,
};
use ray_tracing_series_rust::ray::Ray;
use ray_tracing_series_rust::screen::Screen;
use ray_tracing_series_rust::vec3::{random, random_range, Color, Vec3};
use std::rc::Rc;
use std::time::Instant;

fn ray_color(&r: &Ray, world: &Box<dyn Hittable>, mut depth: i32) -> Color {
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

fn gen_random_scene() -> Box<dyn Hittable> {
    let mut rng = thread_rng();
    let mut list = HittableList::new();
    let ground: Rc<Box<dyn Material>> =
        Rc::new(Box::new(Lambertian::new(Vec3::new(0.5, 0.5, 0.5))));
    list.add(Rc::new(Box::new(Sphere::new(
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
                list.add(Rc::new(Box::new(Sphere::new(
                    center,
                    0.2,
                    Rc::new(sphere_material),
                ))));
            }
        }
    }

    let m1: Rc<Box<dyn Material>> = Rc::new(Box::new(Dielectric::new(1.5)));
    let m2: Rc<Box<dyn Material>> = Rc::new(Box::new(Lambertian::new(Vec3::new(0.4, 0.2, 0.1))));
    let m3: Rc<Box<dyn Material>> = Rc::new(Box::new(Metal::new(Vec3::new(0.7, 0.6, 0.5), 0.0)));

    list.add(Rc::new(Box::new(Sphere::new(Vec3::new(0, 1, 0), 1.0, m1))));
    list.add(Rc::new(Box::new(Sphere::new(Vec3::new(-4, 1, 0), 1.0, m2))));
    list.add(Rc::new(Box::new(Sphere::new(Vec3::new(4, 1, 0), 1.0, m3))));

    let world: Box<dyn Hittable> = Box::new(list);
    world
}

fn main() {
    // timer
    let start = Instant::now();

    // random
    let mut rng = thread_rng();

    // image
    let aspect_ratio: f64 = 3.0 / 2.0;
    let image_width = 800;
    let image_height = (image_width as f64 / aspect_ratio) as i32;
    let samples_per_pixel = 1;
    let max_depth = 50;

    let world = gen_random_scene();

    // camera
    let lookfrom = Vec3::new(13, 2, 3);
    let lookat = Vec3::new(0, 0, 0);
    let vup = Vec3::new(0, 1, 0);
    let dist_to_focus = 10.0;
    let aperature = 0.1;
    let cam = Camera::new(
        lookfrom,
        lookat,
        vup,
        20.0,
        aspect_ratio,
        aperature,
        dist_to_focus,
    );

    let mut screen = Screen::new(image_width as usize, image_height as usize);

    for j in (0..image_height).rev() {
        eprint!("\rScanlines remaining: {j} ");
        for i in 0..image_width {
            let mut pixel = Vec3::new(0, 0, 0);
            for _ in 0..samples_per_pixel {
                let u = (i as f64 + rng.gen::<f64>()) / (image_width - 1) as f64;
                let v = (j as f64 + rng.gen::<f64>()) / (image_height - 1) as f64;
                let r = cam.get_ray(u, v);
                pixel += ray_color(&r, &world, max_depth);
            }
            screen.update(
                j as usize,
                i as usize,
                pixel.get_normalized_color(samples_per_pixel),
            );
        }
    }

    eprintln!("\nDone!");
    screen.write_to_ppm();

    eprintln!("Time taken: {:.3?}", start.elapsed());
}
