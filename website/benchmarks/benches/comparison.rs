#![feature(test)]

extern crate test;

use test::{Bencher, black_box};

const CANARY_RANGE: &str = "^19.3.0-canary-561ed529-20260423";
const CANDIDATE: &str = "19.3.0-canary-72135096-20260421";

#[bench]
fn version_js_semver(b: &mut Bencher) {
    b.iter(|| js_semver::Version::parse(black_box("1.2.3")));
}

#[bench]
fn version_node_semver(b: &mut Bencher) {
    b.iter(|| node_semver::Version::parse(black_box("1.2.3")));
}

#[bench]
fn range_js_semver(b: &mut Bencher) {
    b.iter(|| js_semver::Range::parse(black_box("^19.3.0")));
}

#[bench]
fn range_node_semver(b: &mut Bencher) {
    b.iter(|| node_semver::Range::parse(black_box("^19.3.0")));
}

#[bench]
fn parse_and_satisfies_js_semver(b: &mut Bencher) {
    b.iter(|| {
        let range = js_semver::Range::parse(black_box(CANARY_RANGE)).unwrap();
        let version = js_semver::Version::parse(black_box(CANDIDATE)).unwrap();
        range.satisfies(black_box(&version))
    });
}

#[bench]
fn parse_and_satisfies_node_semver(b: &mut Bencher) {
    b.iter(|| {
        let range = node_semver::Range::parse(black_box(CANARY_RANGE)).unwrap();
        let version = node_semver::Version::parse(black_box(CANDIDATE)).unwrap();
        range.satisfies(black_box(&version))
    });
}
