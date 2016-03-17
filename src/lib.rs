extern crate rand;
extern crate image;

use rand::Rng;
use std::fs::File;
use std::path::Path;

/// An aggregate particle attached to some other particle.
#[derive(Copy, Clone)]
struct Aggregate {
    /// Age (iteration number)
    age: u32,

    /// Points to the position of the parent particle
    parent: (u32, u32),
}

/// A moving particle
struct Particle {
    x: u32,
    y: u32,
}

pub struct Space2d {
    width: u32,
    height: u32,

    /// For every point in the space, store information about a resting particle.

    aggregates: Vec<Option<Aggregate>>,

    /// The attraction neighborhood
    neighborhood: Vec<u8>,
}

impl Space2d {
    pub fn new(w: u32, h: u32) -> Space2d {
        assert!(w > 0 && h > 0);
        let nelems = (w + 2) as usize * (h + 2) as usize;
        Space2d {
            width: w,
            height: h,
            aggregates: (0..nelems).map(|_| None).collect(),
            neighborhood: (0..nelems).map(|_| 0).collect(),
        }
    }

    fn in_free_space(&self, p: &Particle) -> bool {
        let idx = self.xy_to_index(p.x, p.y);
        self.aggregates[idx].is_none()
    }

    fn attaches(&self, p: &Particle) -> bool {
        self.neighborhood[self.xy_to_index(p.x, p.y)] != 0
    }

    pub fn random_walk<R: Rng>(&mut self, iter: u32, rng: &mut R) {
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
                self.set_seed(p.x, p.y, iter);
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

    pub fn set_seed(&mut self, x: u32, y: u32, age: u32) {
        let idx = self.xy_to_index(x, y);
        self.aggregates[idx] = Some(Aggregate{
            age: age,
            parent: (x, y), // we are a root, so we are ourselves' parent
        });

        self.set_neighborhood(idx);
    }

    fn set_neighborhood(&mut self, idx: usize) {
        let rw = self.width as usize + 2;
        self.neighborhood[idx - 1 - rw] = 1;
        self.neighborhood[idx - rw] = 1;
        self.neighborhood[idx + 1 - rw] = 1;
        self.neighborhood[idx - 1] = 1;
        self.neighborhood[idx] = 1;
        self.neighborhood[idx + 1] = 1;
        self.neighborhood[idx - 1 + rw] = 1;
        self.neighborhood[idx + rw] = 1;
        self.neighborhood[idx + 1 + rw] = 1;
    }

    fn get_pixel(&self, x: u32, y: u32) -> Option<u32> {
        let idx = self.xy_to_index(x, y);
        self.aggregates[idx].map(|p| p.age)
    }

    #[inline]
    fn xy_to_index(&self, x: u32, y: u32) -> usize {
        debug_assert!(x < self.width && y < self.height);
        let rw = self.width as usize + 2;
        (y as usize + 1) * rw + x as usize + 1
    }

    pub fn save_png(&self, filename: &str, colors: &[(u8, u8, u8)], colors_step: u32) {
        let mut imgbuf = image::RgbaImage::new(self.width, self.height);

        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let (r, g, b) = match self.get_pixel(x, y) {
                None => {
                    // white
                    (255, 255, 255)
                }
                Some(age) => {
                    let age = ((age as u32) / colors_step) as usize;
                    colors[age % colors.len()]
                }
            };
            *pixel = image::Rgba([r, g, b, 255]);
        }

        let ref mut fout = File::create(&Path::new(filename)).unwrap();
        let _ = image::DynamicImage::ImageRgba8(imgbuf).save(fout, image::PNG).unwrap();
    }
}

pub fn simulate_dla<R>(rng: &mut R,
                   width: u32,
                   height: u32,
                   iterations: u32,
                   seeds: &[(u32, u32)],
                   colors: &[(u8, u8, u8)],
                   colors_step: u32,
                   save_every: u32,
                   basename: &str)
    where R: Rng
{
    let mut space = Space2d::new(width, height);

    for &(x, y) in seeds {
        space.set_seed(x, y, 0);
    }

    space.save_png(&format!("{}_init.png", basename), colors, colors_step);

    for i in 1..iterations + 1 {
        if i % save_every == 0 {
            space.save_png(&format!("{}_{:05}.png", basename, i), colors, colors_step);
        }
        space.random_walk(i, rng);
    }
    space.save_png(&format!("{}_final.png", basename), colors, colors_step);
}
