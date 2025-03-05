# simple-ffmpeg

Simple zero-dependency single-file Rust crate for generating videos with [ffmpeg](https://https://www.ffmpeg.org/)
In fact, this crate can even be used without `cargo`. Just download [lib.rs](./src/lib.rs) and add it to your source tree as a module.

## Basic Usage

```rust
use simple_ffmpeg as ffmpeg;

let mut ffmpeg = ffmpeg::start("out.mp4", WIDTH, HEIGHT, FPS)?;

let mut pixels = [0u32; WIDTH * HEIGHT]
for _ in 0..(DURATION * FPS) {
    // <draw frame into pixels array>

    ffmpeg.send_frame(&pixels)?;
}

ffmpeg.finalize()?;
```

