use criterion::{Criterion, criterion_group, criterion_main};
use picaro_rs::expansion::share_expansion;
use picaro_rs::masking::split_u64;
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;
use std::hint::black_box;

fn bench_expansion<const SHARE_COUNT: usize>(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::seed_from_u64(1234);

    let shares = split_u64::<SHARE_COUNT>(1234, &mut rng);

    c.bench_function(&format!("Picaro Expansion ({SHARE_COUNT} shares)"), |b| {
        b.iter(|| share_expansion(black_box(shares)))
    });
}

pub fn criterion_benchmark(c: &mut Criterion) {
    bench_expansion::<1>(c);
    bench_expansion::<2>(c);
    bench_expansion::<3>(c);
    bench_expansion::<4>(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
