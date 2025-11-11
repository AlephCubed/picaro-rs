use criterion::{Criterion, criterion_group, criterion_main};
use picaro_rs::compression::share_compression;
use picaro_rs::masking::split_u112;
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;
use std::hint::black_box;

fn bench_compression<const SHARE_COUNT: usize>(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::seed_from_u64(1234);

    let shares = split_u112::<SHARE_COUNT>(1234, &mut rng);

    c.bench_function(&format!("Picaro Compression ({SHARE_COUNT} shares)"), |b| {
        b.iter(|| share_compression(black_box(shares)))
    });
}

pub fn criterion_benchmark(c: &mut Criterion) {
    bench_compression::<1>(c);
    bench_compression::<2>(c);
    bench_compression::<3>(c);
    bench_compression::<4>(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
