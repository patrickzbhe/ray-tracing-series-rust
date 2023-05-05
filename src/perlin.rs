use rand::{thread_rng, Rng};
use crate::vec3::Point3;


const POINT_COUNT: i32 = 256;

pub struct Perlin {
    ranfloat: Vec<f64>,
    perm_x: Vec<i32>,
    perm_y: Vec<i32>,
    perm_z: Vec<i32>,
}

impl Perlin {
    pub fn new() -> Perlin {
        let mut rng = thread_rng();
        let mut ranfloat: Vec<f64> = vec![0.0; POINT_COUNT as usize];
        for i in 0..(POINT_COUNT as usize) {
            ranfloat[i] = rng.gen();
        }
        Perlin {ranfloat, perm_x: Perlin::perlin_generate_perm(), perm_y: Perlin::perlin_generate_perm(), perm_z: Perlin::perlin_generate_perm()}
    }

    pub fn noise(&self, p: &Point3) -> f64 {
        let i = ((4.0 * p.x()) as i32 & 255) as usize;
        let j = ((4.0 * p.y()) as i32 & 255) as usize;
        let k = ((4.0 * p.z()) as i32 & 255) as usize;

        self.ranfloat[(self.perm_x[i] ^ self.perm_y[j] ^ self.perm_z[k]) as usize]
    }

    fn perlin_generate_perm() -> Vec<i32> {
        let mut p = vec![0; POINT_COUNT as usize];
        for i in 0..POINT_COUNT {
            p[i as usize] = i;
        }
        Perlin::permute(&mut p);
        p
    }

    fn permute(vec: &mut Vec<i32>) {
        let mut rng = thread_rng();
        for i in (1..vec.len()-1).rev() {
            let target = rng.gen_range(0..i+1);
            (vec[i], vec[target]) = (vec[target], vec[i])
        }
    }
}