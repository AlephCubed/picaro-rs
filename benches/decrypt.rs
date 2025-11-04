use criterion::{Criterion, criterion_group, criterion_main};
use picaro_rs::Picaro;
use std::hint::black_box;

fn bench_decrypt<const MASKING_LEVEL: usize>(c: &mut Criterion) {
    let p = Picaro::<MASKING_LEVEL>::new(1234);
    c.bench_function(&format!("decrypt level {MASKING_LEVEL}"), |b| {
        b.iter(|| p.decrypt(black_box(1234)))
    });
}

pub fn criterion_benchmark(c: &mut Criterion) {
    bench_decrypt::<0>(c);
    bench_decrypt::<1>(c);
    bench_decrypt::<2>(c);
    bench_decrypt::<3>(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
