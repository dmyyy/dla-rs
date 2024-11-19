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
    const H: u32 = 300;
    const N: u32 = 20_000;

    let seeds = vec![(W / 2, H / 2)];
    simulate_dla(&mut rng,
                 W,
                 H,
                 N,
                 &seeds,
                 None);
}
