extern crate criterion;
extern crate rand;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::Rng;
use stacks_encoding_native_js::address::c32::{c32_address, c32_address_decode};

fn bench_c32_decoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("C32 Decoding");

    let mut addrs: Vec<String> = vec![];
    for _ in 0..5 {
        // random version
        let random_version: u8 = rand::thread_rng().gen_range(0..31);
        // random 20 bytes
        let random_bytes = rand::thread_rng().gen::<[u8; 20]>();
        let addr = c32_address(random_version, &random_bytes).unwrap();
        addrs.push(addr);
    }

    for addr in addrs.iter() {
        group.bench_with_input(
            BenchmarkId::new("c32_address_decode", addr),
            addr,
            |b, i| b.iter(|| c32_address_decode(i)),
        );
    }
    group.finish();
}

fn bench_c32_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("C32 Decoding");

    let mut addrs: Vec<(String, u8, [u8; 20])> = vec![];
    for _ in 0..5 {
        // random version
        let random_version: u8 = rand::thread_rng().gen_range(0..31);
        // random 20 bytes
        let random_bytes = rand::thread_rng().gen::<[u8; 20]>();
        let addr = c32_address(random_version, &random_bytes).unwrap();
        addrs.push((addr, random_version, random_bytes));
    }

    for addr in addrs.iter() {
        group.bench_with_input(
            BenchmarkId::new("c32_address", addr.0.to_string()),
            addr,
            |b, i| b.iter(|| c32_address(i.1, &i.2)),
        );
    }
    group.finish();
}

criterion_group!(benches, bench_c32_decoding, bench_c32_encoding);
criterion_main!(benches);
