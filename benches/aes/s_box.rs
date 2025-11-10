use criterion::{Criterion, criterion_group, criterion_main};
use picaro_rs::aes::{aes_affine_transformation, aes_inversion, aes_s_box};
use picaro_rs::masking::split_u112;
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;
use std::hint::black_box;

fn bench_s_box<const SHARE_COUNT: usize>(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::seed_from_u64(1234);

    let shares = split_u112::<SHARE_COUNT>(1234, &mut rng);

    c.bench_function(&format!("AES S-Box ({SHARE_COUNT} shares)"), |b| {
        b.iter(|| aes_s_box(black_box(shares), &mut rng))
    });
}

fn bench_inversion<const SHARE_COUNT: usize>(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::seed_from_u64(1234);

    let shares = split_u112::<SHARE_COUNT>(1234, &mut rng);

    c.bench_function(&format!("AES Inversion ({SHARE_COUNT} shares)"), |b| {
        b.iter(|| aes_inversion(black_box(shares), &mut rng))
    });
}

fn bench_affine<const SHARE_COUNT: usize>(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::seed_from_u64(1234);

    let shares = split_u112::<SHARE_COUNT>(1234, &mut rng);

    c.bench_function(
        &format!("AES Affine Transformation ({SHARE_COUNT} shares)"),
        |b| b.iter(|| aes_affine_transformation(black_box(shares))),
    );
}

pub fn criterion_benchmark(c: &mut Criterion) {
    bench_s_box::<1>(c);
    bench_s_box::<2>(c);
    bench_s_box::<3>(c);
    bench_s_box::<4>(c);

    bench_inversion::<1>(c);
    bench_inversion::<2>(c);
    bench_inversion::<3>(c);
    bench_inversion::<4>(c);

    bench_affine::<1>(c);
    bench_affine::<2>(c);
    bench_affine::<3>(c);
    bench_affine::<4>(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
