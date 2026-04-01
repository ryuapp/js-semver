#![allow(
    missing_docs,
    reason = "Criterion macros generate undocumented bench items."
)]

use criterion as criterion2;
use criterion2::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use js_semver::{Range, Version};

fn bench_version_parse(c: &mut Criterion) {
    let version = "4.5.3";
    let mut group = c.benchmark_group("version_parse");
    group.throughput(Throughput::Bytes(u64::try_from(version.len()).unwrap_or(0)));
    group.bench_function("version", |b| {
        b.iter(|| {
            if let Ok(parsed) = Version::parse(black_box(version)) {
                black_box(parsed);
            }
        });
    });
    group.finish();
}

fn bench_range_parse(c: &mut Criterion) {
    let range = "^4.2.0";
    let mut group = c.benchmark_group("range_parse");
    group.throughput(Throughput::Bytes(u64::try_from(range.len()).unwrap_or(0)));
    group.bench_function("range", |b| {
        b.iter(|| {
            if let Ok(parsed) = Range::parse(black_box(range)) {
                black_box(parsed);
            }
        });
    });
    group.finish();
}

fn bench_parse_and_satisfies(c: &mut Criterion) {
    let range = "^4.1.0-rc";
    let version = "4.1.0-rc.1";
    let mut group = c.benchmark_group("parse_and_satisfies");
    let bytes = u64::try_from(range.len() + version.len()).unwrap_or(0);

    group.throughput(Throughput::Bytes(bytes));
    group.bench_function("full", |b| {
        b.iter(|| {
            if let (Ok(parsed_range), Ok(parsed_version)) = (
                Range::parse(black_box(range)),
                Version::parse(black_box(version)),
            ) {
                black_box(parsed_range.satisfies(&parsed_version));
            }
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
