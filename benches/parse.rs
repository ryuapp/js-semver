#![feature(test)]
#![allow(missing_docs, reason = "Benchmark items do not need documentation.")]
#![allow(
    clippy::tests_outside_test_module,
    reason = "This crate contains only benchmarks."
)]

extern crate test;

use js_semver::{Range, Version};
use test::{Bencher, black_box};

macro_rules! version_benchmark {
    ($name:ident, $input:literal) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            b.iter(|| black_box(Version::parse(black_box($input))));
        }
    };
}

version_benchmark!(version_parse, "4.5.3");
version_benchmark!(version_parse_prefixed, "v4.5.3");
version_benchmark!(version_parse_whitespace, "  4.5.3  ");
version_benchmark!(version_parse_prerelease, "4.1.0-rc.1");
version_benchmark!(version_parse_build, "4.1.0+build.42");
version_benchmark!(version_parse_prerelease_build, "4.1.0-rc.1+build.42");
version_benchmark!(
    version_parse_long_metadata,
    "19.3.0-canary-044d56f3-20260330+sha.abcdef0123456789"
);
version_benchmark!(version_parse_invalid_core, "04.1.0");
version_benchmark!(version_parse_invalid_metadata, "4.1.0-alpha..1");

#[bench]
fn range_parse(b: &mut Bencher) {
    b.iter(|| black_box(Range::parse(black_box("^4.2.0"))));
}

#[bench]
fn parse_and_satisfies(b: &mut Bencher) {
    b.iter(|| {
        let range = Range::parse(black_box("^4.1.0-rc"));
        let version = Version::parse(black_box("4.1.0-rc.1"));
        black_box(match (range, version) {
            (Ok(range), Ok(version)) => Some(range.satisfies(&version)),
            _ => None,
        })
    });
}
