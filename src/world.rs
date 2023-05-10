use crate::bvh::BvhNode;
use crate::camera::Camera;
use crate::hit::{
    ConstantMedium, Dielectric, DiffuseLight, GravitySphere, Hittable, HittableList, Lambertian,
    Material, Metal, MovingSphere, RectPrism, RotateY, Sphere, Translate, XyRect, XzRect, YzRect,
};
use crate::ray::Ray;
use crate::screen::Screen;
use crate::texture::{Checker, Image, Noise, SolidColor};
use crate::vec3::{random, random_range, Color, Point3, Vec3};
use rand::{thread_rng, Rng};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;

const THREADS: usize = 11;

pub struct Config {
    aspect_ratio: f64,
    image_width: i32,
    samples_per_pixel: i32,
    max_depth: i32,
    threads: usize,
}

impl Config {
    pub fn new(
        aspect_ratio: f64,
        image_width: i32,
        samples_per_pixel: i32,
        max_depth: i32,
        threads: usize,
    ) -> Config {
        assert!(threads > 0);
        assert!(image_width > 0);
        assert!(samples_per_pixel > 0);
        assert!(max_depth > 0);
        assert!(threads > 0);

        Config { aspect_ratio, image_width, samples_per_pixel, max_depth, threads }
    }
}

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
            break;
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
                    let center2 = center + Vec3::new(0, 5, 0);
                    list.add(Arc::new(Box::new(MovingSphere::new(
                        center,
                        center2,
                        0.0,
                        10.0,
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

    let bvhnode = BvhNode::from_list(&list, 0.0, 10.0);

    let world: Box<dyn Hittable + Sync> = Box::new(bvhnode);
    //let world: Box<dyn Hittable + Sync> = Box::new(list);
    world
}

fn gen_random_scene_moving() -> Box<dyn Hittable + Sync> {
    let max_time = 100.0;
    let mut rng = thread_rng();
    let mut list = HittableList::new();
    let ground: Arc<Box<dyn Material>> = Arc::new(Box::new(Lambertian::from_pointer(Arc::new(
        Box::new(SolidColor::new(&Color::new(0.8, 0.8, 0.8))),
    ))));
    list.add(Arc::new(Box::new(Sphere::new(
        Vec3::new(0, -1000, -1),
        1000.0,
        ground,
    ))));
    for a in -11..11 {
        for b in -11..11 {
            if i32::abs(a - 0) <= 1 && i32::abs(b - 0) <= 1 {
                continue;
            }
            if i32::abs(a - 4) <= 1 && i32::abs(b - 0) <= 1 {
                continue;
            }
            let choose_mat = rng.gen::<f64>();
            let center = Vec3::new(
                a as f64 + 0.9 * rng.gen::<f64>(),
                1.7 + thread_rng().gen_range(0.0..2.0),
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
                if choose_mat < 1.0 {
                    list.add(Arc::new(Box::new(GravitySphere::new(
                        center,
                        0.0,
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

    let bvhnode = BvhNode::from_list(&list, 0.0, max_time);

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
        Box::new(Image::from_ppm("earthshit.ppm")),
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
    let red: Arc<Box<dyn Material>> =
        Arc::new(Box::new(Lambertian::new(Color::new(0.65, 0.05, 0.05))));
    let white: Arc<Box<dyn Material>> =
        Arc::new(Box::new(Lambertian::new(Color::new(0.73, 0.73, 0.73))));
    let green: Arc<Box<dyn Material>> =
        Arc::new(Box::new(Lambertian::new(Color::new(0.12, 0.45, 0.15))));
    let light: Arc<Box<dyn Material>> =
        Arc::new(Box::new(DiffuseLight::new(&Color::new(15, 15, 15))));
    list.add(Arc::new(Box::new(YzRect::new(
        0.0, 555.0, 0.0, 555.0, 555.0, green,
    ))));
    list.add(Arc::new(Box::new(YzRect::new(
        0.0, 555.0, 0.0, 555.0, 0.0, red,
    ))));
    list.add(Arc::new(Box::new(XzRect::new(
        213.0, 343.0, 227.0, 332.0, 554.0, light,
    ))));
    list.add(Arc::new(Box::new(XzRect::new(
        0.0,
        555.0,
        0.0,
        555.0,
        0.0,
        white.clone(),
    ))));
    list.add(Arc::new(Box::new(XzRect::new(
        0.0,
        555.0,
        0.0,
        555.0,
        555.0,
        white.clone(),
    ))));
    list.add(Arc::new(Box::new(XyRect::new(
        0.0,
        555.0,
        0.0,
        555.0,
        555.0,
        white.clone(),
    ))));

    list.add(Arc::new(Box::new(Translate::new(
        &Vec3::new(265, 0, 295),
        Arc::new(Box::new(RotateY::new(
            15.0,
            Arc::new(Box::new(RectPrism::new(
                &Point3::new(0, 0, 0),
                &Point3::new(165, 330, 165),
                white.clone(),
            ))),
        ))),
    ))));

    list.add(Arc::new(Box::new(Translate::new(
        &Vec3::new(130, 0, 65),
        Arc::new(Box::new(RotateY::new(
            -18.0,
            Arc::new(Box::new(RectPrism::new(
                &Point3::new(0, 0, 0),
                &Point3::new(165, 165, 165),
                white.clone(),
            ))),
        ))),
    ))));

    Box::new(list)
}

fn cornell_smoke() -> Box<dyn Hittable + Sync> {
    let mut list = HittableList::new();
    let red: Arc<Box<dyn Material>> =
        Arc::new(Box::new(Lambertian::new(Color::new(0.65, 0.05, 0.05))));
    let white: Arc<Box<dyn Material>> =
        Arc::new(Box::new(Lambertian::new(Color::new(0.73, 0.73, 0.73))));
    let green: Arc<Box<dyn Material>> =
        Arc::new(Box::new(Lambertian::new(Color::new(0.12, 0.45, 0.15))));
    let light: Arc<Box<dyn Material>> =
        Arc::new(Box::new(DiffuseLight::new(&Color::new(15, 15, 15))));
    list.add(Arc::new(Box::new(YzRect::new(
        0.0, 555.0, 0.0, 555.0, 555.0, green,
    ))));
    list.add(Arc::new(Box::new(YzRect::new(
        0.0, 555.0, 0.0, 555.0, 0.0, red,
    ))));
    list.add(Arc::new(Box::new(XzRect::new(
        213.0, 343.0, 227.0, 332.0, 554.0, light,
    ))));
    list.add(Arc::new(Box::new(XzRect::new(
        0.0,
        555.0,
        0.0,
        555.0,
        0.0,
        white.clone(),
    ))));
    list.add(Arc::new(Box::new(XzRect::new(
        0.0,
        555.0,
        0.0,
        555.0,
        555.0,
        white.clone(),
    ))));
    list.add(Arc::new(Box::new(XyRect::new(
        0.0,
        555.0,
        0.0,
        555.0,
        555.0,
        white.clone(),
    ))));

    list.add(Arc::new(Box::new(ConstantMedium::from_color(
        &Color::new(0, 0, 0),
        0.01,
        Arc::new(Box::new(Translate::new(
            &Vec3::new(265, 0, 295),
            Arc::new(Box::new(RotateY::new(
                15.0,
                Arc::new(Box::new(RectPrism::new(
                    &Point3::new(0, 0, 0),
                    &Point3::new(165, 330, 165),
                    white.clone(),
                ))),
            ))),
        ))),
    ))));

    list.add(Arc::new(Box::new(ConstantMedium::from_color(
        &Color::new(1, 1, 1),
        0.01,
        Arc::new(Box::new(Translate::new(
            &Vec3::new(130, 0, 65),
            Arc::new(Box::new(RotateY::new(
                -18.0,
                Arc::new(Box::new(RectPrism::new(
                    &Point3::new(0, 0, 0),
                    &Point3::new(165, 165, 165),
                    white.clone(),
                ))),
            ))),
        ))),
    ))));

    Box::new(list)
}

fn final_scene() -> Box<dyn Hittable + Sync> {
    let mut list = HittableList::new();
    let mut boxes1 = HittableList::new();
    let ground: Arc<Box<dyn Material>> = Arc::new(Box::new(Lambertian::from_pointer(Arc::new(
        Box::new(SolidColor::new(&Color::new(0.48, 0.83, 0.53))),
    ))));
    let boxes_per_side = 20;
    for i in 0..boxes_per_side {
        for j in 0..boxes_per_side {
            let i = i as f64;
            let j = j as f64;
            let w = 100.0;
            let x0 = -1000.0 + i * w;
            let z0 = -1000.0 + j * w;
            let y0 = 0.0;
            let x1 = x0 + w;
            let y1 = thread_rng().gen_range(1.0..101.0);
            let z1 = z0 + w;
            boxes1.add(Arc::new(Box::new(RectPrism::new(
                &Point3::new(x0, y0, z0),
                &Point3::new(x1, y1, z1),
                ground.clone(),
            ))))
        }
    }
    list.add(Arc::new(Box::new(BvhNode::from_list(&boxes1, 0.0, 1.0))));
    let light: Arc<Box<dyn Material>> = Arc::new(Box::new(DiffuseLight::new(&Color::new(7, 7, 7))));
    list.add(Arc::new(Box::new(XzRect::new(
        123.0, 432.0, 147.0, 412.0, 554.0, light,
    ))));

    let center1 = Point3::new(400, 400, 400);
    let center2 = center1 + Vec3::new(30, 0, 0);

    list.add(Arc::new(Box::new(MovingSphere::new(
        center1,
        center2,
        0.0,
        1.0,
        50.0,
        Arc::new(Box::new(Lambertian::new(Color::new(0.7, 0.3, 1)))),
    ))));
    list.add(Arc::new(Box::new(Sphere::new(
        Point3::new(260, 150, 45),
        50.0,
        Arc::new(Box::new(Dielectric::new(1.5))),
    ))));

    list.add(Arc::new(Box::new(Sphere::new(
        Point3::new(0, 150, 145),
        50.0,
        Arc::new(Box::new(Metal::new(Color::new(0.8, 0.8, 0.9), 1.0))),
    ))));

    list.add(Arc::new(Box::new(Sphere::new(
        Point3::new(360, 150, 145),
        70.0,
        Arc::new(Box::new(Dielectric::new(1.5))),
    ))));

    list.add(Arc::new(Box::new(ConstantMedium::from_color(
        &Color::new(0.2, 0.4, 0.9),
        0.2,
        Arc::new(Box::new(Sphere::new(
            Point3::new(360, 150, 145),
            70.0,
            Arc::new(Box::new(Dielectric::new(1.5))),
        ))),
    ))));

    list.add(Arc::new(Box::new(Sphere::new(
        Point3::new(0, 0, 0),
        5000.0,
        Arc::new(Box::new(Dielectric::new(1.5))),
    ))));
    list.add(Arc::new(Box::new(ConstantMedium::from_color(
        &Color::new(1, 1, 1),
        0.0001,
        Arc::new(Box::new(Sphere::new(
            Point3::new(0, 0, 0),
            5000.0,
            Arc::new(Box::new(Dielectric::new(1.5))),
        ))),
    ))));

    let ground: Arc<Box<dyn Material>> = Arc::new(Box::new(Lambertian::from_pointer(Arc::new(
        Box::new(Image::from_ppm("earthshit.ppm")),
    ))));
    list.add(Arc::new(Box::new(Sphere::new(
        Vec3::new(400, 200, 400),
        100.0,
        ground.clone(),
    ))));

    list.add(Arc::new(Box::new(Sphere::new(
        Point3::new(220, 280, 300),
        80.0,
        Arc::new(Box::new(Lambertian::from_pointer(Arc::new(Box::new(
            Noise::new(0.1),
        ))))),
    ))));

    let white: Arc<Box<dyn Material>> =
        Arc::new(Box::new(Lambertian::new(Color::new(0.73, 0.73, 0.73))));
    let mut boxes2 = HittableList::new();
    let ns = 1000;
    for _ in 0..ns {
        boxes2.add(Arc::new(Box::new(Sphere::new(
            random_range(0.0, 165.0),
            10.0,
            white.clone(),
        ))))
    }
    list.add(Arc::new(Box::new(Translate::new(
        &Vec3::new(-100, 270, 395),
        Arc::new(Box::new(RotateY::new(
            15.0,
            Arc::new(Box::new(BvhNode::from_list(&boxes2, 0.0, 1.0))),
        ))),
    ))));

    Box::new(list)
}

fn gen_moving_test() -> Box<dyn Hittable + Sync> {
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
    let albedo = Color::new(1, 0, 0);
    let sphere_material = Box::new(Lambertian::new(albedo));
    let center1 = Vec3::new(2, -1, 2);

    let center2 = Vec3::new(2, 7, 2);
    list.add(Arc::new(Box::new(MovingSphere::new(
        center1,
        center2,
        0.0,
        10.0,
        1.0,
        Arc::new(sphere_material),
    ))));
    let bvhnode = BvhNode::from_list(&list, 0.0, 10.0);

    let world: Box<dyn Hittable + Sync> = Box::new(bvhnode);
    //let world: Box<dyn Hittable + Sync> = Box::new(list);
    world
}

fn benchmark_test_scene() -> Box<dyn Hittable + Sync> {
    let inner = Sphere::new(Vec3::new(0,0,0), 4.0, Arc::new(Box::new(
        Lambertian::new(Vec3::new(0.5,0.5,0.5))
    )));
    let mut amit = HittableList::new();
    amit.add(Arc::new(Box::new(inner)));
    for _ in 0..19 {
        let mut tramit = HittableList::new();
        tramit.add(Arc::new(Box::new(amit)));
        amit = tramit;
    }
    Box::new(amit)
}

pub fn get_world_cam(config_num: usize) -> (Arc<Box<dyn Hittable + Sync>>, Arc<Camera>, Color) {
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
            let aperture = 0.0;
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
            let aperture = 0.0;
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
            let aperture = 0.0;
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
            let aperture = 0.0;
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
        4 => {
            let world: Arc<Box<dyn Hittable + Sync>> = Arc::new(cornell_box());
            // camera
            let lookfrom = Vec3::new(278, 278, -800);
            let lookat = Vec3::new(278, 278, 0);
            let vup = Vec3::new(0, 1, 0);
            let dist_to_focus = 10.0;
            let aperture = 0.0;
            let cam = Arc::new(Camera::new(
                lookfrom,
                lookat,
                vup,
                40.0,
                1.0,
                aperture,
                dist_to_focus,
                0.0,
                1.0,
            ));
            return (world, cam, Color::new(0, 0, 0));
        }
        5 => {
            let world: Arc<Box<dyn Hittable + Sync>> = Arc::new(cornell_smoke());
            // camera
            let lookfrom = Vec3::new(278, 278, -800);
            let lookat = Vec3::new(278, 278, 0);
            let vup = Vec3::new(0, 1, 0);
            let dist_to_focus = 10.0;
            let aperture = 0.0;
            let cam = Arc::new(Camera::new(
                lookfrom,
                lookat,
                vup,
                40.0,
                1.0,
                aperture,
                dist_to_focus,
                0.0,
                1.0,
            ));
            return (world, cam, Color::new(0, 0, 0));
        }
        6 => {
            let world: Arc<Box<dyn Hittable + Sync>> = Arc::new(final_scene());
            // camera
            let lookfrom = Vec3::new(478, 278, -600);
            let lookat = Vec3::new(278, 278, 0);
            let vup = Vec3::new(0, 1, 0);
            let dist_to_focus = 10.0;
            let aperture = 0.0;
            let cam = Arc::new(Camera::new(
                lookfrom,
                lookat,
                vup,
                40.0,
                1.0,
                aperture,
                dist_to_focus,
                0.0,
                1.0,
            ));
            return (world, cam, Color::new(0, 0, 0));
        }
        7 => {
            let world: Arc<Box<dyn Hittable + Sync>> = Arc::new(gen_moving_test());
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
                2.0,
                2.5,
            ));
            return (world, cam, background);
        }
        8 => {
            let world: Arc<Box<dyn Hittable + Sync>> = Arc::new(gen_random_scene_moving());
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
                10.0,
            ));
            return (world, cam, background);
        }
        9 => {
            let world = Arc::new(benchmark_test_scene());
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
                10.0,
            ));
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
                10.0,
            ));
            return (world, cam, background);
        }
    }
}

pub fn render_scene(
    world: Arc<Box<dyn Hittable + Sync>>,
    cam: Arc<Camera>,
    background: Vec3,
    config: Config,
) {
    let (sender, receiver) = channel();

    // image
    let aspect_ratio = config.aspect_ratio;
    let image_width = config.image_width;
    let image_height: i32 = (image_width as f64 / aspect_ratio) as i32;
    let samples_per_pixel = config.samples_per_pixel;
    let max_depth = config.max_depth;

    let mut screen = Screen::new(image_width as usize, image_height as usize);

    let chunk_size = image_height as usize / config.threads;

    for t in 0..config.threads {
        let start = t * chunk_size;
        let end = usize::min(t * chunk_size + chunk_size, image_height as usize);
        let send_clone = sender.clone();
        let shared_world: Arc<Box<dyn Hittable + Sync>> = world.clone();
        let shared_cam = cam.clone();

        thread::spawn(move || {
            for j in start..end {
                for i in 0..image_width {
                    let mut pixel = Vec3::new(0, 0, 0);
                    for _ in 0..samples_per_pixel {
                        let u = (i as f64 + thread_rng().gen::<f64>()) / (image_width - 1) as f64;
                        let v = (j as f64 + thread_rng().gen::<f64>()) / (image_height - 1) as f64;
                        let r = shared_cam.get_ray(u, v);
                        pixel += ray_color(&r, &background, shared_world.as_ref(), max_depth);
                    }
                    send_clone
                        .send((
                            j as usize,
                            i as usize,
                            pixel.get_normalized_color(samples_per_pixel as u32),
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

    screen.write_to_ppm();
}

pub fn render_scene_with_time(t0: f64, t1: f64, path: &str, world: Arc<Box<dyn Hittable + Sync>>) {
    let (sender, receiver) = channel();

    let background = Color::new(0.7, 0.8, 1);
    let aspect_ratio: f64 = 1.0;
    let image_width = 500;
    let image_height = (image_width as f64 / aspect_ratio) as i32;
    let samples_per_pixel = 500;
    let max_depth = 50;
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
        t0,
        t1,
    ));
    // image

    // let world: Box<dyn Hittable + Sync> = gen_random_scene();

    let mut screen = Screen::new(image_width as usize, image_height as usize);

    let chunk_size = image_height as usize / THREADS;

    for t in 0..THREADS {
        let start = t * chunk_size;
        let end = usize::min(t * chunk_size + chunk_size, image_height as usize);
        let send_clone = sender.clone();
        let shared_world: Arc<Box<dyn Hittable + Sync>> = world.clone();
        let shared_cam = cam.clone();

        thread::spawn(move || {
            for j in start..end {
                for i in 0..image_width {
                    let mut pixel = Vec3::new(0, 0, 0);
                    for _ in 0..samples_per_pixel {
                        let u = (i as f64 + thread_rng().gen::<f64>()) / (image_width - 1) as f64;
                        let v = (j as f64 + thread_rng().gen::<f64>()) / (image_height - 1) as f64;
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
        if (loops % 20000) == 0 {
            eprintln!("\rDone {} many loops out of {}", loops, total);
        }
    }

    screen.write_to_ppm_file(path);
}
