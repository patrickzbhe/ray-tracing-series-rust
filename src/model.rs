use crate::hit::{HittableList, Lambertian, Triangle};
use crate::vec3::{Color, Point3};
use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;

pub struct TriangleModel {
    vertices: Vec<Point3>,
    faces: Vec<(usize, usize, usize)>,
}

impl TriangleModel {
    pub fn load_from_file(path: &str, scale: f64) -> TriangleModel {
        // kind of hard coded
        let mut file = File::open(path).expect("Couldn't open the file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Trouble reading file...");
        let mut contents = contents.split("\n");
        let mut vertex_count = 0;
        let mut face_count = 0;
        loop {
            let line = contents.next().unwrap();
            if line == "end_header" {
                break;
            }
            let line_contents: Vec<&str> = line.split(" ").collect();
            if line_contents[0] == "element" {
                if line_contents[1] == "vertex" {
                    vertex_count = line_contents[2].parse::<i32>().unwrap();
                }
                if line_contents[1] == "face" {
                    face_count = line_contents[2].parse::<i32>().unwrap();
                }
            }
        }
        let mut vertices = vec![];
        let mut faces = vec![];

        for _ in 0..vertex_count {
            let line = contents.next().unwrap();
            let line_contents: Vec<&str> = line.split(" ").collect();
            vertices.push(Point3::new(
                line_contents[0].parse::<f64>().unwrap() * scale,
                line_contents[1].parse::<f64>().unwrap() * scale,
                line_contents[2].parse::<f64>().unwrap() * scale,
            ))
        }

        for _ in 0..face_count {
            let line = contents.next().unwrap();
            let line_contents: Vec<&str> = line.split(" ").collect();

            faces.push((
                line_contents[1].parse::<usize>().unwrap(),
                line_contents[2].parse::<usize>().unwrap(),
                line_contents[3].parse::<usize>().unwrap(),
            ))
        }

        TriangleModel { vertices, faces }
    }

    pub fn to_hittable(&self) -> HittableList {
        let mut triangles = HittableList::new();
        for (v0, v1, v2) in &self.faces {
            //eprintln!("{} {} {}",self.vertices[*v0],self.vertices[*v1],self.vertices[*v2]);
            triangles.add(Arc::new(Box::new(Triangle::new(
                self.vertices[*v0],
                self.vertices[*v1],
                self.vertices[*v2],
                Arc::new(Box::new(Lambertian::new(Color::new(0.2, 0.2, 0.2)))),
            ))));
        }
        triangles
    }
}
