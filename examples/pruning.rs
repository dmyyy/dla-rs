extern crate dla;
extern crate pcg;
extern crate rand;

use pcg::PcgRng;
use rand::SeedableRng;
use dla::simulate_dla_with_pruning;

fn main() {
    let mut rng: PcgRng = SeedableRng::from_seed([0, 0]);
    const W: u32 = 400;
    const H: u32 = 300;
    const N: u32 = 20_000;

    let seeds = vec![(W / 2, H / 2)];
    simulate_dla_with_pruning(&mut rng,
                 W,
                 H,
                 N,
                 &seeds,
                 &[(0, 0, 0)],
                 1,
                 0.5,
                 10, // prune every
                 40, // prune max age.
                 500,
                 "dla_middle");
}
