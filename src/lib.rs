use rand::Rng;
use std::usize;
use std::cmp;
use std::path::Path;

pub fn simulate_dla<R>(rng: &mut R,
                       width: u32,
                       height: u32,
                       iterations: u32,
                       seeds: &[(u32, u32)],
                       pruning: Option<Pruning>) -> Space2d
    where R: Rng
{
    let mut space = Space2d::new(width, height);

    for &(x, y) in seeds {
        space.set_seed(x, y, 0);
    }

    for i in 0..iterations {
        space.random_walk(i as i32, rng);
        if let Some(ref prune) = pruning {
            if i % prune.every == 0 {
                space.prune(prune.probability, i as i32 - prune.age, rng);
            }
        }
    }

    space
}

// TODO: figure out how pruning works? no documentation :(
pub struct Pruning {
    pub probability: f32,
    pub every: u32,
    pub age: i32,
}

pub struct Space2d {
    width: u32,
    height: u32,

    /// For every point in the space, store information about a resting particle.
    aggregates: Vec<Aggregate>,

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
            aggregates: (0..nelems).map(|_| Aggregate::empty()).collect(),
            neighborhood: (0..nelems).map(|_| 0).collect(),
        }
    }

    fn in_free_space(&self, x: u32, y: u32) -> bool {
        let idx = self.xy_to_index(x, y);
        self.aggregates[idx].age < 0
    }

    fn attaches(&self, x: u32, y: u32) -> bool {
        self.neighborhood[self.xy_to_index(x, y)] != 0
    }

    pub fn random_walk<R: Rng>(&mut self, iter: i32, rng: &mut R) {
        let mut x;
        let mut y;

        // find free space
        loop {
            x = rng.gen_range(0..self.width);
            y = rng.gen_range(0..self.height);

            if self.in_free_space(x,y) {
                break;
            }
        }

        // now simulate until it hits another particle.
        loop {
            if self.attaches(x, y) {
                self.set_aggregate(x, y, iter, rng);
                break;
            }

            let mut x_i32: i32 = x as i32;
            let mut y_i32: i32 = y as i32;

            let dx: i32 = rng.gen_range(0i32..3) - 1;
            let dy: i32 = rng.gen_range(0i32..3) - 1;

            x_i32 += dx;
            y_i32 += dy;

            if x_i32 >= 0 && x_i32 < self.width as i32 {
                x = x_i32 as u32;
            }

            if y_i32 >= 0 && y_i32 < self.height as i32 {
                y = y_i32 as u32;
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
            n => potential_parents.as_slice()[rng.gen_range(0..n)],
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

    fn get_age(&self, x: u32, y: u32) -> Option<u32> {
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

    // Returns a width * height grayscale image - pixels part of dla aggregate will be black
    pub fn to_image_buf(&self) -> image::GrayImage {
        let mut imgbuf = image::GrayImage::from_pixel(self.width, self.height, image::Luma([255]));

        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            // change pixel color to black
            // TODO: this changes all pixels to black - we want only some of them to be black
            // TODO: query blah
            *pixel = image::Luma([0]);
        }

        imgbuf
    }

    pub fn save_image(&self, filename: &str) {
        image::DynamicImage::ImageLuma8(self.to_image_buf()).save(&Path::new(filename)).unwrap();
    } 
}

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
