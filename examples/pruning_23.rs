extern crate dla;
extern crate pcg;
extern crate rand;

use pcg::PcgRng;
use rand::SeedableRng;
use dla::{simulate_dla, Pruning};

fn main() {
    let mut rng: PcgRng = SeedableRng::from_seed([0, 0]);
    const W: u32 = 400;
    const H: u32 = 180;
    const N: u32 = 20_000;

    const N_IN: u32 = 2;
    const N_OUT: u32 = 3;
    const SEED_HALF_WIDTH: u32 = 2;
    const SEED_HEIGHT: u32 = 2;

    let mut seeds = Vec::new();

    for i in 0..N_IN {
        for y in 0..SEED_HEIGHT {
            for x in 0..SEED_HALF_WIDTH {
                seeds.push(((i + 1) * W / (N_IN + 1) + x, y));
                seeds.push(((i + 1) * W / (N_IN + 1) - x, y));
            }
        }
    }

    for i in 0..N_OUT {
        for y in 0..SEED_HEIGHT {
            for x in 0..SEED_HALF_WIDTH {
                seeds.push(((i + 1) * W / (N_OUT + 1) + x, H - 1 - y));
                seeds.push(((i + 1) * W / (N_OUT + 1) - x, H - 1 - y));
            }
        }
    }

    let pruning = Some(Pruning {
        probability: 0.5,
        every: 10,
        age: 40,
    });

    let colors = [(0, 0, 0), (255, 0, 0), (0, 255, 0), (0, 0, 255), (255, 0, 255), (0, 255, 255)];
    simulate_dla(&mut rng,
                 W,
                 H,
                 N,
                 &seeds,
                 &colors,
                 2000,
                 pruning,
                 500,
                 "dla_pruning_23");
}
