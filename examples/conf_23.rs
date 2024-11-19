extern crate dla;
extern crate rand;
extern crate bevy_prng;
extern crate bevy_rand;

use rand::SeedableRng;
use dla::simulate_dla;
use bevy_prng::WyRand;
use bevy_rand::prelude::EntropyComponent;

/// `bevy_ecs` compatible non-secure BevyRng alias
pub type BevyRng = EntropyComponent<WyRand>;


fn main() {
    let mut rng: BevyRng = SeedableRng::seed_from_u64(1);
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

    simulate_dla(&mut rng, W, H, N, &seeds, None);
}
