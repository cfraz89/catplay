use catplay_iap2_link::{iap2_gen_checksum, iap2_gen_checksum_fast};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

fn bench_iap2_gen_checksum(c: &mut Criterion) {
    let mut group = c.benchmark_group("iap2_gen_checksum");

    for size in [8usize, 32, 128, 512, 2048, 4096, 8192, 16384, 32768, 65536] {
        let payload: Vec<u8> = (0..size).map(|i| (i as u8).wrapping_mul(31)).collect();
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::new("baseline", size), &payload, |b, data| {
            b.iter(|| black_box(iap2_gen_checksum(black_box(data))));
        });
        group.bench_with_input(BenchmarkId::new("fast", size), &payload, |b, data| {
            b.iter(|| black_box(iap2_gen_checksum_fast(black_box(data))));
        });
    }

    group.finish();
}

criterion_group!(benches, bench_iap2_gen_checksum);
criterion_main!(benches);
