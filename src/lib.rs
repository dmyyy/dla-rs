extern crate rand;
extern crate image;

use rand::Rng;
use std::fs::File;
use std::path::Path;
use std::usize;
use std::cmp;

/// An aggregate particle attached to some other particle.
#[derive(Copy, Clone)]
struct Aggregate {
    /// Age (iteration number). If negative, aggregate does not exist.
    age: i32,

    /// Counts the number of children we have
    children: u32,

    /// Points to the index of the parent particle
    parent: usize,
}

impl Aggregate {
    fn empty() -> Self {
        Aggregate {
            age: -1,
            children: 0,
            parent: usize::MAX,
        }
    }
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
    aggregates: Vec<Aggregate>,

    /// The attraction neighborhood
    neighborhood: Vec<u8>,
}

struct PotentialParents {
    len: usize,
    arr: [usize; 8],
}

impl PotentialParents {
    fn new() -> Self {
        PotentialParents {
            len: 0,
            arr: [0; 8],
        }
    }
    fn len(&self) -> usize {
        self.len
    }
    fn push(&mut self, element: usize) {
        debug_assert!(self.len < 8);
        self.arr[self.len] = element;
        self.len += 1;
    }
    fn as_slice(&self) -> &[usize] {
        &self.arr[0..self.len]
    }
}

impl Space2d {
    pub fn new(w: u32, h: u32) -> Space2d {
        assert!(w > 0 && h > 0);
        let nelems = (w + 2) as usize * (h + 2) as usize;
        Space2d {
            width: w,
            height: h,
            aggregates: (0..nelems).map(|_| Aggregate::empty()).collect(),
            neighborhood: (0..nelems).map(|_| 0).collect(),
        }
    }

    fn in_free_space(&self, p: &Particle) -> bool {
        let idx = self.xy_to_index(p.x, p.y);
        self.aggregates[idx].age < 0
    }

    fn attaches(&self, p: &Particle) -> bool {
        self.neighborhood[self.xy_to_index(p.x, p.y)] != 0
    }

    pub fn random_walk<R: Rng>(&mut self, iter: i32, rng: &mut R) {
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
                self.set_aggregate(p.x, p.y, iter, rng);
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

    /// Prune terminal aggregate particles with `probability`

    pub fn prune<R>(&mut self, prune_probability: f32, prune_age: i32, rng: &mut R)
        where R: Rng
    {
        let prune_age = cmp::max(prune_age, 0);

        let mut prune_count = 0;

        let n = self.aggregates.len();

        for i in 0..n {
            if self.aggregates[i].children == 0 && self.aggregates[i].age > prune_age {
                // a terminal particle is every particle that has no children
                let v: f32 = rng.gen();
                if v < prune_probability {
                    // prune aggregate
                    prune_count += 1;
                    let parent = self.aggregates[i].parent;
                    self.aggregates[i].age = -1;
                    self.aggregates[i].parent = usize::MAX;

                    self.aggregates[parent].children -= 1;
                }
            }
        }
        if prune_count > 0 {
            self.clear_neighborhood();
            // we have to recalculate the neighborhood
            for idx in 0..n {
                if self.aggregates[idx].age >= 0 {
                    self.set_neighborhood(idx);
                }
            }
        }
    }

    pub fn set_seed(&mut self, x: u32, y: u32, age: i32) {
        let idx = self.xy_to_index(x, y);
        self.aggregates[idx] = Aggregate {
            age: age,
            children: 1,
            parent: idx, // we are a root, so we are ourselves' parent
        };

        self.set_neighborhood(idx);
    }

    /// There can be up to eight potential parent particles which this particle could attach to.
    /// Choose a random one in case there is more than one.

    fn set_aggregate<R>(&mut self, x: u32, y: u32, age: i32, rng: &mut R)
        where R: Rng
    {
        let idx = self.xy_to_index(x, y);

        let ix = x as i32;
        let iy = y as i32;

        let mut potential_parents = PotentialParents::new();

        for &yoff in &[-1, 0, 1] {
            for &xoff in &[-1, 0, 1] {
                if xoff == 0 && yoff == 0 {
                    continue;
                }
                if let Some(i) = self.xy_opt_to_index(ix + xoff, iy + yoff) {
                    if self.aggregates[i].age >= 0 {
                        potential_parents.push(i);
                    }
                }
            }
        }

        let parent = match potential_parents.len() {
            0 => {
                panic!();
            }
            1 => potential_parents.as_slice()[0],
            n => potential_parents.as_slice()[rng.gen_range(0, n)],
        };

        debug_assert!(self.aggregates[idx].age < 0);
        debug_assert!(self.aggregates[parent].age >= 0);

        self.aggregates[idx] = Aggregate {
            age: age,
            children: 0,
            parent: parent,
        };

        // increase the number of children
        self.aggregates[parent].children += 1;

        self.set_neighborhood(idx);
    }

    fn clear_neighborhood(&mut self) {
        for entry in self.neighborhood.iter_mut() {
            *entry = 0;
        }
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
        let age = self.aggregates[idx].age;
        if age < 0 {
            None
        } else {
            Some(age as u32)
        }
    }

    #[inline]
    fn xy_to_index(&self, x: u32, y: u32) -> usize {
        debug_assert!(x < self.width && y < self.height);
        let rw = self.width as usize + 2;
        (y as usize + 1) * rw + x as usize + 1
    }

    #[inline]
    fn xy_opt_to_index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            None
        } else {
            Some(self.xy_to_index(x as u32, y as u32))
        }
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
        space.random_walk(i as i32, rng);
    }
    space.save_png(&format!("{}_final.png", basename), colors, colors_step);
}

pub fn simulate_dla_with_pruning<R>(rng: &mut R,
                                    width: u32,
                                    height: u32,
                                    iterations: u32,
                                    seeds: &[(u32, u32)],
                                    colors: &[(u8, u8, u8)],
                                    colors_step: u32,
                                    prune_probability: f32,  
                                    prune_every: u32,
                                    prune_age: i32,
                                    save_every: u32,
                                    basename: &str)
    where R: Rng
{
    let mut space = Space2d::new(width, height);

    for &(x, y) in seeds {
        space.set_seed(x, y, 0);
    }

    space.save_png(&format!("{}_init.png", basename), colors, colors_step);

    for i in 0..iterations {
        if i % save_every == 0 {
            space.save_png(&format!("{}_{:05}.png", basename, i), colors, colors_step);
        }
        space.random_walk(i as i32, rng);
        if i % prune_every == 0 {
            space.prune(prune_probability, i as i32 - prune_age, rng);
        }
    }
    space.save_png(&format!("{}_final.png", basename), colors, colors_step);
}
