use criterion::{Criterion, criterion_group, criterion_main};
use picaro_rs::Picaro;
use std::hint::black_box;

fn bench_encrypt<const SHARE_COUNT: usize>(c: &mut Criterion) {
    let mut p = Picaro::<SHARE_COUNT>::new_from_seed(1234, 1234);
    c.bench_function(&format!("Picaro Encrypt ({SHARE_COUNT} shares)"), |b| {
        b.iter(|| p.encrypt(black_box(1234)))
    });
}

pub fn criterion_benchmark(c: &mut Criterion) {
    bench_encrypt::<1>(c);
    bench_encrypt::<2>(c);
    bench_encrypt::<3>(c);
    bench_encrypt::<4>(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
