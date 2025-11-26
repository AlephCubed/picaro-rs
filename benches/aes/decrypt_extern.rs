use aes::Aes128;
use aes::cipher::{Block, BlockDecrypt, KeyInit};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

pub fn criterion_benchmark(c: &mut Criterion) {
    let aes = Aes128::new_from_slice(&1234u128.to_ne_bytes()).unwrap();
    c.bench_function("AES Decrypt External", |b| {
        let mut block = *Block::<Aes128>::from_slice(&1234u128.to_ne_bytes());
        b.iter(|| aes.decrypt_block(black_box(&mut block)));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
