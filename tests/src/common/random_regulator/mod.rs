use rand::Rng;

pub fn gen_random_regulator(max_assumed_mismatch_per_100_bases: u32) -> (u32, u32, u32, u32, f32) {
    let mut rng = rand::thread_rng();

    let px = rng.gen_range(1..10);
    let po = rng.gen_range(0..10);
    let pe = rng.gen_range(1..10);
    
    let minl = rng.gen_range(50..150);
    let maxp = {
        (
            px * rng.gen_range(1..max_assumed_mismatch_per_100_bases) // mismatch per 100 bases
        ) as f32 / 100.0
    };
    (px, po, pe, minl, maxp)
}
pub fn gen_random_regulator_not_errored_in_v032(max_assumed_mismatch_per_100_bases: u32) -> (u32, u32, u32, u32, f32) {
    let mut rng = rand::thread_rng();

    let (px, po, pe) = loop {
        let px = rng.gen_range(1..10);
        let po = rng.gen_range(0..10);
        let pe = rng.gen_range(1..10);

        if px < po + pe {
            break (px, po, pe);
        }
    };
    let minl = rng.gen_range(50..150);
    let maxp = {
        (
            px * rng.gen_range(1..max_assumed_mismatch_per_100_bases) // mismatch per 100 bases
        ) as f32 / 100.0
    };
    (px, po, pe, minl, maxp)
}