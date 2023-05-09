# Ray Tracing Series in Rust

Implementation (+ additional experimentation) of Peter Shirley's [Ray Tracing in One Weekend Series](https://raytracing.github.io/).

# Implementation

Trying to write (somewhat) idiomatic Rust.

Also adding some random stuff (like image buffer, importing/export different file formats, spectral rendering) plus performance optimizations, which I'll benchmark.

Using minimal dependencies (just rand right now).

## Benchmarking
### In One Weekend Final
#### Set up
```Rust
let aspect_ratio = 3.0 / 2.0;
let image_width = 800;
let image_height = (image_width as f64 / aspect_ratio) as i32;
let samples_per_pixel = 500;
let max_depth = 50;
```
![Ray Tracing in One Week Final Image](/images/book1.png)
| Version                    | Time (s) |
|----------------------------|----------|
| Brute force + stupid stuff | 936.957s |
| Iterative tracing          | 903.980s |
| Parallel (self built mspc); 10 threads | 146.440s |

### Next Week Final
#### Set up
```Rust
let aspect_ratio = 1.0;
let image_width = 1000;
let image_height = (image_width as f64 / aspect_ratio) as i32;
let samples_per_pixel = 10000;
let max_depth = 50;
```
![Ray Tracing in One Week Final Image](/images/book2.png)
| Version                    | Time (s) |
|----------------------------|----------|
| 10 threads | 12453.138s |

### Video testing
![Video testing](/images/bouncing.gif)

