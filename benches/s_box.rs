use criterion::{Criterion, criterion_group, criterion_main};
use picaro_rs::masking::{split_u8, split_u112};
use picaro_rs::s_box::{s_box, s_box_byte};
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;
use std::hint::black_box;

fn bench_s_box<const SHARE_COUNT: usize>(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::seed_from_u64(1234);

    let shares = split_u112::<SHARE_COUNT>(1234, &mut rng);

    c.bench_function(&format!("Picaro S-Box ({SHARE_COUNT} shares)"), |b| {
        b.iter(|| s_box(black_box(shares), &mut rng))
    });
}

fn bench_s_box_byte<const SHARE_COUNT: usize>(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::seed_from_u64(1234);

    let shares = split_u8::<SHARE_COUNT>(123, &mut rng);

    c.bench_function(&format!("Picaro S-Box Byte ({SHARE_COUNT} shares)"), |b| {
        b.iter(|| s_box_byte(black_box(shares), &mut rng))
    });
}

pub fn criterion_benchmark(c: &mut Criterion) {
    bench_s_box::<1>(c);
    bench_s_box::<2>(c);
    bench_s_box::<3>(c);
    bench_s_box::<4>(c);

    bench_s_box_byte::<1>(c);
    bench_s_box_byte::<2>(c);
    bench_s_box_byte::<3>(c);
    bench_s_box_byte::<4>(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
