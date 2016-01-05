extern crate rand;
extern crate image;
extern crate pcg;

use pcg::PcgRng;
use rand::{Rng, SeedableRng};
use std::fs::File;
use std::path::Path;

fn main() {
    let mut rng: PcgRng = SeedableRng::from_seed([0, 0]);

    let mut imgbuf = image::ImageBuffer::new(200, 200);

    for (_x, _y, pixel) in imgbuf.enumerate_pixels_mut() {
        let luma: u8 = rng.gen();
        *pixel = image::Luma([luma]);
    }

    let ref mut fout = File::create(&Path::new("fractal.png")).unwrap();
    let _ = image::ImageLuma8(imgbuf).save(fout, image::PNG);
}
