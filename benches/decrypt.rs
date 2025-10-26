use criterion::{Criterion, criterion_group, criterion_main};
use picaro_rs::Picaro;
use std::hint::black_box;

pub fn criterion_benchmark(c: &mut Criterion) {
    let p = Picaro::new(1234);

    c.bench_function("decrypt 1234", |b| b.iter(|| p.decrypt(black_box(1234))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
