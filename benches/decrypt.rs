use criterion::{Criterion, criterion_group, criterion_main};
use picaro_rs::Picaro;
use std::hint::black_box;

fn bench_decrypt<const SHARE_COUNT: usize>(c: &mut Criterion) {
    let mut p = Picaro::<SHARE_COUNT>::new_from_seed(1234, 1234);
    c.bench_function(&format!("Decrypt ({SHARE_COUNT} shares)"), |b| {
        b.iter(|| p.decrypt(black_box(1234)))
    });
}

pub fn criterion_benchmark(c: &mut Criterion) {
    bench_decrypt::<1>(c);
    bench_decrypt::<2>(c);
    bench_decrypt::<3>(c);
    bench_decrypt::<4>(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
