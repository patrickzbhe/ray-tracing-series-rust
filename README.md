# Ray Tracing Series in Rust

Implementation (+ additional experimentation) of Peter Shirley's [Ray Tracing in One Weekend Series](https://raytracing.github.io/).

# Implementation

Starting out by just mostly translating the C++ codebase, then converting it into (somewhat) idiomatic Rust.
Also adding some random stuff plus performance optimizations, which I'll benchmark.
Using minimal dependencies.

## Benchmarking
### In One Weekend Final
#### Set up
```Rust
let aspect_ratio: f64 = 3.0 / 2.0;
let image_width = 800;
let image_height = (image_width as f64 / aspect_ratio) as i32;
let samples_per_pixel = 1;
let max_depth = 50;
```
| Version                    | Time (s) |
|----------------------------|----------|
| Brute force + stupid stuff | 936.957s |
| Iterative tracing          | 903.980s |


