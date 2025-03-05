#![warn(missing_docs)]

//! # Simple zero-dependency single-file crate for generating videos with ffmpeg in Rust
//! This crate is meant to be extremely light-weight. If you need a feature this crate doesn't provide,
//! go use something else.
//!
//! In fact, this crate can even be used without `cargo`. Just download `lib.rs` and add it to your source tree
//! as a module.
//!
//! ## Basic Usage
//! ```rust
//! use simple_ffmpeg as ffmpeg;
//!
//! let mut ffmpeg = ffmpeg::start("out.mp4", WIDTH, HEIGHT, FPS)?;
//!
//! let mut pixels = [0u32; WIDTH * HEIGHT]
//! for _ in 0..(DURATION * FPS) {
//!     // <draw frame into pixels array>
//!
//!     ffmpeg.send_frame(&pixels)?;
//! }
//!
//! ffmpeg.finalize()?;
//! ```

use std::error;
use std::result;
use std::fmt;
use std::process::{Command, Child, Stdio, ExitStatus};
use std::io::Write;
use std::ffi::OsStr;

/// Representation of a single pixel
///
/// This library assumes you store colors as RGBA32. Note that you have to keep byte order in mind when creating
/// colors. So green with alpha 0 would be 0x0000FF00 on a little-endian machine and 0x00FF0000 on a big-endian machine.
/// To solve this, a function called [`get_color`] is provided
pub type Color = u32;

/// Turn separate R, G, B, and A values into a single RGBA [`Color`]
///
/// This works regardless of endianness
pub fn get_color(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_be_bytes([r, g, b, a])
}

/// Main error type
///
/// This error is returned from every function in this crate that can fail (which is most of them)
#[derive(Debug)]
pub enum Error {
    /// IO Error
    IOError(std::io::Error),
    /// FFMpeg exited with non-zero code
    FFMpegExitedAbnormally(ExitStatus),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::FFMpegExitedAbnormally(code) => if let Some(code) = code.code() {
                write!(f, "ffmpeg exited abnormally with code {code}")
            } else {
                write!(f, "ffmpeg exited abnormally")
            },
            Error::IOError(e) => write!(f, "io error: {e}"),
        }
    }
}

impl error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error { Error::IOError(e) }
}

/// Main Result type
///
/// This Result is returned from every function in this crate that can fail (which is most of them)
pub type Result<T> = result::Result<T, Error>;

/// Main interface into FFMPEG
///
/// This struct holds a child ffmpeg process that you can send frames into. Remember to call [`FFMpeg::finalize`] when you're done.
/// Dropping this struct basically calls `finalize` anyway but it ignores errors, so it's better to call `finalize` explicitly
pub struct FFMpeg {
    child: Child,
    width: usize,
    height: usize,
    fps: u32,
}

/// Start the FFMPEG rendering
///
/// Alias for [`FFMpeg::start`]
pub fn start(out_file: impl AsRef<OsStr>, width: usize, height: usize, fps: u32) -> Result<FFMpeg> {
    FFMpeg::start(out_file, width, height, fps)
}

impl FFMpeg {
    /// Start the FFMPEG rendering
    ///
    /// Starts the FFMPEG rendering.
    pub fn start(out_file: impl AsRef<OsStr>, width: usize, height: usize, fps: u32) -> Result<FFMpeg> {
        let child = Command::new("ffmpeg")
            .args(["-loglevel", "verbose", "-y"])
            // Input file options
            .args(["-f", "rawvideo"])
            .args(["-pix_fmt", "rgba"])
            .args(["-s", &format!("{width}x{height}")])
            .args(["-r", &format!("{fps}")])
            .args(["-i", "-"])
            // Output file options
            .arg(out_file)
            .stdin(Stdio::piped())
            .spawn()?;

        Ok(FFMpeg { child, width, height, fps })
    }

    /// Get the render width
    pub fn width(&self) -> usize { self.width }

    /// Get the render height
    pub fn height(&self) -> usize { self.height }

    /// Get the render FPS
    pub fn fps(&self) -> u32 { self.fps }

    /// Get the render resolution
    pub fn resolution(&self) -> (usize, usize) { (self.width, self.height) }

    /// Send a frame to the FFMPEG process
    ///
    /// Send a frame to the FFMPEG process. `pixels.len()` must be equal to `ffmpeg.width() * ffmpeg.height()`
    pub fn send_frame(&mut self, pixels: &[Color]) -> Result<()> {
        assert_eq!(pixels.len(), self.width * self.height);

        let stdin = self.child.stdin.as_mut().expect("we set stdin to piped");

        let pixels_u8: &[u8] = unsafe {
            let ptr = pixels.as_ptr();
            let len = pixels.len();

            use std::mem::size_of;
            std::slice::from_raw_parts(ptr as *const u8, len * (size_of::<Color>() / size_of::<u8>()))
        };
        stdin.write_all(pixels_u8)?;

        Ok(())
    }

    /// Finalize the FFMPEG rendering
    ///
    /// If this method isn't called directly or indirectly (such as if `std::mem::forget` is called on `FFMpeg`),
    /// the final video may not be complete
    pub fn finalize(mut self) -> Result<()> {
        let retcode = self.child.wait()?;
        if !retcode.success() {
            return Err(Error::FFMpegExitedAbnormally(retcode));
        }
        Ok(())
    }
}

impl std::ops::Drop for FFMpeg {
    fn drop(&mut self) {
        _ = self.child.wait();
    }
}
