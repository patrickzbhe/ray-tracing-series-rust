use ray_tracing_series_rust::world::*;
use std::time::Instant;

const THREADS: usize = 11;
const SCENE_ID: usize = 11;

fn main() {
    let start = Instant::now();

    let (world, cam, background) = get_world_cam(SCENE_ID);
    let config = Config::new(1.6, 600, 1000, 50, THREADS);

    render_scene(world, cam, background, config);

    eprintln!("Time taken: {:.3?}", start.elapsed());
}
