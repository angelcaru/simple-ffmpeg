use simple_ffmpeg as ffmpeg;
use ffmpeg::Color;

const FACTOR: usize = 40;
const WIDTH: usize = 16 * FACTOR;
const HEIGHT: usize = 9 * FACTOR;
const FPS: u32 = 60;
const PIXEL_COUNT: usize = WIDTH * HEIGHT;

fn draw_rect(pixels: &mut [Color], width: usize, x0: i32, y0: i32, w: i32, h: i32, color: Color) {
    let height = pixels.len() / width;
    for y in y0..=(y0 + h) {
        if (0..height as i32).contains(&y) {
            for x in x0..=(x0 + w) {
                if (0..width as i32).contains(&x) {
                    pixels[(y * width as i32 + x) as usize] = color;
                }
            }
        }
    }
}

fn main() -> ffmpeg::Result<()> {
    let mut pixels: [Color; PIXEL_COUNT] = [0; PIXEL_COUNT];

    let mut ffmpeg = ffmpeg::start("out.mp4", WIDTH, HEIGHT, FPS)?;

    let mut x = 0;
    let mut y = 0;
    let mut dx = 1;
    let mut dy = 1;
    let w = 50;
    let h = 50;
    for _ in 0..(60 * FPS) {
        pixels.fill(0);
        draw_rect(&mut pixels, WIDTH, x, y, w, h, 0xFF00FF00);
        ffmpeg.send_frame(&pixels)?;

        let new_x = x + dx;
        if (0..WIDTH as i32 - w).contains(&new_x) {
            x = new_x;
        } else {
            dx *= -1;
        }

        let new_y = y + dy;
        if (0..HEIGHT as i32 - h).contains(&new_y) {
            y = new_y;
        } else {
            dy *= -1;
        }
    }

    ffmpeg.finalize()?;

    Ok(())
}
