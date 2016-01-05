extern crate rand;
extern crate image;
extern crate pcg;

use pcg::PcgRng;
use rand::{Rng, SeedableRng};
use std::fs::File;
use std::path::Path;

struct Space2d {
    width: u32,
    height: u32,
    matrix: Vec<u8>,
}

impl Space2d {
    fn new(w: u32, h: u32) -> Space2d {
        assert!(w > 0 && h > 0);
        let nelems = (w + 2) as usize * (h + 2) as usize;
        Space2d {
            width: w,
            height: h,
            matrix: (0..nelems).map(|_| 0).collect(),
        }
    }

    fn in_free_space(&self, p: &Particle) -> bool {
        let idx = self.xy_to_index(p.x, p.y);
        self.matrix[idx] == 0
    }

    fn attaches(&self, p: &Particle) -> bool {
        let idx = self.xy_to_index(p.x, p.y);
        let rw = self.width as usize + 2;
        (self.matrix[idx - 1 - rw] | self.matrix[idx - rw] | self.matrix[idx + 1 - rw] |
         self.matrix[idx - 1] | self.matrix[idx] | self.matrix[idx + 1] |
         self.matrix[idx - 1 + rw] | self.matrix[idx + rw] |
         self.matrix[idx + 1 + rw]) == 1
    }

    fn random_walk<R: Rng>(&mut self, rng: &mut R) {
        let mut p;

        // find free space
        loop {
            p = Particle {
                x: rng.gen_range(0, self.width),
                y: rng.gen_range(0, self.height),
            };

            if self.in_free_space(&p) {
                break;
            }
        }

        // now simulate until it hits another particle.
        loop {
            if self.attaches(&p) {
                self.set_seed(p.x, p.y);
                break;
            }
            let mut x: i32 = p.x as i32;
            let mut y: i32 = p.y as i32;

            let off_x: i32 = rng.gen_range(0i32, 3) - 1;
            let off_y: i32 = rng.gen_range(0i32, 3) - 1;

            x += off_x;
            y += off_y;

            if x >= 0 && x < self.width as i32 {
                p.x = x as u32;
            }

            if y >= 0 && y < self.height as i32 {
                p.y = y as u32;
            }
        }
    }

    fn set_seed(&mut self, x: u32, y: u32) {
        let idx = self.xy_to_index(x, y);
        self.matrix[idx] = 1;
    }

    fn get_pixel(&self, x: u32, y: u32) -> u8 {
        let idx = self.xy_to_index(x, y);
        self.matrix[idx]
    }

    #[inline]
    fn xy_to_index(&self, x: u32, y: u32) -> usize {
        debug_assert!(x < self.width && y < self.height);
        let rw = self.width as usize + 2;
        (y as usize + 1) * rw + x as usize + 1
    }

    fn save_png(&self, filename: &str) {
        let mut imgbuf = image::ImageBuffer::new(self.width, self.height);

        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let luma: u8 = 255 * self.get_pixel(x, y);
            *pixel = image::Luma([luma]);
        }

        let ref mut fout = File::create(&Path::new(filename)).unwrap();
        let _ = image::ImageLuma8(imgbuf).save(fout, image::PNG).unwrap();
    }
}

struct Particle {
    x: u32,
    y: u32,
}

fn main() {
    let mut rng: PcgRng = SeedableRng::from_seed([0, 0]);

    const W: u32 = 400;
    const H: u32 = 400;
    const N: u32 = 10000;

    let mut space = Space2d::new(W, H);
    space.set_seed(W / 2, H / 2);
    for _ in 0..N {
        space.random_walk(&mut rng);
    }

    space.save_png("dla.png");
}
