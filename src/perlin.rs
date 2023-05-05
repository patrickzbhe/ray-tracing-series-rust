use crate::vec3::{random_range, Point3, Vec3};
use rand::{thread_rng, Rng};

const POINT_COUNT: i32 = 256;

pub struct Perlin {
    ranvec: Vec<Vec3>,
    perm_x: Vec<i32>,
    perm_y: Vec<i32>,
    perm_z: Vec<i32>,
}

impl Perlin {
    pub fn new() -> Perlin {
        let mut rng = thread_rng();
        let mut ranvec: Vec<Vec3> = vec![];
        for i in 0..(POINT_COUNT as usize) {
            ranvec.push(random_range(-1.0, 1.0));
        }
        Perlin {
            ranvec,
            perm_x: Perlin::perlin_generate_perm(),
            perm_y: Perlin::perlin_generate_perm(),
            perm_z: Perlin::perlin_generate_perm(),
        }
    }

    pub fn noise(&self, p: &Point3) -> f64 {
        let u = p.x() - f64::floor(p.x());
        let v = p.y() - f64::floor(p.y());
        let w = p.z() - f64::floor(p.z());

        let i = f64::floor(p.x()) as i32;
        let j = f64::floor(p.y()) as i32;
        let k = f64::floor(p.z()) as i32;

        let mut c: [[[Vec3; 2]; 2]; 2] = [[[Vec3::new(0, 0, 0); 2]; 2]; 2];
        // LMAO
        // TODO fix types
        for di in 0..2 as usize {
            for dj in 0..2 as usize {
                for dk in 0..2 as usize {
                    c[di][dj][dk] = self.ranvec[(self.perm_x[((i + di as i32) & 255) as usize]
                        ^ self.perm_y[((j + dj as i32) & 255) as usize]
                        ^ self.perm_z[((k + dk as i32) & 255) as usize])
                        as usize];
                }
            }
        }

        Perlin::trilinear_interp(c, u, v, w)
    }

    pub fn turbulence(&self, p: &Point3, depth: usize) -> f64 {
        let mut accum = 0.0;
        let mut temp_p = p.clone();
        let mut weight = 1.0;

        for _ in 0..depth {
            accum += weight * self.noise(&temp_p);
            weight *= 0.5;
            temp_p = temp_p * 2;
        }

        return f64::abs(accum);
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
        for i in (1..vec.len() - 1).rev() {
            let target = rng.gen_range(0..i + 1);
            (vec[i], vec[target]) = (vec[target], vec[i])
        }
    }

    fn trilinear_interp(c: [[[Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        // TODO: there is def a more idiomatic way to write this in rust...
        let uu = u * u * (3.0 - 2.0 * u);
        let vv = v * v * (3.0 - 2.0 * v);
        let ww = w * w * (3.0 - 2.0 * w);
        let mut accum = 0.0;
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let i1 = i as f64;
                    let j1 = j as f64;
                    let k1 = k as f64;
                    let weight_v = Vec3::new(u - i as f64, v - j as f64, w - k as f64);
                    accum += (i1 * uu + (1.0 - i1) * (1.0 - uu))
                        * (j1 * vv + (1.0 - j1) * (1.0 - vv))
                        * (k1 * ww + (1.0 - k1) * (1.0 - ww))
                        * c[i as usize][j as usize][k as usize].dot(&weight_v);
                }
            }
        }
        accum
    }
}
