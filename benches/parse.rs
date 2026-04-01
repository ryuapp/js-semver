#![allow(missing_docs)]

use criterion as criterion2;
use criterion2::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use js_semver::{Range, Version};

fn bench_version_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("version_parse");
    const VERSION: &str = "4.5.3";

    group.throughput(Throughput::Bytes(VERSION.len() as u64));
    group.bench_function("version", |b| {
        b.iter(|| {
            black_box(Version::parse(black_box(VERSION)).unwrap());
        });
    });
    group.finish();
}

fn bench_range_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("range_parse");
    const RANGE: &str = "^4.2.0";

    group.throughput(Throughput::Bytes(RANGE.len() as u64));
    group.bench_function("range", |b| {
        b.iter(|| {
            black_box(Range::parse(black_box(RANGE)).unwrap());
        });
    });
    group.finish();
}

fn bench_parse_and_satisfies(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_and_satisfies");
    const RANGE: &str = "^4.1.0-rc";
    const VERSION: &str = "4.1.0-rc.1";
    let bytes = (RANGE.len() + VERSION.len()) as u64;

    group.throughput(Throughput::Bytes(bytes));
    group.bench_function("full", |b| {
        b.iter(|| {
            let parsed_range = black_box(Range::parse(black_box(RANGE)).unwrap());
            let parsed_version = black_box(Version::parse(black_box(VERSION)).unwrap());
            black_box(parsed_range.satisfies(&parsed_version));
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_version_parse,
    bench_range_parse,
    bench_parse_and_satisfies
);
criterion_main!(benches);
